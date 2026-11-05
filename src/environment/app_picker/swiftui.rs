/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! On iOS, the app picker is native SwiftUI
//! (`iphone/iDared32bit/AppPicker.swift`) rather than drawn with our own
//! UIKit.

use super::{quick_option_args, quick_options, AppInfo, PickerPerformance, QuickOptionKind};
use crate::image::Image;
use crate::paths;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

/// Shows the picker until an app is picked. Takes JSON describing it and one
/// RGBA icon per app. Returns the picked app's index, then each quick option's
/// value, one per line, in a string to be freed with `free()`.
type PickerFn = unsafe extern "C" fn(*const c_char, *const *const u8, usize) -> *mut c_char;

static PICKER: OnceLock<PickerFn> = OnceLock::new();

/// Called by the iOS app's `main()` with the Swift picker. Rust can't link to
/// it directly, as this crate is also built as a dylib without it.
#[no_mangle]
pub extern "C" fn idared_set_swiftui_app_picker(picker: PickerFn) {
    let _ = PICKER.set(picker);
}

/// Receives bounded construction milestones from the native Swift picker.
/// Values are elapsed microseconds from Swift bridge entry unless the event
/// name explicitly says it includes the preceding Rust scan.
#[no_mangle]
pub unsafe extern "C" fn idared_picker_performance_event(
    scan_id: u64,
    event: *const c_char,
    elapsed_us: u64,
) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CStr::from_ptr(event) }.to_string_lossy();
    echo!(
        "Picker Performance scan #{} Swift: {} = {:.3} ms",
        scan_id,
        event,
        elapsed_us as f64 / 1000.0
    );
}

pub fn is_available() -> bool {
    PICKER.get().is_some()
}

pub enum Picked {
    /// The app's path and option arguments.
    App(PathBuf, Vec<String>),
    /// The Games folder changed, so the picker should be shown again.
    Reload(ReloadState),
}

/// What the picker keeps when it's shown again.
pub struct ReloadState {
    page: usize,
    quick_option_values: Vec<usize>,
}

fn parse_picker_result(paths: &[PathBuf], result: String) -> Result<Picked, String> {
    let mut lines = result.lines();
    if result.starts_with("reload\n") {
        lines.next();
        let page = lines.next().and_then(|page| page.parse().ok()).unwrap_or(0);
        let quick_option_values = lines.map(|value| value.parse().unwrap_or(0)).collect();
        echo!("The Games folder changed, so the app picker is shown again.");
        return Ok(Picked::Reload(ReloadState {
            page,
            quick_option_values,
        }));
    }
    let app = lines
        .next()
        .and_then(|idx| idx.parse::<usize>().ok())
        .and_then(|idx| paths.get(idx))
        .ok_or_else(|| "The app picker returned an invalid app.".to_string())?;
    let values: Vec<usize> = lines.map(|value| value.parse().unwrap_or(0)).collect();
    echo!("Picked: {}", app.display());
    Ok(Picked::App(
        app.clone(),
        quick_option_args(&values).collect(),
    ))
}

pub fn app_picker(
    apps: Result<&[AppInfo], &str>,
    state: Option<ReloadState>,
    performance: PickerPerformance,
) -> Result<Picked, String> {
    let preparation_started = Instant::now();
    let picker = *PICKER.get().unwrap();
    let (apps, error) = match apps {
        Ok(apps) => (apps, None),
        Err(e) => (&[][..], Some(e.to_owned())),
    };

    // Swift draws a "?" on this for apps without an icon.
    let started = Instant::now();
    let mut placeholder = Image::from_pixel_vec([128, 128, 128, 255].repeat(57 * 57), (57, 57));
    // Should match Bundle::load_icon().
    placeholder.round_corners(10.0, /* four_corners: */ true);
    let icons: Vec<&Image> = apps
        .iter()
        .map(|app| app.icon.as_ref().unwrap_or(&placeholder))
        .collect();
    let placeholder_and_icon_refs = started.elapsed();

    let started = Instant::now();
    let wallpaper = paths::wallpaper_files()
        .iter()
        .map(|file| paths::user_data_base_path().join(file))
        .find(|path| path.exists());
    let branding = crate::branding();

    let apps_json: Vec<String> = apps
        .iter()
        .zip(&icons)
        .map(|(app, icon)| {
            let library_key = app
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| app.path.to_string_lossy().into_owned());
            format!(
                "{{\"name\":{},\"identifier\":{},\"libraryKey\":{},\"iconWidth\":{},\"iconHeight\":{},\"hasIcon\":{},\"compatibility\":{}}}",
                json_string(&app.display_name),
                json_string(&app.bundle_identifier),
                json_string(&library_key),
                icon.dimensions().0,
                icon.dimensions().1,
                app.icon.is_some(),
                json_string(super::picker_compatibility(app)),
            )
        })
        .collect();
    let apps_json_elapsed = started.elapsed();
    // Swift shows these, and sends back each one's value: the choice index,
    // or 1 for a switch that's on. Only Rust knows the option arguments.
    let started = Instant::now();
    let quick_options_json: Vec<String> = quick_options()
        .map(|option| {
            let kind = match option.kind {
                QuickOptionKind::Choice(choices) => {
                    let names: Vec<String> = choices.iter().map(|c| json_string(c.0)).collect();
                    format!("\"choices\":[{}]", names.join(","))
                }
                QuickOptionKind::Switch { .. } => "\"choices\":null".to_owned(),
            };
            format!(
                "{{\"label\":{},\"footer\":{},{},\"defaultValue\":{}}}",
                json_string(option.label),
                json_string(option.footer),
                kind,
                option.default_value(),
            )
        })
        .collect();
    let quick_options_elapsed = started.elapsed();
    let started = Instant::now();
    let optional = |s: Option<String>| s.map_or("null".to_owned(), |s| json_string(&s));
    let json = format!(
        "{{\"apps\":[{}],\"error\":{},\"wallpaperPath\":{},\
         \"wallpaperFolder\":{},\"wallpaperFiles\":[{}],\"appsFolder\":{},\
         \"initialPage\":{},\"initialQuickOptionValues\":{},\"branding\":{},\
         \"shortVersionText\":{},\"devVersionText\":{},\"copyright\":{},\
         \"fileManagerURL\":{},\"websiteURL\":{},\"quickOptions\":[{}],\
         \"startError\":{},\"performanceScanID\":{},\"performanceContext\":{},\
         \"rustElapsedBeforeBridgeUs\":{}}}",
        apps_json.join(","),
        optional(error),
        optional(wallpaper.map(|p| p.to_string_lossy().into_owned())),
        json_string(&paths::user_data_base_path().to_string_lossy()),
        paths::wallpaper_files()
            .iter()
            .map(|file| json_string(file))
            .collect::<Vec<_>>()
            .join(","),
        json_string(
            &paths::user_data_base_path()
                .join(paths::apps_dir())
                .to_string_lossy()
        ),
        state
            .as_ref()
            .map_or("null".to_owned(), |state| state.page.to_string()),
        state.as_ref().map_or("null".to_owned(), |state| {
            let values: Vec<String> = state
                .quick_option_values
                .iter()
                .map(usize::to_string)
                .collect();
            format!("[{}]", values.join(","))
        }),
        json_string(branding),
        json_string(crate::DISPLAY_VERSION),
        json_string("Dev 2.34"),
        json_string(&crate::licenses::get_text()),
        optional(paths::url_for_opening_user_data_dir().ok()),
        json_string("https://iDared32bit-emu.com/"),
        quick_options_json.join(","),
        optional(crate::window::START_ERROR.lock().unwrap().take()),
        performance.scan_id,
        json_string(performance.context.label()),
        performance.scan_started.elapsed().as_micros(),
    );
    // Control characters, including NUL, are escaped above.
    let json = CString::new(json).unwrap();

    let icon_ptrs: Vec<*const u8> = icons.iter().map(|icon| icon.pixels().as_ptr()).collect();
    let remaining_json_and_pointers = started.elapsed();
    let preparation_total = preparation_started.elapsed();
    echo!("=== iDared Picker Rust-to-Swift Preparation ===");
    echo!(
        "Scan #{} Context: {}",
        performance.scan_id,
        performance.context.label()
    );
    echo!(
        "Placeholder/icon references: {:.3} ms",
        placeholder_and_icon_refs.as_secs_f64() * 1000.0
    );
    echo!(
        "App JSON generation:         {:.3} ms",
        apps_json_elapsed.as_secs_f64() * 1000.0
    );
    echo!(
        "Quick-option JSON:           {:.3} ms",
        quick_options_elapsed.as_secs_f64() * 1000.0
    );
    echo!(
        "Other JSON/C string/pointers:{:>10.3} ms",
        remaining_json_and_pointers.as_secs_f64() * 1000.0
    );
    echo!(
        "Total Rust-to-Swift prep:    {:.3} ms",
        preparation_total.as_secs_f64() * 1000.0
    );
    echo!("=== End iDared Picker Rust-to-Swift Preparation ===");
    let paths: Vec<PathBuf> = apps.iter().map(|app| app.path.clone()).collect();
    let result_ptr = unsafe { picker(json.as_ptr(), icon_ptrs.as_ptr(), icon_ptrs.len()) };
    if result_ptr.is_null() {
        return Err("Couldn't show the app picker.".to_string());
    }
    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { libc::free(result_ptr.cast()) };

    parse_picker_result(&paths, result)
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
