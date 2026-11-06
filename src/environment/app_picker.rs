//! App picker GUI.
//!
//! This also includes a license text viewer. The license text viewer is needed
//! on Android, where the command-line way to view license text doesn't exist.
//!
//! On iOS, a native SwiftUI picker is used instead when the app provides one
//! (see `app_picker/swiftui.rs`).

mod persistent_index;
#[cfg(target_os = "ios")]
mod swiftui;

use crate::bundle::Bundle;
use crate::frameworks::core_graphics::cg_bitmap_context::{
    CGBitmapContextCreate, CGBitmapContextCreateImage,
};
use crate::frameworks::core_graphics::cg_color_space::CGColorSpaceCreateDeviceRGB;
use crate::frameworks::core_graphics::cg_context::{
    CGContextFillRect, CGContextRelease, CGContextScaleCTM, CGContextSetRGBFillColor,
    CGContextTranslateCTM,
};
use crate::frameworks::core_graphics::cg_image::{self, kCGImageAlphaPremultipliedLast};
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_run_loop::run_run_loop_single_iteration;
use crate::frameworks::foundation::{ns_string, NSUInteger};
use crate::frameworks::uikit::ui_font::{
    size_with_font, UITextAlignmentCenter, UITextAlignmentLeft, UITextAlignmentRight,
};
use crate::frameworks::uikit::ui_graphics::{UIGraphicsPopContext, UIGraphicsPushContext};
use crate::frameworks::uikit::ui_view::ui_control::ui_button::{
    UIButtonTypeCustom, UIButtonTypeRoundedRect,
};
use crate::frameworks::uikit::ui_view::ui_control::{
    UIControlEventTouchUpInside, UIControlEventValueChanged, UIControlStateNormal,
};
use crate::fs::BundleData;
use crate::image::Image;
use crate::mem::Ptr;
use crate::objc::{
    id, msg, msg_class, nil, objc_classes, release, AnyHostObject, ClassExports, HostObject,
    NSZonePtr,
};
use crate::options::Options;
use crate::paths;
use crate::Environment;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(target_os = "ios")]
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

static NEXT_PICKER_SCAN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
pub enum PickerInvocation {
    ColdStart,
    ReturnFromGame,
    LibraryReload,
    RetainedReturn,
}

impl PickerInvocation {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::ColdStart => "Cold Start",
            Self::ReturnFromGame => "Return From Game",
            Self::LibraryReload => "Library Reload",
            Self::RetainedReturn => "Retained Return",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PickerPerformance {
    pub(super) scan_id: u64,
    pub(super) context: PickerInvocation,
    pub(super) scan_started: Instant,
}

#[derive(Clone, Default)]
struct PackageTiming {
    name: String,
    total: Duration,
    zip_open: Duration,
    archive_validation: Duration,
    bundle_discovery: Duration,
    info_plist_read: Duration,
    info_plist_parse: Duration,
    filesystem_creation: Duration,
    localization: Duration,
    icon_candidates: Duration,
    icon_reads: Duration,
    icon_decoding: Duration,
    icon_processing: Duration,
    archive_entries: usize,
}

impl PackageTiming {
    fn measured_components(&self) -> Duration {
        self.zip_open
            + self.archive_validation
            + self.bundle_discovery
            + self.info_plist_read
            + self.info_plist_parse
            + self.filesystem_creation
            + self.localization
            + self.icon_candidates
            + self.icon_reads
            + self.icon_decoding
            + self.icon_processing
    }

    fn remainder(&self) -> Duration {
        self.total.saturating_sub(self.measured_components())
    }
}

#[derive(Default)]
struct LibraryTiming {
    loose_import: Duration,
    language_selection: Duration,
    directory_enumeration: Duration,
    sorting: Duration,
    packages_attempted: usize,
    packages_succeeded: usize,
    packages_skipped: usize,
    packages: Vec<PackageTiming>,
    persistent_index: Option<persistent_index::ReconcileStats>,
}

/// Cheap picker identity for a package in the Games directory.
///
/// IPA contents are never hashed: file metadata is sufficient for the normal
/// Files-app replacement/import paths and keeps an unchanged 16 GiB library
/// cheap to reconcile. Loose `.app` packages additionally receive a recursive
/// metadata-only digest because changing a child does not reliably update the
/// parent directory timestamp.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct LibraryEntryFingerprint {
    relative_path: String,
    package_type: String,
    file_kind: u8,
    len: u64,
    modified_secs: u64,
    modified_nanos: u32,
    device_id: u64,
    file_id: u64,
    directory_entries: u64,
    directory_bytes: u64,
    directory_metadata_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LibrarySnapshot(Vec<LibraryEntryFingerprint>);

#[cfg(target_os = "ios")]
struct LibraryCatalog {
    apps: Vec<AppInfo>,
    snapshot: LibrarySnapshot,
}

#[cfg(target_os = "ios")]
thread_local! {
    // AppInfo contains decoded image storage and guest-pointer-shaped fields,
    // so keep this native launcher cache on the same main thread that owns
    // the picker rather than claiming it is safe to move between threads.
    static RETAINED_LIBRARY: RefCell<Option<LibraryCatalog>> = const { RefCell::new(None) };
    static PURGE_RETAINED_LIBRARY: Cell<bool> = const { Cell::new(false) };
}

struct AppInfo {
    path: PathBuf,
    bundle_identifier: String,
    display_name: String,
    compatibility: String,
    icon: Option<Image>,
    /// `NSString*`
    display_name_ns_string: Option<id>,
    /// `UIImage*`
    icon_ui_image: Option<id>,
}

/// Broad picker-only compatibility grouping from the maintained device-test
/// ledger. This never changes guest behavior; it only powers the native
/// picker's optional Playable / Needs Work filter. "Playable" deliberately
/// includes titles with a useful gameplay path but a known later defect.
fn classify_picker_compatibility(bundle_identifier: &str, display_name: &str) -> &'static str {
    const PLAYABLE_IDENTIFIERS: &[&str] = &[
        "CW-Gunner",
        "com.BaneGames.FlickBuddies",
        "com.andrewgarrison.SimplePhysics",
        "com.chillingo.spoing",
        "com.clickgamer.floealittlebear",
        "com.clickgamer.sticky",
        "com.dtp-entertainment.skijumping",
        "com.ezone.Diversion",
        "com.gamevil.airpenguin",
        "com.geekbeach.minibeastshd",
        "com.mikemassei.daredevildave",
        "com.nocturnsoft.nsactionblockbuster",
        "com.raptisoft.solomonsboneyard",
        "com.sega.chuchu",
        "com.simogo.kosmospin",
        "com.spacesofplay.spiritsiphone",
    ];
    const PLAYABLE_NAMES: &[&str] = &[
        "BLOT",
        "BOUNCE",
        "BULBA THE CAT",
        "BUMPY ROAD",
        "CRAZYGOLF",
        "CRAZY GOLF",
        "CRASH DRIVE",
        "CUBE",
        "DARKNEBULA2",
        "DARK NEBULA 2",
        "DH BOWLING",
        "DIRT",
        "DODOFLY",
        "DODO FLY",
        "GLOWFISH",
        "HAMLET",
        "INFINIGHT",
        "JACK LUMBER",
        "LOCV",
        "MOS SPEEDRUN",
        "ODDY SMOG",
        "OSCURA",
        "PAPER MONSTERS",
        "PMONSTERS",
        "PIYO BLOCKS",
        "RASTAMONKEY",
        "RASTA MONKEY",
        "ROBOTO",
        "SHARPSHOOTER",
        "SILVERFISH",
        "SNAIL MAIL",
        "SOLOMON'S BONEYARD",
        "SPACE MONKEY",
        "SPACEMNKEY",
        "SPEEDRUN",
        "SPIRITS",
        "SPOING",
        "STICK SKATER",
        "STICKY",
        "SWAY",
        "TOY BOT",
        "TOYBOT",
    ];

    let name = display_name.to_uppercase();
    if PLAYABLE_IDENTIFIERS.contains(&bundle_identifier) || PLAYABLE_NAMES.contains(&name.as_str())
    {
        "playable"
    } else {
        "needsWork"
    }
}

fn picker_compatibility(app: &AppInfo) -> &str {
    &app.compatibility
}

fn modified_parts(metadata: &Metadata, path: &Path) -> Result<(u64, u32), String> {
    let modified = metadata
        .modified()
        .map_err(|error| format!("couldn't read mtime for {}: {error}", path.display()))?;
    let duration = modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| format!("invalid mtime for {}: {error}", path.display()))?;
    Ok((duration.as_secs(), duration.subsec_nanos()))
}

#[cfg(unix)]
fn file_identity(metadata: &Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}

#[cfg(not(unix))]
fn file_identity(_metadata: &Metadata) -> (u64, u64) {
    (0, 0)
}

fn directory_metadata_fingerprint(directory: &Path) -> Result<(u64, u64, String), String> {
    let mut pending = vec![directory.to_path_buf()];
    let mut count = 0u64;
    let mut bytes = 0u64;
    let mut digest = Sha256::new();

    while let Some(current) = pending.pop() {
        let entries = std::fs::read_dir(&current)
            .map_err(|error| format!("couldn't read {}: {error}", current.display()))?;
        let mut entries = entries
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("couldn't read an entry in {}: {error}", current.display()))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in entries {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|error| format!("couldn't inspect {}: {error}", path.display()))?;
            let relative = path
                .strip_prefix(directory)
                .map_err(|error| format!("couldn't normalize {}: {error}", path.display()))?;
            let (modified_secs, modified_nanos) = modified_parts(&metadata, &path)?;
            let (device_id, file_id) = file_identity(&metadata);
            let kind = if metadata.is_file() {
                1u8
            } else if metadata.is_dir() {
                2u8
            } else {
                3u8
            };

            count = count.saturating_add(1);
            if metadata.is_file() {
                bytes = bytes.saturating_add(metadata.len());
            }
            digest.update(relative.to_string_lossy().as_bytes());
            digest.update([0]);
            digest.update([kind]);
            digest.update(metadata.len().to_le_bytes());
            digest.update(modified_secs.to_le_bytes());
            digest.update(modified_nanos.to_le_bytes());
            digest.update(device_id.to_le_bytes());
            digest.update(file_id.to_le_bytes());

            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }

    Ok((count, bytes, format!("{:x}", digest.finalize())))
}

fn fingerprint_entry(path: &Path, prefix: &str) -> Result<LibraryEntryFingerprint, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("couldn't inspect {}: {error}", path.display()))?;
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| format!("package path is not valid UTF-8: {}", path.display()))?;
    let (modified_secs, modified_nanos) = modified_parts(&metadata, path)?;
    let (device_id, file_id) = file_identity(&metadata);
    let file_kind = if metadata.is_file() {
        1
    } else if metadata.is_dir() {
        2
    } else {
        3
    };
    let (directory_entries, directory_bytes, directory_metadata_sha256) = if metadata.is_dir() {
        directory_metadata_fingerprint(path)?
    } else {
        (0, 0, String::new())
    };

    Ok(LibraryEntryFingerprint {
        relative_path: format!("{prefix}/{name}"),
        package_type: path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase(),
        file_kind,
        len: metadata.len(),
        modified_secs,
        modified_nanos,
        device_id,
        file_id,
        directory_entries,
        directory_bytes,
        directory_metadata_sha256,
    })
}

fn snapshot_entries(
    directory: &Path,
    prefix: &str,
    include: impl Fn(&Path) -> bool,
    output: &mut Vec<LibraryEntryFingerprint>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("couldn't read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("couldn't read an entry in {}: {error}", directory.display())
        })?;
        let path = entry.path();
        if !include(&path) {
            continue;
        }
        output.push(fingerprint_entry(&path, prefix)?);
    }
    Ok(())
}

fn capture_library_snapshot() -> Result<LibrarySnapshot, String> {
    let base = paths::user_data_base_path();
    let games = base.join(paths::apps_dir());
    let mut entries = Vec::new();
    snapshot_entries(
        &games,
        "Games",
        |path| {
            path.extension().is_some_and(|extension| {
                extension.eq_ignore_ascii_case("ipa") || extension.eq_ignore_ascii_case("app")
            })
        },
        &mut entries,
    )?;
    // iOS can receive IPAs in the main Documents directory. The normal full
    // path moves them into Games; include them so a retained picker never
    // hides a newly shared package.
    snapshot_entries(
        &base,
        "Loose",
        |path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("ipa"))
        },
        &mut entries,
    )?;
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(LibrarySnapshot(entries))
}

#[cfg(target_os = "ios")]
fn retained_icon_bytes(apps: &[AppInfo]) -> usize {
    apps.iter()
        .filter_map(|app| app.icon.as_ref())
        .map(|icon| icon.pixels().len())
        .sum()
}

#[cfg(target_os = "ios")]
fn store_catalog_after_picker(catalog: LibraryCatalog) {
    let icon_bytes = retained_icon_bytes(&catalog.apps);
    if PURGE_RETAINED_LIBRARY.with(|purge| purge.replace(false)) {
        echo!(
            "iDared retained library catalog discarded after a memory warning ({} apps, {:.2} MiB decoded icon pixels).",
            catalog.apps.len(),
            icon_bytes as f64 / (1024.0 * 1024.0)
        );
        return;
    }
    echo!(
        "iDared retained library catalog ready: {} apps, {:.2} MiB decoded icon pixels; no guest filesystem state retained.",
        catalog.apps.len(),
        icon_bytes as f64 / (1024.0 * 1024.0)
    );
    RETAINED_LIBRARY.with(|retained| *retained.borrow_mut() = Some(catalog));
}

/// Called by the native iOS application on a memory warning. The native
/// picker hierarchy is already released after selection; this releases the
/// process-lifetime metadata/icon catalog as well. The next return safely uses
/// the existing full scan.
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn idared_purge_retained_picker_catalog() {
    RETAINED_LIBRARY.with(|retained| {
        if let Some(catalog) = retained.borrow_mut().take() {
            PURGE_RETAINED_LIBRARY.with(|purge| purge.set(false));
            echo_no_panic!(
                "iDared retained library catalog purged for memory pressure ({} apps, {:.2} MiB decoded icon pixels).",
                catalog.apps.len(),
                retained_icon_bytes(&catalog.apps) as f64 / (1024.0 * 1024.0)
            );
        } else {
            // The catalog may currently be local to the native picker call.
            // Prevent it from being retained after the user selects a game.
            PURGE_RETAINED_LIBRARY.with(|purge| purge.set(true));
        }
    });
}

pub fn app_picker(
    options: Options,
    invocation: PickerInvocation,
) -> Result<(PathBuf, Vec<String>), String> {
    #[cfg(target_os = "ios")]
    if swiftui::is_available() {
        let return_reason = if matches!(invocation, PickerInvocation::ReturnFromGame) {
            crate::window::take_picker_return_reason()
        } else {
            crate::window::PickerReturnReason::Other
        };
        let mut state = None;
        let mut scan_invocation = invocation;
        let mut retained_catalog = None;
        let mut retained_return_started = None;

        if matches!(invocation, PickerInvocation::ReturnFromGame) {
            let check_started = Instant::now();
            // Take ownership rather than holding a RefCell borrow while Swift
            // runs a nested main-thread run loop. A memory warning can then
            // request that this local catalog not be retained after selection.
            let cached = RETAINED_LIBRARY.with(|retained| retained.borrow_mut().take());
            let current = capture_library_snapshot();
            let check_elapsed = check_started.elapsed();
            echo!("=== iDared Picker Fast Return ===");
            echo!("Return Type: {}", return_reason.label());
            echo!("Library cheap-check: {:.3} ms", duration_ms(check_elapsed));
            match (cached, current) {
                (Some(cached), Ok(current)) if cached.snapshot == current => {
                    echo!("Picker Return Mode: retained-catalog");
                    echo!(
                        "Retained apps: {}; decoded icon pixels: {:.2} MiB",
                        cached.apps.len(),
                        retained_icon_bytes(&cached.apps) as f64 / (1024.0 * 1024.0)
                    );
                    retained_catalog = Some(cached);
                    retained_return_started = Some(check_started);
                }
                (Some(_), Ok(_)) => {
                    echo!("Picker Return Mode: full-rescan");
                    echo!("Fallback reason: library-changed");
                }
                (None, _) => {
                    echo!("Picker Return Mode: full-rescan");
                    echo!("Fallback reason: no-retained-library");
                }
                (Some(_), Err(error)) => {
                    echo!("Picker Return Mode: full-rescan");
                    echo!("Fallback reason: snapshot-error ({error})");
                }
            }
            echo!("=== End iDared Picker Fast Return ===");
        }

        // The picker asks to be shown again when the Games folder changes,
        // e.g. after a game is added in the Files app.
        loop {
            if let Some(catalog) = retained_catalog.take() {
                let performance = PickerPerformance {
                    scan_id: NEXT_PICKER_SCAN_ID.fetch_add(1, Ordering::Relaxed),
                    context: PickerInvocation::RetainedReturn,
                    scan_started: retained_return_started.unwrap_or_else(Instant::now),
                };
                match swiftui::app_picker(Ok(&catalog.apps), state, performance)? {
                    swiftui::Picked::App(path, args) => {
                        store_catalog_after_picker(catalog);
                        return Ok((path, args));
                    }
                    swiftui::Picked::Reload(new_state) => {
                        echo!(
                            "Retained picker requested a library reload; discarding its catalog."
                        );
                        state = Some(new_state);
                        scan_invocation = PickerInvocation::LibraryReload;
                    }
                }
                continue;
            }

            let (apps, performance) = find_apps(&options, scan_invocation);
            match apps {
                Err(error) => match swiftui::app_picker(Err(&error), state, performance)? {
                    swiftui::Picked::App(path, args) => return Ok((path, args)),
                    swiftui::Picked::Reload(new_state) => {
                        state = Some(new_state);
                        scan_invocation = PickerInvocation::LibraryReload;
                    }
                },
                Ok(apps) => {
                    // Capture after the scan and before presentation. The
                    // Swift picker also watches the directory while visible;
                    // a change there asks this loop to perform a full reload.
                    let snapshot = capture_library_snapshot().map_err(|error| {
                        echo!(
                            "Warning: picker catalog cannot be retained because its library snapshot failed: {error}"
                        );
                        error
                    });
                    match swiftui::app_picker(Ok(&apps), state, performance)? {
                        swiftui::Picked::App(path, args) => {
                            if let Ok(snapshot) = snapshot {
                                store_catalog_after_picker(LibraryCatalog { apps, snapshot });
                            }
                            return Ok((path, args));
                        }
                        swiftui::Picked::Reload(new_state) => {
                            state = Some(new_state);
                            scan_invocation = PickerInvocation::LibraryReload;
                        }
                    }
                }
            }
        }
    }

    let (apps, _performance) = find_apps(&options, invocation);
    show_app_picker_gui(options, apps)
}

fn find_apps(
    options: &Options,
    invocation: PickerInvocation,
) -> (Result<Vec<AppInfo>, String>, PickerPerformance) {
    let performance = PickerPerformance {
        scan_id: NEXT_PICKER_SCAN_ID.fetch_add(1, Ordering::Relaxed),
        context: invocation,
        scan_started: Instant::now(),
    };
    let mut timing = LibraryTiming::default();
    let apps_dir = paths::user_data_base_path().join(paths::apps_dir());

    let result = if !apps_dir.is_dir() {
        Err(format!(
            "The {} folder couldn't be found.",
            paths::apps_dir()
        ))
    } else {
        #[cfg(any(target_os = "ios", target_os = "android"))]
        {
            let started = Instant::now();
            move_loose_ipas(&apps_dir);
            timing.loose_import = started.elapsed();
        }
        let started = Instant::now();
        let languages = options.preferred_languages.clone().unwrap_or_else(|| {
            sdl2::locale::get_preferred_locales()
                .map(|loc| loc.lang)
                .collect()
        });
        timing.language_selection = started.elapsed();
        // Only the iOS picker shows bigger icons at their full size.
        #[cfg(target_os = "ios")]
        let best_icons = swiftui::is_available();
        #[cfg(not(target_os = "ios"))]
        let best_icons = false;
        enumerate_apps(&apps_dir, &languages, best_icons, &mut timing)
            .map_err(|err| {
                format!(
                    "Couldn't get list of apps in the {} directory: {}.",
                    paths::apps_dir(),
                    err
                )
            })
            .and_then(|apps| {
                if apps.is_empty() {
                    Err(format!(
                        "No games were found in the {} folder.",
                        paths::apps_dir()
                    ))
                } else {
                    Ok(apps)
                }
            })
    };
    log_library_performance(performance, &timing);
    (result, performance)
}

/// Moves .ipa files left in the main folder, e.g. by the Files app, into the
/// Games folder, where they're looked for.
#[cfg(any(target_os = "ios", target_os = "android"))]
fn move_loose_ipas(apps_dir: &Path) {
    move_loose_ipas_from(&paths::user_data_base_path(), apps_dir);
}

fn move_loose_ipas_from(source_directory: &Path, apps_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(source_directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_ipa = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ipa"));
        if !is_ipa || !path.is_file() {
            continue;
        }
        let dest = apps_dir.join(entry.file_name());
        if dest.exists() {
            log!(
                "Not moving {} into {}, which already has a file of that name.",
                path.display(),
                paths::apps_dir(),
            );
            continue;
        }
        match std::fs::rename(&path, &dest) {
            Ok(()) => {
                log!("Moved {} into {}.", path.display(), paths::apps_dir());
            }
            Err(e) => {
                log!("Warning: Couldn't move {}: {}", path.display(), e);
            }
        }
    }
}

fn enumerate_apps(
    apps_dir: &Path,
    languages: &[String],
    best_icons: bool,
    timing: &mut LibraryTiming,
) -> Result<Vec<AppInfo>, std::io::Error> {
    #[cfg(target_os = "ios")]
    if best_icons {
        return enumerate_apps_persistent(apps_dir, languages, timing);
    }

    enumerate_apps_uncached(apps_dir, languages, best_icons, timing)
}

fn inspect_app_package(
    app_path: &Path,
    languages: &[String],
    best_icons: bool,
) -> (persistent_index::InspectionResult, PackageTiming) {
    let package_started = Instant::now();
    let mut package = PackageTiming {
        name: app_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| app_path.display().to_string()),
        ..PackageTiming::default()
    };

    let reject = |error: String, mut package: PackageTiming| {
        package.total = package_started.elapsed();
        log!(
            "Warning: couldn't open app bundle {}: {} (skipping)",
            app_path.display(),
            error
        );
        (persistent_index::InspectionResult::Rejected(error), package)
    };

    // TODO: avoid loading the whole FS somehow? The persistent index makes
    // this cost package-incremental instead of paying it for every unchanged
    // package on every process launch.
    let (bundle_data, open_timings) = match BundleData::open_any_timed(app_path) {
        Ok(ok) => ok,
        Err(error) => return reject(error.to_string(), package),
    };
    package.zip_open = open_timings.zip_open;
    package.archive_validation = open_timings.archive_validation;
    package.bundle_discovery = open_timings.bundle_discovery;
    package.archive_entries = open_timings.archive_entries;
    let (bundle, fs, construction_timings) = match Bundle::new_bundle_and_fs_from_host_path_timed(
        bundle_data,
        /* read_only_mode: */ true,
    ) {
        Ok(ok) => ok,
        Err(error) => return reject(error.to_string(), package),
    };
    package.info_plist_read = construction_timings.info_plist_read;
    package.info_plist_parse = construction_timings.info_plist_parse;
    package.filesystem_creation = construction_timings.filesystem_creation;

    let started = Instant::now();
    let display_name = bundle.localized_display_name(&fs, languages);
    package.localization = started.elapsed();

    let icon = if best_icons {
        bundle.load_best_icon_timed(&fs)
    } else {
        bundle.load_icon_timed(&fs)
    };
    let icon = match icon {
        Ok((icon, icon_timings)) => {
            package.icon_candidates = icon_timings.candidate_discovery;
            package.icon_reads = icon_timings.reads;
            package.icon_decoding = icon_timings.decoding;
            package.icon_processing = icon_timings.processing;
            Some(icon)
        }
        Err(error) => {
            log!(
                "Warning: couldn't load icon for app bundle {}: {} (displaying placeholder instead)",
                app_path.display(),
                error
            );
            None
        }
    };

    let bundle_identifier = bundle.bundle_identifier().to_owned();
    let compatibility = classify_picker_compatibility(&bundle_identifier, &display_name).to_owned();
    let canonical_name = bundle.canonical_bundle_name_optional().map(str::to_owned);
    let bundle_version = bundle.bundle_version_optional().map(str::to_owned);
    let minimum_os_version = bundle.minimum_os_version_optional().map(str::to_owned);
    let app = AppInfo {
        path: app_path.to_owned(),
        bundle_identifier,
        display_name,
        compatibility,
        icon,
        display_name_ns_string: None,
        icon_ui_image: None,
    };
    package.total = package_started.elapsed();
    (
        persistent_index::InspectionResult::App(persistent_index::InspectedApp {
            app,
            canonical_name,
            bundle_version,
            minimum_os_version,
        }),
        package,
    )
}

fn sort_picker_apps(apps: &mut [AppInfo], timing: &mut LibraryTiming) {
    // Put the current focused device-test batch first, in alphabetical order,
    // so a full round can be exercised without repeatedly paging through the
    // library. Everything else remains alphabetical.
    // Normalized display names, bundle identifiers and archive filenames cover
    // the mixed naming in this batch and keep every requested test reachable
    // even when an archive's display-name metadata differs from its filename.
    const DEV_233_TEST_NAMES: &[&str] = &[
        "1112EP01",
        "BOUNCYMOUSE",
        "BUNGEEBALL",
        "CRAZYGOLF",
        "DOODLEGRUB",
        "FLIPSESCAPE",
        "HIGHWAYRIDER",
        "ISTUNTRELOADEDPRO",
        "COMWALABERJELLYCAR",
        "JUNKJACKRETRO",
        "KICKTHEBUDDY",
        "MXMAYHEM",
        "RRACINGGTI",
        "REALRACING",
        "SUPERFALL",
        "TIKITOTEMSPREMIUM",
        "TOWERBLOXX",
    ];
    let started = Instant::now();
    apps.sort_by_key(|app| {
        let normalized = |text: &str| -> String {
            text.to_uppercase()
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .collect()
        };
        let display_name = normalized(&app.display_name);
        let bundle_identifier = normalized(&app.bundle_identifier);
        let filename = normalized(
            app.path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        );
        let rank = DEV_233_TEST_NAMES
            .iter()
            .position(|&name| {
                [&display_name, &bundle_identifier, &filename]
                    .into_iter()
                    .any(|identity| identity == name || identity.starts_with(name))
            })
            .unwrap_or(usize::MAX);
        (rank, app.display_name.to_uppercase())
    });
    timing.sorting = started.elapsed();
}

fn enumerate_apps_persistent(
    apps_dir: &Path,
    languages: &[String],
    timing: &mut LibraryTiming,
) -> Result<Vec<AppInfo>, std::io::Error> {
    let started = Instant::now();
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(apps_dir)? {
        let path = entry?.path();
        if path.extension() != Some(OsStr::new("app"))
            && path.extension() != Some(OsStr::new("ipa"))
        {
            continue;
        }
        let relative_path = path
            .file_name()
            .map(|name| format!("{}/{}", paths::apps_dir(), name.to_string_lossy()))
            .unwrap_or_else(|| path.display().to_string());
        let fingerprint = match fingerprint_entry(&path, paths::apps_dir()) {
            Ok(fingerprint) => Some(fingerprint),
            Err(error) => {
                log!(
                    "Warning: {} cannot be persistently identified; inspecting it for this launch: {}",
                    path.display(),
                    error
                );
                None
            }
        };
        sources.push(persistent_index::SourcePackage {
            path,
            relative_path,
            fingerprint,
        });
    }
    sources.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    timing.directory_enumeration += started.elapsed();
    timing.packages_attempted = sources.len();

    let result = persistent_index::reconcile(
        &persistent_index::default_cache_root(),
        sources,
        languages,
        |path| {
            let (result, package) = inspect_app_package(path, languages, true);
            timing.packages.push(package);
            result
        },
    );
    timing.packages_succeeded = result.apps.len();
    timing.packages_skipped = timing
        .packages_attempted
        .saturating_sub(timing.packages_succeeded);
    timing.persistent_index = Some(result.stats);
    let mut apps = result.apps;
    sort_picker_apps(&mut apps, timing);
    Ok(apps)
}

fn enumerate_apps_uncached(
    apps_dir: &Path,
    languages: &[String],
    best_icons: bool,
    timing: &mut LibraryTiming,
) -> Result<Vec<AppInfo>, std::io::Error> {
    let mut apps = Vec::new();
    let started = Instant::now();
    let mut entries = std::fs::read_dir(apps_dir)?;
    timing.directory_enumeration += started.elapsed();
    loop {
        let started = Instant::now();
        let entry = entries.next();
        timing.directory_enumeration += started.elapsed();
        let Some(app) = entry else {
            break;
        };
        let app_path = app?.path();
        if app_path.extension() != Some(OsStr::new("app"))
            && app_path.extension() != Some(OsStr::new("ipa"))
        {
            continue;
        }

        timing.packages_attempted += 1;
        let (result, package) = inspect_app_package(&app_path, languages, best_icons);
        timing.packages.push(package);
        match result {
            persistent_index::InspectionResult::App(inspected) => {
                timing.packages_succeeded += 1;
                apps.push(inspected.app);
            }
            persistent_index::InspectionResult::Rejected(_) => {
                timing.packages_skipped += 1;
            }
        }
    }

    sort_picker_apps(&mut apps, timing);

    Ok(apps)
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn percentile(sorted: &[Duration], percentile: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let index = (sorted.len() * percentile).div_ceil(100).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn log_library_performance(performance: PickerPerformance, timing: &LibraryTiming) {
    let total = performance.scan_started.elapsed();
    let sum = |f: fn(&PackageTiming) -> Duration| timing.packages.iter().map(f).sum();
    let zip_open = sum(|p| p.zip_open);
    let archive_validation = sum(|p| p.archive_validation);
    let bundle_discovery = sum(|p| p.bundle_discovery);
    let info_plist_read = sum(|p| p.info_plist_read);
    let info_plist_parse = sum(|p| p.info_plist_parse);
    let filesystem_creation = sum(|p| p.filesystem_creation);
    let localization = sum(|p| p.localization);
    let icon_candidates = sum(|p| p.icon_candidates);
    let icon_reads = sum(|p| p.icon_reads);
    let icon_decoding = sum(|p| p.icon_decoding);
    let icon_processing = sum(|p| p.icon_processing);
    let package_remainder: Duration = timing.packages.iter().map(PackageTiming::remainder).sum();
    let measured = timing.loose_import
        + timing.language_selection
        + timing.directory_enumeration
        + zip_open
        + archive_validation
        + bundle_discovery
        + info_plist_read
        + info_plist_parse
        + filesystem_creation
        + localization
        + icon_candidates
        + icon_reads
        + icon_decoding
        + icon_processing
        + package_remainder
        + timing.sorting;
    let outside_packages = total.saturating_sub(measured);
    let archive_entries: usize = timing.packages.iter().map(|p| p.archive_entries).sum();

    let mut totals: Vec<Duration> = timing.packages.iter().map(|p| p.total).collect();
    totals.sort_unstable();
    let median = percentile(&totals, 50);
    let p95 = percentile(&totals, 95);
    let mut slowest = timing.packages.clone();
    slowest.sort_by_key(|p| std::cmp::Reverse(p.total));

    echo!("=== iDared Picker Performance ===");
    echo!(
        "Scan #{} Context: {}",
        performance.scan_id,
        performance.context.label()
    );
    echo!(
        "Rust scan started near process t={:.3} ms",
        crate::performance_process_elapsed_ms() - total.as_secs_f64() * 1000.0
    );
    echo!(
        "Packages: {} attempted, {} loaded, {} skipped; {} archive entries",
        timing.packages_attempted,
        timing.packages_succeeded,
        timing.packages_skipped,
        archive_entries
    );
    if let Some(index) = timing.persistent_index.as_ref() {
        echo!("--- Persistent Picker Index ---");
        echo!(
            "Schema: {}; existed: {}; state: {}",
            persistent_index::INDEX_SCHEMA_VERSION,
            index.index_existed,
            index.index_state
        );
        echo!(
            "Cache results: {} successful hits, {} negative hits, {} new, {} changed, {} uncacheable, {} missing/corrupt icons",
            index.successful_hits,
            index.negative_hits,
            index.new_misses,
            index.changed_misses,
            index.uncacheable_misses,
            index.icon_misses,
        );
        echo!(
            "Focused inspection: {} succeeded, {} rejected; {} deleted; {} records persisted",
            index.inspected_successes,
            index.inspected_failures,
            index.deleted_entries,
            index.persisted_entries,
        );
        echo!(
            "Index read/decode:          {:.3} / {:.3} ms",
            duration_ms(index.index_read),
            duration_ms(index.index_decode)
        );
        echo!(
            "Incremental reconciliation:{:>10.3} ms",
            duration_ms(index.reconciliation)
        );
        echo!(
            "Icon cache work:            {:.3} ms ({} stale removed)",
            duration_ms(index.icon_cache),
            index.icons_removed
        );
        echo!(
            "Index write/cleanup:        {:.3} ms",
            duration_ms(index.index_write)
        );
        if index.index_state != "loaded" {
            echo!("Full/focused rescan reason: {}", index.index_state);
        }
        if let Some(error) = index.write_error.as_deref() {
            echo!(
                "Warning: persistent index was usable for this launch but could not be saved: {}",
                error
            );
        }
        echo!("--- End Persistent Picker Index ---");
    }
    echo!(
        "Loose IPA/import check:       {:.3} ms",
        duration_ms(timing.loose_import)
    );
    echo!(
        "Language selection:           {:.3} ms",
        duration_ms(timing.language_selection)
    );
    echo!(
        "Directory enumeration:        {:.3} ms",
        duration_ms(timing.directory_enumeration)
    );
    echo!(
        "ZIP open/central directory:   {:.3} ms",
        duration_ms(zip_open)
    );
    echo!(
        "Archive entry validation:     {:.3} ms",
        duration_ms(archive_validation)
    );
    echo!(
        "Application bundle discovery:{:>10.3} ms",
        duration_ms(bundle_discovery)
    );
    echo!(
        "Info.plist reading:           {:.3} ms",
        duration_ms(info_plist_read)
    );
    echo!(
        "Info.plist parsing:           {:.3} ms",
        duration_ms(info_plist_parse)
    );
    echo!(
        "Guest filesystem creation:    {:.3} ms",
        duration_ms(filesystem_creation)
    );
    echo!(
        "Localization:                 {:.3} ms",
        duration_ms(localization)
    );
    echo!(
        "Icon candidate discovery:     {:.3} ms",
        duration_ms(icon_candidates)
    );
    echo!(
        "Icon reads/decompression:     {:.3} ms",
        duration_ms(icon_reads)
    );
    echo!(
        "Icon decoding:                {:.3} ms",
        duration_ms(icon_decoding)
    );
    echo!(
        "Icon processing:              {:.3} ms",
        duration_ms(icon_processing)
    );
    echo!(
        "Per-package remainder:        {:.3} ms",
        duration_ms(package_remainder)
    );
    echo!(
        "Sorting/classification:       {:.3} ms",
        duration_ms(timing.sorting)
    );
    echo!(
        "Unattributed scan overhead:   {:.3} ms",
        duration_ms(outside_packages)
    );
    echo!("Total Rust library scan:      {:.3} ms", duration_ms(total));
    echo!(
        "Package timing median/p95:   {:.3} / {:.3} ms",
        duration_ms(median),
        duration_ms(p95)
    );
    echo!("Slowest packages (bounded to five):");
    for (index, package) in slowest.iter().take(5).enumerate() {
        echo!(
            "{}. {} — {:.3} ms (entries {}, zip {:.3}, validate {:.3}, fs {:.3}, metadata {:.3}, icons {:.3}, other {:.3})",
            index + 1,
            package.name,
            duration_ms(package.total),
            package.archive_entries,
            duration_ms(package.zip_open),
            duration_ms(package.archive_validation),
            duration_ms(package.filesystem_creation),
            duration_ms(package.info_plist_read + package.info_plist_parse + package.localization),
            duration_ms(package.icon_candidates + package.icon_reads + package.icon_decoding + package.icon_processing),
            duration_ms(package.remainder()),
        );
    }
    echo!("=== End iDared Picker Performance ===");
}

#[cfg(test)]
mod picker_performance_tests {
    use super::percentile;
    use std::time::Duration;

    #[test]
    fn percentile_is_bounded_and_uses_nearest_rank() {
        let samples: Vec<Duration> = (1..=20).map(Duration::from_millis).collect();
        assert_eq!(percentile(&samples, 50), Duration::from_millis(10));
        assert_eq!(percentile(&samples, 95), Duration::from_millis(19));
        assert_eq!(percentile(&samples, 100), Duration::from_millis(20));
        assert_eq!(percentile(&[], 95), Duration::ZERO);
    }
}

#[derive(Default)]
struct AppPickerDelegateHostObject {
    grid_tapped_point: Option<CGPoint>,
    requested_page: Option<usize>,
    prev_page: bool,
    next_page: bool,
    copyright_show: bool,
    copyright_hide: bool,
    copyright_prev: bool,
    copyright_next: bool,
    quick_options_show: bool,
    quick_options_hide: bool,
    /// The quick option button (`UIButton*`) that was tapped.
    quick_option_choice: Option<id>,
    /// The quick option switch (`UISwitch*`) that was changed.
    quick_option_switch: Option<id>,
}
impl HostObject for AppPickerDelegateHostObject {}

struct AppPickerGridViewHostObject {
    superclass: crate::frameworks::uikit::ui_view::UIViewHostObject,
    delegate: id,
    start_point: Option<CGPoint>,
    icon_buttons: Vec<id>,
    highlighted_button: id,
}

impl HostObject for AppPickerGridViewHostObject {
    fn as_superclass<'a>(&'a self) -> Option<&'a (dyn AnyHostObject + 'static)> {
        Some(&self.superclass)
    }
    fn as_superclass_mut<'a>(&'a mut self) -> Option<&'a mut (dyn AnyHostObject + 'static)> {
        Some(&mut self.superclass)
    }
}
impl Default for AppPickerGridViewHostObject {
    fn default() -> Self {
        Self {
            superclass: crate::frameworks::uikit::ui_view::UIViewHostObject::default(),
            delegate: nil,
            start_point: None,
            icon_buttons: Vec::new(),
            highlighted_button: nil,
        }
    }
}

struct AppPickerPageControlHostObject {
    superclass: crate::frameworks::uikit::ui_view::UIViewHostObject,
    delegate: id,
    number_of_pages: usize,
    current_page: usize,
}
impl Default for AppPickerPageControlHostObject {
    fn default() -> Self {
        Self {
            superclass: crate::frameworks::uikit::ui_view::UIViewHostObject::default(),
            delegate: nil,
            number_of_pages: 0,
            current_page: 0,
        }
    }
}
impl HostObject for AppPickerPageControlHostObject {
    fn as_superclass<'a>(&'a self) -> Option<&'a (dyn AnyHostObject + 'static)> {
        Some(&self.superclass)
    }
    fn as_superclass_mut<'a>(&'a mut self) -> Option<&'a mut (dyn AnyHostObject + 'static)> {
        Some(&mut self.superclass)
    }
}

pub const DYLIB: crate::dyld::HostDylib = crate::dyld::HostDylib {
    // Not a real iOS dylib obviously. This shouldn't really be in the list of
    // dylibs if we can avoid it somehow (TODO?).
    path: "/.touchHLE/AppPickerHelpers.dylib",
    aliases: &[],
    class_exports: &[CLASSES],
    constant_exports: &[],
    function_exports: &[],
};

/// Be careful! These classes go in the normal class list, just like everything
/// else, so an app could try to instantiate them. Don't give them special
/// powers that could be exploited!
const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation _touchHLE_AppPickerDelegate: NSObject

- (())gridTappedAt:(CGPoint)point {
    let host_obj = env.objc.borrow_mut::<AppPickerDelegateHostObject>(this);
    host_obj.grid_tapped_point = Some(point);
}

- (())pageSelected:(NSUInteger)page {
    let host_obj = env.objc.borrow_mut::<AppPickerDelegateHostObject>(this);
    host_obj.requested_page = Some(page as usize);
}

- (())prevPage {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).prev_page = true;
}

- (())nextPage {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).next_page = true;
}

- (())copyrightInfoShow {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).copyright_show = true;
}
- (())copyrightInfoHide {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).copyright_hide = true;
}
- (())copyrightInfoPrevPage {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).copyright_prev = true;
}
- (())copyrightInfoNextPage {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).copyright_next = true;
}

- (())quickOptionsShow {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).quick_options_show = true;
}
- (())quickOptionsHide {
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).quick_options_hide = true;
}
- (())quickOptionChoice:(id)button { // UIButton*
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).quick_option_choice = Some(button);
}
- (())quickOptionSwitch:(id)switch { // UISwitch*
    env.objc.borrow_mut::<AppPickerDelegateHostObject>(this).quick_option_switch = Some(switch);
}

- (())openFileManager {
    // Assert (see above).
    let _ = env.objc.borrow_mut::<AppPickerDelegateHostObject>(this);

    match paths::url_for_opening_user_data_dir() {
        Ok(url) => {
            // Our `openURL:` implementation is bypassed because it doesn't
            // allow non-web URLs.
            let url_res = crate::window::open_url(env, &url);
            if let Err(e) = url_res {
                echo!("Couldn't open file manager at {:?}: {}", url, e);
            } else {
                echo!("Opened file manager at {:?}.", url);
                if std::env::consts::OS != "ios" {
                    echo!("Exiting.");
                    std::process::exit(0);
                }
            }
        },
        Err(e) => echo!("Couldn't open file manager: {}", e),
    }
}

- (())visitWebsite {
    // Assert (see above).
    let _ = env.objc.borrow_mut::<AppPickerDelegateHostObject>(this);

    if let Err(e) = crate::window::open_url(env, "https://iDared32bit-emu.com/") {
        echo!("Couldn't open iDared32bit-emu.com: {}", e);
    }
}

@end

@implementation _touchHLE_AppPickerPageControl: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<AppPickerPageControlHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setNumberOfPages:(NSUInteger)n {
    env.objc.borrow_mut::<AppPickerPageControlHostObject>(this).number_of_pages = n as usize;
    // Like UIPageControl's hidesForSinglePage: one dot says nothing.
    () = msg![env; this setHidden:(n <= 1)];
    () = msg![env; this setNeedsDisplay];
}
- (())setCurrentPage:(NSUInteger)n {
    env.objc.borrow_mut::<AppPickerPageControlHostObject>(this).current_page = n as usize;
    () = msg![env; this setNeedsDisplay];
}

- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<AppPickerPageControlHostObject>(this).delegate = delegate;
}

- (())touchesEnded:(id)touches withEvent:(id)_event {
    let touch: id = msg![env; touches anyObject];
    let point: CGPoint = msg![env; touch locationInView:this];
    let bounds: CGRect = msg![env; this bounds];
    let (number_of_pages, current_page, delegate) = {
        let host_obj = env.objc.borrow::<AppPickerPageControlHostObject>(this);
        (host_obj.number_of_pages, host_obj.current_page, host_obj.delegate)
    };

    if number_of_pages == 0 || delegate == nil { return; }

    let dot_spacing: CGFloat = 15.0;
    let total_width = (number_of_pages as f32 - 1.0) * dot_spacing;
    let start_x = bounds.size.width / 2.0 - total_width / 2.0;

    // A tap on a dot goes to its page. The dots are too small to hit
    // reliably, so like UIPageControl, a tap anywhere else goes one page
    // back or forward, depending on which side of the current dot it's on.
    let nearest = ((point.x - start_x) / dot_spacing).round();
    if nearest >= 0.0 && (nearest as usize) < number_of_pages
        && (point.x - (start_x + nearest * dot_spacing)).abs() <= dot_spacing / 2.0 {
        () = msg![env; delegate pageSelected:(nearest as NSUInteger)];
    } else if point.x < start_x + (current_page as f32) * dot_spacing {
        () = msg![env; delegate prevPage];
    } else {
        () = msg![env; delegate nextPage];
    }
}

- (())drawRect:(CGRect)_rect {
    use crate::frameworks::core_graphics::cg_context::{CGContextFillRect, CGContextSetRGBFillColor};
    let context = crate::frameworks::uikit::ui_graphics::UIGraphicsGetCurrentContext(env);
    let bounds: CGRect = msg![env; this bounds];
    let &AppPickerPageControlHostObject { number_of_pages, current_page, .. } = env.objc.borrow(this);

    if number_of_pages == 0 { return; }

    let dot_spacing = 15.0;
    let total_width = (number_of_pages as f32 - 1.0) * dot_spacing;
    let start_x = bounds.size.width / 2.0 - total_width / 2.0;
    let y = (bounds.size.height / 2.0).round();

    for i in 0..number_of_pages {
        let is_current = i == current_page;
        let size = if is_current { 7.0 } else { 5.0 };
        let alpha = if is_current { 1.0 } else { 0.5 };
        CGContextSetRGBFillColor(env, context, 1.0, 1.0, 1.0, alpha);

        let x = (start_x + (i as f32) * dot_spacing).round();
        let dot_rects = [
            CGRect { origin: CGPoint { x: x - size/2.0 + 1.0, y: y - size/2.0 }, size: CGSize { width: size - 2.0, height: size } },
            CGRect { origin: CGPoint { x: x - size/2.0, y: y - size/2.0 + 1.0 }, size: CGSize { width: size, height: size - 2.0 } },
        ];
        for r in dot_rects {
            CGContextFillRect(env, context, r);
        }
    }
}

@end

@implementation _touchHLE_AppPickerGridView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<AppPickerGridViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())touchesBegan:(id)touches withEvent:(id)_event {
    let touch: id = msg![env; touches anyObject];
    let point: CGPoint = msg![env; touch locationInView:this];

    let (buttons, previous) = {
        let host_obj = env.objc.borrow_mut::<AppPickerGridViewHostObject>(this);
        host_obj.start_point = Some(point);
        (host_obj.icon_buttons.clone(), std::mem::take(&mut host_obj.highlighted_button))
    };
    // A new touch (e.g. a second finger) replaces the old one: un-highlight
    // its icon, or it would stay half-transparent.
    if previous != nil {
        () = msg![env; previous setAlpha:(1.0 as CGFloat)];
    }

    for &button in &buttons {
        let frame: CGRect = msg![env; button frame];
        if point.x >= frame.origin.x && point.x < frame.origin.x + frame.size.width &&
           point.y >= frame.origin.y && point.y < frame.origin.y + frame.size.height {
            () = msg![env; button setAlpha:(0.5 as CGFloat)];
            let host_obj = env.objc.borrow_mut::<AppPickerGridViewHostObject>(this);
            host_obj.highlighted_button = button;
            break;
        }
    }
}

- (())touchesMoved:(id)touches withEvent:(id)_event {
    let touch: id = msg![env; touches anyObject];
    let point: CGPoint = msg![env; touch locationInView:this];

    let (start_point, highlighted_button) = {
        let host_obj = env.objc.borrow::<AppPickerGridViewHostObject>(this);
        (host_obj.start_point, host_obj.highlighted_button)
    };
    if let Some(start_point) = start_point {
        let dx = point.x - start_point.x;
        if dx.abs() > 50.0 && highlighted_button != nil {
            () = msg![env; highlighted_button setAlpha:(1.0 as CGFloat)];
            let host_obj = env.objc.borrow_mut::<AppPickerGridViewHostObject>(this);
            host_obj.highlighted_button = nil;
        }
    }
}

- (())touchesEnded:(id)touches withEvent:(id)_event {
    let touch: id = msg![env; touches anyObject];
    let point: CGPoint = msg![env; touch locationInView:this];

    let (delegate, start_point, highlighted_button) = {
        let host_obj = env.objc.borrow::<AppPickerGridViewHostObject>(this);
        (host_obj.delegate, host_obj.start_point, host_obj.highlighted_button)
    };

    if let Some(start_point) = start_point {
        let dx = point.x - start_point.x;
        if dx > 50.0 {
            () = msg![env; delegate prevPage];
        } else if dx < -50.0 {
            () = msg![env; delegate nextPage];
        } else if highlighted_button != nil {
            // Only launch the icon that was pressed, and only if the finger
            // is still on it: not whichever icon the finger ends up on.
            let frame: CGRect = msg![env; highlighted_button frame];
            if point.x >= frame.origin.x && point.x < frame.origin.x + frame.size.width &&
               point.y >= frame.origin.y && point.y < frame.origin.y + frame.size.height {
                () = msg![env; delegate gridTappedAt:point];
            }
        }
    }

    if highlighted_button != nil {
        () = msg![env; highlighted_button setAlpha:(1.0 as CGFloat)];
    }
    let host_obj = env.objc.borrow_mut::<AppPickerGridViewHostObject>(this);
    host_obj.start_point = None;
    host_obj.highlighted_button = nil;
}

- (())touchesCancelled:(id)_touches withEvent:(id)_event {
    // Forget the touch without acting on it, and un-highlight its icon.
    let highlighted_button = {
        let host_obj = env.objc.borrow_mut::<AppPickerGridViewHostObject>(this);
        host_obj.start_point = None;
        std::mem::take(&mut host_obj.highlighted_button)
    };
    if highlighted_button != nil {
        () = msg![env; highlighted_button setAlpha:(1.0 as CGFloat)];
    }
}

@end

};

fn show_app_picker_gui(
    options: Options,
    apps: Result<Vec<AppInfo>, String>,
) -> Result<(PathBuf, Vec<String>), String> {
    let icon = {
        // iDared 32bit has one icon for every branding.
        let bytes: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/icon.png"));
        let mut image = Image::from_bytes(bytes).unwrap();
        // should match Bundle::load_icon()
        image.round_corners(
            (10.0 / 57.0) * (image.dimensions().0 as f32),
            /* four_corners: */ true,
        );
        image
    };
    let environment = Environment::new_without_app(options, icon)?;
    Ok(environment.run_app_picker(|env| app_picker_inner(env, apps)))
}

fn app_picker_inner(
    env: &mut Environment,
    mut apps: Result<Vec<AppInfo>, String>,
) -> (PathBuf, Vec<String>) {
    let mut option_args = Vec::new();
    // Note that objects are generally not released in this code, because they
    // don't need to be: the entire Environment is thrown away at the end.

    // Bypassing UIApplicationMain!
    let ui_application: id = msg_class![env; UIApplication new];
    let delegate = env
        .objc
        .get_known_class("_touchHLE_AppPickerDelegate", &mut env.mem);
    let delegate = env.objc.alloc_object(
        delegate,
        Box::<AppPickerDelegateHostObject>::default(),
        &mut env.mem,
    );
    () = msg![env; ui_application setDelegate:delegate];

    let screen: id = msg_class![env; UIScreen mainScreen];
    let bounds: CGRect = msg![env; screen bounds];

    let window: id = msg_class![env; UIWindow alloc];
    let window: id = msg![env; window initWithFrame:bounds];

    // When the screen is the device's shape, lay things out like the iOS home
    // screen: the whole screen is used, but icons and buttons stay in the safe
    // area, clear of the Dynamic Island or notch and the home indicator.
    let fitted = env.window().is_screen_fitted();
    let (safe_top, _, safe_bottom, _) = env.window().screen_safe_insets();
    // The version label goes at the top, beside the Dynamic Island or notch
    // if there is one, so keep the icons below it even when nothing else is
    // there (e.g. an iPad with a Home button).
    let top_bar = if fitted { safe_top.max(20.0) } else { 0.0 };
    if fitted {
        () = msg![env; ui_application setStatusBarHidden:true];
    }

    let app_frame: CGRect = msg![env; screen applicationFrame];
    let main_view: id = msg_class![env; UIView alloc];
    let main_view: id = msg![env; main_view initWithFrame:app_frame];
    () = msg![env; window addSubview:main_view];
    let safe_frame = CGRect {
        origin: CGPoint {
            x: 0.0,
            y: safe_top,
        },
        size: CGSize {
            width: app_frame.size.width,
            height: app_frame.size.height - safe_top - safe_bottom,
        },
    };

    // Wallpaper
    let mut found_wallpaper = false;
    let mut have_wallpaper = false;
    for candidate in paths::wallpaper_files() {
        let candidate = paths::user_data_base_path().join(candidate);
        if !candidate.exists() {
            continue;
        }
        found_wallpaper = true;

        let image = match std::fs::read(&candidate) {
            Ok(image) => image,
            Err(e) => {
                log!("Warning: couldn't read {}: {}", candidate.display(), e);
                break;
            }
        };
        let image = match Image::from_bytes(&image) {
            Ok(image) => image,
            Err(e) => {
                log!("Warning: couldn't decode {}: {}", candidate.display(), e);
                break;
            }
        };

        let image = cg_image::from_image(env, image);
        let image: id = msg_class![env; UIImage imageWithCGImage:image];
        let wallpaper: id = msg_class![env; UIImageView alloc];
        let wallpaper: id = msg![env; wallpaper initWithImage:image];
        () = msg![env; wallpaper setFrame:(CGRect {
            origin: CGPoint {
                x: 0.0,
                y: 0.0,
            },
            size: app_frame.size,
        })];
        // Crop the middle of the image to the screen's shape rather than
        // stretching it.
        let image_size: CGSize = msg![env; image size];
        let scale_x = app_frame.size.width / image_size.width;
        let scale_y = app_frame.size.height / image_size.height;
        let scale = scale_x.max(scale_y);
        let (crop_w, crop_h) = (scale_x / scale, scale_y / scale);
        let crop = CGRect {
            origin: CGPoint {
                x: (1.0 - crop_w) / 2.0,
                y: (1.0 - crop_h) / 2.0,
            },
            size: CGSize {
                width: crop_w,
                height: crop_h,
            },
        };
        let layer: id = msg![env; wallpaper layer];
        () = msg![env; layer setContentsRect:crop];
        () = msg![env; wallpaper setAlpha:(0.5 as CGFloat)];
        () = msg![env; main_view addSubview:wallpaper];
        have_wallpaper = true;
        break;
    }
    if !found_wallpaper {
        let CGSize { width, height } = app_frame.size;
        log!(
            "No wallpaper found; filename can be one of: {}; ideal size is {}×{} pixels",
            paths::wallpaper_files().join(", "),
            width,
            height,
        );
    }

    // Version label
    {
        let mut font_size: CGFloat = 12.0;
        let mut font: id = msg_class![env; UIFont systemFontOfSize:font_size];
        let full_text = format!(
            "iDared 32bit {}{}{} · Dev 2.34",
            crate::branding(),
            if crate::branding().is_empty() {
                ""
            } else {
                " "
            },
            crate::DISPLAY_VERSION
        );
        // Devices with a Dynamic Island or notch have a taller safe area inset
        // than just a status bar: 59+ and 44–50 points respectively.
        let device_safe_top = safe_top / env.window().screen_scale();
        let (label_frame, alignment, text) = if fitted && device_safe_top >= 40.0 {
            // Beside the Dynamic Island or notch, where the clock would be.
            // The notch is wider than the island, so there's less room.
            let cutout_fraction = if device_safe_top >= 55.0 { 0.33 } else { 0.56 };
            let room = app_frame.size.width * (1.0 - cutout_fraction) / 2.0;
            let text = ns_string::from_rust_string(env, full_text);
            let size: CGSize = msg![env; text sizeWithFont:font];
            let text = if size.width <= room - 12.0 {
                text
            } else {
                // A little smaller is still readable, any smaller isn't.
                font_size = 11.0;
                font = msg_class![env; UIFont systemFontOfSize:font_size];
                let size: CGSize = msg![env; text sizeWithFont:font];
                if size.width <= room - 12.0 {
                    text
                } else {
                    ns_string::from_rust_string(env, crate::DISPLAY_VERSION.to_string())
                }
            };
            let frame = CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: (safe_top - 15.0) / 2.0,
                },
                size: CGSize {
                    width: room,
                    height: 15.0,
                },
            };
            (frame, UITextAlignmentCenter, text)
        } else if fitted {
            // In the top left, where the clock would be.
            let frame = CGRect {
                origin: CGPoint {
                    x: 8.0,
                    y: (top_bar - 15.0) / 2.0,
                },
                size: CGSize {
                    width: app_frame.size.width / 2.0,
                    height: 15.0,
                },
            };
            let text = ns_string::from_rust_string(env, full_text);
            (frame, UITextAlignmentLeft, text)
        } else {
            let frame = CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: app_frame.size.height - 20.0,
                },
                size: CGSize {
                    width: app_frame.size.width - 5.0,
                    height: 15.0,
                },
            };
            let text = ns_string::from_rust_string(env, full_text);
            (frame, UITextAlignmentRight, text)
        };
        let label: id = msg_class![env; UILabel alloc];
        let label: id = msg![env; label initWithFrame:label_frame];
        () = msg![env; label setText:text];
        () = msg![env; label setTextAlignment:alignment];
        () = msg![env; label setFont:font];
        let text_color: id = if have_wallpaper {
            msg_class![env; UIColor whiteColor]
        } else {
            msg_class![env; UIColor lightGrayColor]
        };
        () = msg![env; label setTextColor:text_color];
        let bg_color: id = msg_class![env; UIColor clearColor];
        () = msg![env; label setBackgroundColor:bg_color];
        () = msg![env; main_view addSubview:label];
    }

    // The buttons take the place of the dock. Without a home indicator they
    // sit where they always have.
    let bottom_gap = if fitted { safe_bottom.max(8.0) } else { 22.5 };
    let buttons_row2_center = app_frame.size.height - bottom_gap - 15.0;
    let buttons_row_center = buttons_row2_center - 37.5;
    let divider = buttons_row_center - 25.0;
    let grid_frame = CGRect {
        origin: CGPoint { x: 0.0, y: top_bar },
        size: CGSize {
            width: app_frame.size.width,
            height: divider - top_bar,
        },
    };

    let mut icon_grid_stuff = match &mut apps {
        Ok(ref mut apps) => {
            let mut icon_grid_stuff = make_icon_grid(
                env,
                delegate,
                main_view,
                grid_frame,
                fitted,
                apps.len(),
                have_wallpaper,
            );
            update_icon_grid(env, &mut icon_grid_stuff, apps, 0);
            Some(icon_grid_stuff)
        }
        Err(e) => {
            let label_frame = CGRect {
                origin: CGPoint {
                    x: 10.0,
                    y: grid_frame.origin.y + 10.0,
                },
                size: CGSize {
                    width: grid_frame.size.width - 20.0,
                    height: grid_frame.size.height - 20.0,
                },
            };
            let label: id = msg_class![env; UILabel alloc];
            let label: id = msg![env; label initWithFrame:label_frame];
            let text = ns_string::from_rust_string(env, e.clone());
            () = msg![env; label setText:text];
            () = msg![env; label setTextAlignment:UITextAlignmentCenter];
            () = msg![env; label setNumberOfLines:0]; // unlimited
            let text_color: id = msg_class![env; UIColor lightGrayColor];
            () = msg![env; label setTextColor:text_color];
            let bg_color: id = msg_class![env; UIColor clearColor];
            () = msg![env; label setBackgroundColor:bg_color];
            () = msg![env; main_view addSubview:label];
            None
        }
    };

    make_button_row(
        env,
        delegate,
        main_view,
        app_frame.size,
        buttons_row_center,
        &[
            ("File manager", "openFileManager"),
            ("Quick options", "quickOptionsShow"),
        ],
        None,
    );
    make_button_row(
        env,
        delegate,
        main_view,
        app_frame.size,
        buttons_row2_center,
        &[
            ("Copyright info", "copyrightInfoShow"),
            ("iDared 32bit Web", "visitWebsite"),
        ],
        None,
    );

    let copyright_info_text = crate::licenses::get_text();
    let mut copyright_info_stuff = setup_copyright_info(env, delegate, main_view, safe_frame);
    let mut copyright_info_page_idx = 0;

    let quick_options_stuff = setup_quick_options(env, delegate, main_view, safe_frame);
    let mut quick_option_values: Vec<usize> =
        quick_options().map(QuickOption::default_value).collect();

    fn update_quick_option_buttons(env: &mut Environment, buttons: &[id], selected_idx: usize) {
        for (idx, &button) in buttons.iter().enumerate() {
            let color: id = if idx == selected_idx {
                msg_class![env; UIColor magentaColor]
            } else {
                msg_class![env; UIColor grayColor]
            };
            () = msg![env; button setBackgroundColor:color];
        }
    }
    for (option, controls) in quick_options().zip(&quick_options_stuff.controls) {
        if let QuickOptionKind::Choice(_) = option.kind {
            update_quick_option_buttons(env, controls, option.default_value());
        }
    }

    () = msg![env; window makeKeyAndVisible];

    let main_run_loop: id = msg_class![env; NSRunLoop mainRunLoop];
    let mut current_page_idx = 0;
    // If an app is picked, this loop returns. If the user quits touchHLE, the
    // process exits.
    let app_path = loop {
        run_run_loop_single_iteration(env, main_run_loop);
        let host_obj = env.objc.borrow_mut::<AppPickerDelegateHostObject>(delegate);

        if std::mem::take(&mut host_obj.prev_page) {
            if current_page_idx > 0 {
                current_page_idx -= 1;
                let grid_stuff = icon_grid_stuff.as_mut().unwrap();
                update_icon_grid(env, grid_stuff, apps.as_mut().unwrap(), current_page_idx);
                () = msg![env; (grid_stuff.page_control) setCurrentPage:(current_page_idx as NSUInteger)];
            }
            continue;
        }
        if let Some(idx) = std::mem::take(&mut host_obj.requested_page) {
            let grid_stuff = icon_grid_stuff.as_mut().unwrap();
            if idx < grid_stuff.pages.len() && idx != current_page_idx {
                current_page_idx = idx;
                update_icon_grid(env, grid_stuff, apps.as_mut().unwrap(), current_page_idx);
                () = msg![env; (grid_stuff.page_control) setCurrentPage:(current_page_idx as NSUInteger)];
            }
            continue;
        }
        if std::mem::take(&mut host_obj.next_page) {
            if current_page_idx + 1 < icon_grid_stuff.as_ref().unwrap().pages.len() {
                current_page_idx += 1;
                let grid_stuff = icon_grid_stuff.as_mut().unwrap();
                update_icon_grid(env, grid_stuff, apps.as_mut().unwrap(), current_page_idx);
                () = msg![env; (grid_stuff.page_control) setCurrentPage:(current_page_idx as NSUInteger)];
            }
            continue;
        }
        if let Some(point) = std::mem::take(&mut host_obj.grid_tapped_point) {
            let grid_stuff = icon_grid_stuff.as_mut().unwrap();
            let app_idx_range = grid_stuff.pages[current_page_idx].clone();
            let mut app_idx = None;

            for (i, &(button, _label)) in grid_stuff.icon_buttons_and_labels.iter().enumerate() {
                let frame: CGRect = msg![env; button frame];
                if point.x >= frame.origin.x
                    && point.x < frame.origin.x + frame.size.width
                    && point.y >= frame.origin.y
                    && point.y < frame.origin.y + frame.size.height
                {
                    let idx = app_idx_range.start + i;
                    if idx < app_idx_range.end {
                        app_idx = Some((idx, button));
                    }
                    break;
                }
            }

            if let Some((app_idx, icon_button)) = app_idx {
                // Provide visual feedback that the app has been picked
                // (it may take a while for the splash screen to appear etc)
                () = msg![env; icon_button setAlpha:(0.5 as CGFloat)];
                // Redraw screen, even if this makes the next frame early
                // (the app picker will never be redrawn after this).
                crate::frameworks::core_animation::recomposite_if_necessary(
                    env, /* force: */ true,
                );
                // Ensure touchHLE is responsive from the OS perspective,
                // otherwise screen redraw might not show up? (Unclear if
                // this explanation is correct.)
                run_run_loop_single_iteration(env, main_run_loop);

                let app_path = &apps.as_ref().unwrap()[app_idx].path;
                echo!("Picked: {}", app_path.display());
                break app_path.clone();
            }
            continue;
        }

        if std::mem::take(&mut host_obj.copyright_show) {
            copyright_info_page_idx = 0;
            change_copyright_page(
                env,
                &mut copyright_info_stuff,
                &copyright_info_text,
                copyright_info_page_idx,
            );
            () = msg![env; (copyright_info_stuff.main_view) setHidden:false];
        } else if std::mem::take(&mut host_obj.copyright_hide) {
            () = msg![env; (copyright_info_stuff.main_view) setHidden:true];
        } else if std::mem::take(&mut host_obj.copyright_prev) && copyright_info_page_idx != 0 {
            copyright_info_page_idx -= 1;
            change_copyright_page(
                env,
                &mut copyright_info_stuff,
                &copyright_info_text,
                copyright_info_page_idx,
            );
        } else if std::mem::take(&mut host_obj.copyright_next)
            && Some(copyright_info_page_idx) != copyright_info_stuff.last_page_idx
        {
            copyright_info_page_idx += 1;
            change_copyright_page(
                env,
                &mut copyright_info_stuff,
                &copyright_info_text,
                copyright_info_page_idx,
            );
        } else if std::mem::take(&mut host_obj.quick_options_show) {
            () = msg![env; (quick_options_stuff.main_view) setHidden:false];
        } else if std::mem::take(&mut host_obj.quick_options_hide) {
            () = msg![env; (quick_options_stuff.main_view) setHidden:true];
        } else if let Some(button) = std::mem::take(&mut host_obj.quick_option_choice) {
            for (i, buttons) in quick_options_stuff.controls.iter().enumerate() {
                if let Some(choice) = buttons.iter().position(|&b| b == button) {
                    quick_option_values[i] = choice;
                    update_quick_option_buttons(env, buttons, choice);
                }
            }
        } else if let Some(switch) = std::mem::take(&mut host_obj.quick_option_switch) {
            for (i, controls) in quick_options_stuff.controls.iter().enumerate() {
                if controls == &[switch] {
                    let on: bool = msg![env; switch isOn];
                    quick_option_values[i] = on as usize;
                }
            }
        }
    };

    // Apply user-specified overrides
    option_args.extend(quick_option_args(&quick_option_values));

    // Return the environment so some parts of it can be salvaged.
    (app_path, option_args)
}

const ICON_SIZE: CGSize = CGSize {
    width: 57.0,
    height: 57.0,
};
const LABEL_WIDTH: CGFloat = 74.0;
/// About two characters wider, used when the neighbouring names leave room.
const LABEL_MAX_WIDTH: CGFloat = 88.0;

struct IconGridStuff {
    page_control: id,
    num_cols: usize,
    icon_buttons_and_labels: Vec<(id, id)>,
    placeholder_icon: Option<id>,
    pages: Vec<std::ops::Range<usize>>,
}

fn make_icon_grid(
    env: &mut Environment,
    delegate: id,
    main_view: id,
    grid_frame: CGRect,
    spread_rows: bool,
    total_app_count: usize,
    have_wallpaper: bool,
) -> IconGridStuff {
    let label_size = CGSize {
        width: LABEL_WIDTH,
        height: 13.0,
    };
    let icon_gap_x: CGFloat = 19.0;
    // 4 columns on a 320-point screen, more on a wider one.
    let num_cols = (((grid_frame.size.width - 35.0 + icon_gap_x) / (ICON_SIZE.width + icon_gap_x))
        .floor() as usize)
        .max(1);
    let num_cols_f = num_cols as CGFloat;
    let mut icon_gap_y: CGFloat = 4.0 + label_size.height + 8.0;
    let icon_grid_width = (ICON_SIZE.width * num_cols_f) + icon_gap_x * (num_cols_f - 1.0);
    let icon_grid_origin = CGPoint {
        x: (grid_frame.size.width - icon_grid_width) / 2.0,
        y: 8.0,
    };

    let grid_view_frame = grid_frame;

    // As many rows as fit above the page control: 4 on a 480-point screen,
    // more on a taller one.
    let rows_height = grid_view_frame.size.height - 26.0 - icon_grid_origin.y;
    let num_rows =
        (((rows_height + icon_gap_y) / (ICON_SIZE.height + icon_gap_y)).floor() as usize).max(1);
    // Like the iOS home screen, spread the rows out to fill the space.
    // The last row's label and a little gap must still fit above the dots.
    if spread_rows && num_rows > 1 {
        let used = ICON_SIZE.height * num_rows as CGFloat + icon_gap_y * (num_rows - 1) as CGFloat;
        let spare = rows_height - (4.0 + label_size.height + 6.0) - used;
        icon_gap_y += spare.max(0.0) / (num_rows - 1) as CGFloat;
    }
    let grid_view: id = msg_class![env; _touchHLE_AppPickerGridView alloc];
    let grid_view: id = msg![env; grid_view initWithFrame:grid_view_frame];
    {
        let host_obj = env
            .objc
            .borrow_mut::<AppPickerGridViewHostObject>(grid_view);
        host_obj.delegate = delegate;
    }
    () = msg![env; main_view addSubview:grid_view];

    let page_control: id = msg_class![env; _touchHLE_AppPickerPageControl alloc];
    let page_control_frame = CGRect {
        origin: CGPoint {
            x: 0.0,
            // The dots stay where they were, 10 points above the grid's
            // bottom, but the strip is taller, to be easier to tap.
            y: grid_view_frame.origin.y + grid_view_frame.size.height - 26.0,
        },
        size: CGSize {
            width: grid_view_frame.size.width,
            height: 32.0,
        },
    };
    let page_control: id = msg![env; page_control initWithFrame:page_control_frame];
    // Just the dots, over the wallpaper.
    let bg_color: id = msg_class![env; UIColor clearColor];
    () = msg![env; page_control setBackgroundColor:bg_color];
    () = msg![env; page_control setOpaque:false];
    () = msg![env; page_control setDelegate:delegate];
    () = msg![env; main_view addSubview:page_control];

    let mut icon_buttons_and_labels = Vec::new();

    for i in 0..(num_cols * num_rows) {
        let col = i % num_cols;
        let row = i / num_cols;

        // Rounding is needed here to avoid a blurry or offset image.
        let icon_frame = CGRect {
            origin: CGPoint {
                x: (icon_grid_origin.x + (col as CGFloat) * (ICON_SIZE.width + icon_gap_x)).round(),
                y: (icon_grid_origin.y + (row as CGFloat) * (ICON_SIZE.height + icon_gap_y))
                    .round(),
            },
            size: ICON_SIZE,
        };
        let icon_button: id = msg_class![env; UIButton buttonWithType:UIButtonTypeCustom];
        () = msg![env; icon_button setFrame:icon_frame];
        let image_view: id = msg![env; icon_button imageView];
        let bounds: CGRect = msg![env; icon_button bounds];
        () = msg![env; image_view setFrame:bounds];
        () = msg![env; icon_button setUserInteractionEnabled:false];
        () = msg![env; grid_view addSubview:icon_button];

        // Rounding is needed here to avoid blurry text.
        let label_frame = CGRect {
            origin: CGPoint {
                x: (icon_frame.origin.x - (label_size.width - ICON_SIZE.width) / 2.0).round(),
                y: (icon_frame.origin.y + ICON_SIZE.height + 4.0).round(),
            },
            size: label_size,
        };
        let label: id = msg_class![env; UILabel alloc];
        let label: id = msg![env; label initWithFrame:label_frame];
        () = msg![env; label setTextAlignment:UITextAlignmentCenter];
        let font_size: CGFloat = label_size.height - 2.0;
        let font: id = if have_wallpaper {
            msg_class![env; UIFont systemFontOfSize:font_size]
        } else {
            msg_class![env; UIFont boldSystemFontOfSize:font_size]
        };
        () = msg![env; label setFont:font];
        let text_color: id = if have_wallpaper {
            msg_class![env; UIColor whiteColor]
        } else {
            msg_class![env; UIColor lightGrayColor]
        };
        () = msg![env; label setTextColor:text_color];
        let bg_color: id = msg_class![env; UIColor clearColor];
        () = msg![env; label setBackgroundColor:bg_color];
        () = msg![env; grid_view addSubview:label];

        icon_buttons_and_labels.push((icon_button, label));

        {
            let host_obj = env
                .objc
                .borrow_mut::<AppPickerGridViewHostObject>(grid_view);
            host_obj.icon_buttons.push(icon_button);
        }
    }

    // TODO: Use UIScrollView pagination and UIPageControl once available.
    let mut pages = Vec::new();
    let mut start = 0;
    while start < total_app_count {
        let end = (start + icon_buttons_and_labels.len()).min(total_app_count);
        pages.push(start..end);
        start = end;
    }

    let page_count = pages.len();
    () = msg![env; page_control setNumberOfPages:(page_count as NSUInteger)];

    IconGridStuff {
        page_control,
        num_cols,
        icon_buttons_and_labels,
        placeholder_icon: None,
        pages,
    }
}

fn make_icon_from_glyph(
    env: &mut Environment,
    glyph: char,
    font_size: CGFloat,
    baseline_offset: CGFloat,
    bg_color: (CGFloat, CGFloat, CGFloat, CGFloat),
) -> id {
    let color_space = CGColorSpaceCreateDeviceRGB(env);
    let context = CGBitmapContextCreate(
        env,
        Ptr::null(),
        ICON_SIZE.width as u32,
        ICON_SIZE.height as u32,
        8,
        4 * (ICON_SIZE.width as u32),
        color_space,
        kCGImageAlphaPremultipliedLast,
    );
    UIGraphicsPushContext(env, context);

    // Compensate for row order inversion
    CGContextTranslateCTM(env, context, 0.0, ICON_SIZE.height);
    CGContextScaleCTM(env, context, 1.0, -1.0);

    let (r, g, b, a) = bg_color;
    CGContextSetRGBFillColor(env, context, r, g, b, a);
    CGContextFillRect(
        env,
        context,
        CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: ICON_SIZE,
        },
    );

    let font: id = msg_class![env; UIFont systemFontOfSize:font_size];
    let glyph_string: id = ns_string::from_rust_string(env, [glyph].into_iter().collect());
    let glyph_size: CGSize = msg![env; glyph_string sizeWithFont:font];
    CGContextSetRGBFillColor(env, context, 1.0, 1.0, 1.0, 1.0); // white
    let glyph_origin = CGPoint {
        x: ICON_SIZE.width / 2.0 - glyph_size.width / 2.0,
        y: ICON_SIZE.height / 2.0 - glyph_size.height / 2.0 + baseline_offset,
    };
    let _: CGSize = msg![env; glyph_string drawAtPoint:glyph_origin withFont:font];
    release(env, glyph_string);

    UIGraphicsPopContext(env);

    let cg_image = CGBitmapContextCreateImage(env, context);
    // This radius should match the one in src/bundle.rs.
    cg_image::borrow_image_mut(&mut env.objc, cg_image).round_corners(
        (10.0 / 57.0) * ICON_SIZE.width,
        /* four_corners: */ true,
    );
    CGContextRelease(env, context);

    let ui_image: id = msg_class![env; UIImage imageWithCGImage:cg_image];
    release(env, cg_image);

    ui_image
}

fn update_icon_grid(
    env: &mut Environment,
    icon_grid_stuff: &mut IconGridStuff,
    apps: &mut [AppInfo],
    page_idx: usize,
) {
    let app_idx_range = icon_grid_stuff.pages[page_idx].clone();

    let mut icon_iter = icon_grid_stuff.icon_buttons_and_labels.iter();

    for app_idx in app_idx_range.clone() {
        let app = &mut apps[app_idx];

        let &(icon_button, label) = icon_iter.next().unwrap();

        if let Some(icon) = app.icon.take() {
            let image = cg_image::from_image(env, icon);
            let image: id = msg_class![env; UIImage imageWithCGImage:image];
            app.icon_ui_image = Some(image);
        }

        let image = app.icon_ui_image.unwrap_or_else(|| {
            *icon_grid_stuff.placeholder_icon.get_or_insert_with(|| {
                make_icon_from_glyph(env, '?', 40.0, 0.0, (0.5, 0.5, 0.5, 1.0))
            })
        });
        () = msg![env; icon_button setImage:image forState:UIControlStateNormal];

        let text = *app
            .display_name_ns_string
            .get_or_insert_with(|| ns_string::from_rust_string(env, app.display_name.clone()));
        () = msg![env; label setText:text];
    }

    // There may be remaining spaces might need to be blanked.
    for &(icon_button, label) in icon_iter {
        () = msg![env; icon_button setImage:nil forState:UIControlStateNormal];
        () = msg![env; label setText:(ns_string::get_static_str(env, ""))];
    }

    // Let a name grow past LABEL_WIDTH only while it can't touch the names
    // either side of it.
    let num_cols = icon_grid_stuff.num_cols;
    let items = &icon_grid_stuff.icon_buttons_and_labels;
    let text_widths: Vec<CGFloat> = items
        .iter()
        .map(|&(_, label)| {
            let text: id = msg![env; label text];
            let font: id = msg![env; label font];
            let size: CGSize = msg![env; text sizeWithFont:font];
            size.width.min(LABEL_MAX_WIDTH)
        })
        .collect();
    for (i, &(icon_button, label)) in items.iter().enumerate() {
        let mut width = text_widths[i];
        if i % num_cols > 0 {
            width = width.min(2.0 * LABEL_WIDTH - text_widths[i - 1]);
        }
        if i % num_cols + 1 < num_cols && i + 1 < items.len() {
            width = width.min(2.0 * LABEL_WIDTH - text_widths[i + 1]);
        }
        let width = width.max(LABEL_WIDTH).ceil();
        let icon_frame: CGRect = msg![env; icon_button frame];
        let mut frame: CGRect = msg![env; label frame];
        frame.origin.x = (icon_frame.origin.x + (ICON_SIZE.width - width) / 2.0).round();
        frame.size.width = width;
        () = msg![env; label setFrame:frame];

        // Like iOS, end names that still don't fit with an ellipsis.
        let text: id = msg![env; label text];
        let font: id = msg![env; label font];
        let name = ns_string::to_rust_string(env, text);
        if size_with_font(env, font, &name, None).width > width {
            let chars: Vec<char> = name.chars().collect();
            let truncated = (0..chars.len())
                .rev()
                .map(|n| format!("{}…", chars[..n].iter().collect::<String>().trim_end()))
                .find(|t| size_with_font(env, font, t, None).width <= width)
                .unwrap_or_else(|| "…".to_owned());
            // Rather than cut into the first word, drop the words after it.
            let first_word = name.split(' ').next().unwrap();
            let cuts_first_word = truncated.len() - '…'.len_utf8() < first_word.len();
            let truncated =
                if cuts_first_word && size_with_font(env, font, first_word, None).width <= width {
                    first_word.to_owned()
                } else {
                    truncated
                };
            let truncated = ns_string::from_rust_string(env, truncated);
            () = msg![env; label setText:truncated];
            release(env, truncated);
        }
    }
}

fn make_button_row(
    env: &mut Environment,
    delegate: id,
    super_view: id,
    super_view_size: CGSize,
    buttons_row_center: CGFloat,
    buttons: &[(&'static str, &'static str)],
    font_size: Option<CGFloat>,
) -> Vec<id> {
    let margin = 10.0;

    let button_size = CGSize {
        width: (super_view_size.width - margin) / (buttons.len() as CGFloat) - margin,
        height: 30.0,
    };
    let mut button_frame = CGRect {
        origin: CGPoint {
            x: margin,
            y: buttons_row_center - button_size.height / 2.0,
        },
        size: button_size,
    };

    let mut ui_buttons = Vec::new();
    for (title_text, selector) in buttons {
        let button: id = msg_class![env; UIButton buttonWithType:UIButtonTypeRoundedRect];
        let text = ns_string::get_static_str(env, title_text);
        () = msg![env; button setTitle:text forState:UIControlStateNormal];
        () = msg![env; button setFrame:button_frame];
        // FIXME: manually calling layoutSubviews shouldn't be needed?
        () = msg![env; button layoutSubviews];

        if let Some(font_size) = font_size {
            let label: id = msg![env; button titleLabel];
            let font: id = msg_class![env; UIFont systemFontOfSize:font_size];
            () = msg![env; label setFont:font];
        }

        let selector = env.objc.lookup_selector(selector).unwrap();
        () = msg![env; button addTarget:delegate
                                 action:selector
                       forControlEvents:UIControlEventTouchUpInside];
        () = msg![env; super_view addSubview:button];

        button_frame.origin.x += button_size.width + margin;
        ui_buttons.push(button);
    }
    ui_buttons
}

struct CopyrightInfoStuff {
    main_view: id,
    text_frame: CGRect,
    text_label: id,
    font: id,
    pages: Vec<(std::ops::Range<usize>, CGFloat)>,
    last_page_idx: Option<usize>,
    prev_page_button: id,
    next_page_button: id,
}

fn setup_copyright_info(
    env: &mut Environment,
    delegate: id,
    super_view: id,
    safe_frame: CGRect,
) -> CopyrightInfoStuff {
    let main_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: safe_frame.size,
    };

    let divider = main_frame.size.height - 40.0;

    // Container for all the other stuff

    let main_view: id = msg_class![env; UIView alloc];
    let full_frame: CGRect = msg![env; super_view bounds];
    let main_view: id = msg![env; main_view initWithFrame:full_frame];
    // TODO: Isn't white the default?
    let bg_color: id = msg_class![env; UIColor whiteColor];
    () = msg![env; main_view setBackgroundColor:bg_color];
    // This main_view is hidden until the copyright info button is tapped.
    () = msg![env; main_view setHidden:true];
    () = msg![env; super_view addSubview:main_view];

    // The background covers the whole screen, but the contents stay in the
    // safe area, clear of the Dynamic Island and home indicator.
    let content_view: id = msg_class![env; UIView alloc];
    let content_view: id = msg![env; content_view initWithFrame:safe_frame];
    let bg_color: id = msg_class![env; UIColor clearColor];
    () = msg![env; content_view setBackgroundColor:bg_color];
    () = msg![env; main_view addSubview:content_view];

    // UILabel that will display part of the copyright text

    let padding = 10.0;
    let text_frame = CGRect {
        origin: CGPoint {
            x: padding,
            y: padding,
        },
        size: CGSize {
            width: safe_frame.size.width - padding * 2.0,
            height: divider - padding * 2.0,
        },
    };

    let text_label: id = msg_class![env; UILabel alloc];
    let text_label: id = msg![env; text_label initWithFrame:text_frame];
    () = msg![env; text_label setNumberOfLines:0]; // unlimited
    let text_color: id = msg_class![env; UIColor blackColor];
    () = msg![env; text_label setTextColor:text_color];
    let bg_color: id = msg_class![env; UIColor clearColor];
    () = msg![env; text_label setBackgroundColor:bg_color];
    let font_size: CGFloat = 16.0;
    let font: id = msg_class![env; UIFont systemFontOfSize:font_size];
    () = msg![env; text_label setFont:font];
    () = msg![env; content_view addSubview:text_label];

    // Navigation

    let buttons_row_center = (main_frame.size.height + divider) / 2.0;
    let buttons = make_button_row(
        env,
        delegate,
        content_view,
        main_frame.size,
        buttons_row_center,
        &[
            ("↑", "copyrightInfoPrevPage"),
            ("↓", "copyrightInfoNextPage"),
            ("×", "copyrightInfoHide"),
        ],
        Some(30.0),
    );

    CopyrightInfoStuff {
        main_view,
        text_frame,
        text_label,
        font,
        pages: Vec::new(),
        last_page_idx: None,
        prev_page_button: buttons[0],
        next_page_button: buttons[1],
    }
}

fn change_copyright_page(
    env: &mut Environment,
    copyright_info_stuff: &mut CopyrightInfoStuff,
    copyright_info_text: &str,
    page_idx: usize,
) {
    // TODO: Eventually this should be ripped out and replaced with a scrolling
    // UITextView, once that's implemented.

    let &mut CopyrightInfoStuff {
        text_frame,
        text_label,
        font,
        ref mut pages,
        ref mut last_page_idx,
        prev_page_button,
        next_page_button,
        ..
    } = copyright_info_stuff;

    // Lazily lay out pages of text as needed.

    if page_idx == pages.len() {
        let mut page_start = pages.last().map_or(0, |page| page.0.end);
        while copyright_info_text[page_start..].starts_with([' ', '\n', '\r']) {
            page_start += 1;
        }
        let mut page_height = 0.0;
        let page_end = loop {
            let mut line_start = page_start;
            while line_start < copyright_info_text.len() {
                let is_first_line = line_start == page_start;

                let line_end = if let Some(i) = copyright_info_text[line_start..].find('\n') {
                    line_start + i + 1
                } else {
                    copyright_info_text.len()
                };

                let line = &copyright_info_text[line_start..line_end];

                // Force pagination before Markdown-style headings, if any
                // appear in the license text
                if !is_first_line && line.starts_with("###") {
                    break;
                }

                let line_temp = ns_string::from_rust_string(env, line.to_string());
                let line_size: CGSize = msg![env; line_temp sizeWithFont:font
                                                       constrainedToSize:(text_frame.size)];
                // Avoid accumulation of old line strings.
                release(env, line_temp);

                if page_height + line_size.height > text_frame.size.height {
                    break;
                }

                page_height += line_size.height;
                line_start = line_end;

                // Force pagination after dividers
                if !is_first_line && line.starts_with("---") {
                    break;
                }
            }
            let page_end = line_start;
            assert!(page_start != page_end);

            // Avoid entirely blank pages
            if copyright_info_text[page_start..page_end].trim() == "" {
                page_start = page_end;
            } else {
                break page_end;
            }
        };
        assert!(page_start != page_end);
        pages.push((page_start..page_end, page_height));
        if page_end == copyright_info_text.len() {
            *last_page_idx = Some(page_idx);
        }
    }

    // Actually display the page

    let (page, page_height) = pages[page_idx].clone();
    let page = &copyright_info_text[page];

    let page: id = ns_string::from_rust_string(env, page.to_string());
    () = msg![env; text_label setText:page];
    // Avoid accumulation of old page strings.
    release(env, page);

    // UILabel always vertically centers text. Work around that by resizing it.
    let label_frame = CGRect {
        origin: text_frame.origin,
        size: CGSize {
            width: text_frame.size.width,
            // The page height is slightly off, a little padding is needed.
            height: page_height + 10.0,
        },
    };
    () = msg![env; text_label setFrame:label_frame];

    () = msg![env; prev_page_button setHidden:(page_idx == 0)];
    () = msg![env; next_page_button setHidden:(Some(page_idx) == *last_page_idx)];
}

/// One of the quick options. Both this picker and the SwiftUI one on iOS
/// (see `app_picker/swiftui.rs`) are built from [QUICK_OPTIONS].
pub(super) struct QuickOption {
    pub label: &'static str,
    /// Shown under the option in the SwiftUI picker.
    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    pub footer: &'static str,
    pub kind: QuickOptionKind,
    /// Hidden on always-fullscreen platforms.
    pub desktop_only: bool,
}

pub(super) enum QuickOptionKind {
    /// Each choice is its title and the option argument it adds. The first is
    /// the default.
    Choice(&'static [(&'static str, Option<&'static str>)]),
    /// The option arguments added when it's on or off.
    Switch {
        default: bool,
        on: Option<&'static str>,
        off: Option<&'static str>,
    },
}

impl QuickOption {
    /// The choice index, or for a switch, 1 if it's on.
    pub fn default_value(&self) -> usize {
        match self.kind {
            QuickOptionKind::Choice(_) => 0,
            QuickOptionKind::Switch { default, .. } => default as usize,
        }
    }

    fn arg(&self, value: usize) -> Option<&'static str> {
        match self.kind {
            QuickOptionKind::Choice(choices) => choices[value].1,
            QuickOptionKind::Switch { on, off, .. } => {
                if value != 0 {
                    on
                } else {
                    off
                }
            }
        }
    }
}

const QUICK_OPTIONS: &[QuickOption] = &[
    QuickOption {
        label: "Scale hack",
        footer: "Attempt to scale the game to a higher resolution (some may not work).",
        kind: QuickOptionKind::Choice(&[
            ("Default", None),
            ("Off", Some("--scale-hack=1")),
            ("2×", Some("--scale-hack=2")),
            ("3×", Some("--scale-hack=3")),
            ("4×", Some("--scale-hack=4")),
        ]),
        desktop_only: false,
    },
    QuickOption {
        label: "Orientation",
        footer: "For games that are in the wrong orientation.",
        kind: QuickOptionKind::Choice(&[
            ("Default", None),
            ("←", Some("--landscape-left")),
            ("→", Some("--landscape-right")),
            ("↓", Some("--upside-down")),
        ]),
        desktop_only: false,
    },
    QuickOption {
        label: "Network access",
        footer: "Off means games work as if in airplane mode.",
        kind: QuickOptionKind::Switch {
            default: false,
            on: Some("--allow-network-access"),
            off: None,
        },
        desktop_only: false,
    },
    QuickOption {
        label: "Use analog sticks for tilt controls",
        footer: "Use a controller's analog sticks to tilt the device.",
        kind: QuickOptionKind::Switch {
            default: true,
            on: None,
            off: Some("--disable-analog-stick-tilt-controls"),
        },
        desktop_only: false,
    },
    QuickOption {
        label: "Device",
        footer: "Which version of a game made for both iPhone and iPad to run.",
        kind: QuickOptionKind::Choice(&[
            ("Default", None),
            ("iPhone", Some("--device-family=iphone")),
            ("iPad", Some("--device-family=ipad")),
        ]),
        desktop_only: false,
    },
    QuickOption {
        label: "Fullscreen (override)",
        footer: "",
        kind: QuickOptionKind::Switch {
            default: false,
            on: Some("--fullscreen"),
            off: None,
        },
        desktop_only: true,
    },
];

/// The quick options shown on this platform.
pub(super) fn quick_options() -> impl Iterator<Item = &'static QuickOption> {
    QUICK_OPTIONS
        .iter()
        .filter(|option| !option.desktop_only || !crate::window::Window::rotatable_fullscreen())
}

/// The option arguments for the values (see [QuickOption::default_value]) of
/// [quick_options].
pub(super) fn quick_option_args(values: &[usize]) -> impl Iterator<Item = String> + '_ {
    quick_options()
        .zip(values)
        .filter_map(|(option, &value)| option.arg(value).map(ToOwned::to_owned))
}

struct QuickOptionsStuff {
    main_view: id,
    /// For each of [quick_options], its buttons, or its switch.
    controls: Vec<Vec<id>>,
}

fn setup_quick_options(
    env: &mut Environment,
    delegate: id,
    super_view: id,
    safe_frame: CGRect,
) -> QuickOptionsStuff {
    // UIView*
    let main_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: safe_frame.size,
    };

    // Container for all the other stuff

    let main_view: id = msg_class![env; UIView alloc];
    let full_frame: CGRect = msg![env; super_view bounds];
    let main_view: id = msg![env; main_view initWithFrame:full_frame];
    // TODO: Isn't white the default?
    let bg_color: id = msg_class![env; UIColor whiteColor];
    () = msg![env; main_view setBackgroundColor:bg_color];
    // This main_view is hidden until the copyright info button is tapped.
    () = msg![env; main_view setHidden:true];
    () = msg![env; super_view addSubview:main_view];

    // The background covers the whole screen, but the contents stay in the
    // safe area, clear of the Dynamic Island and home indicator.
    let content_view: id = msg_class![env; UIView alloc];
    let content_view: id = msg![env; content_view initWithFrame:safe_frame];
    let bg_color: id = msg_class![env; UIColor clearColor];
    () = msg![env; content_view setBackgroundColor:bg_color];
    () = msg![env; main_view addSubview:content_view];

    let divider = 40.0;

    // Close button
    {
        let button_frame = CGRect {
            origin: CGPoint {
                x: main_frame.size.width - 30.0,
                y: 10.0,
            },
            size: CGSize {
                width: 20.0,
                height: 20.0,
            },
        };

        let button: id = msg_class![env; UIButton buttonWithType:UIButtonTypeRoundedRect];
        let text = ns_string::get_static_str(env, "×");
        () = msg![env; button setTitle:text forState:UIControlStateNormal];
        () = msg![env; button setFrame:button_frame];
        // FIXME: manually calling layoutSubviews shouldn't be needed?
        () = msg![env; button layoutSubviews];

        let label: id = msg![env; button titleLabel];
        let font: id = msg_class![env; UIFont systemFontOfSize:(30.0 as CGFloat)];
        () = msg![env; label setFont:font];

        let selector = env.objc.lookup_selector("quickOptionsHide").unwrap();
        () = msg![env; button addTarget:delegate
                                 action:selector
                       forControlEvents:UIControlEventTouchUpInside];
        () = msg![env; content_view addSubview:button];
    }

    // Each option is a label and a row of buttons or a switch. They're spaced
    // as if all were shown, even on platforms where some are hidden.
    let row_count = QUICK_OPTIONS.len() * 2;
    let row_center = |row: usize| {
        divider
            + ((1 + row) as CGFloat)
                * ((main_frame.size.height - divider) / ((row_count + 1) as CGFloat))
    };

    let mut controls = Vec::new();
    for (i, option) in quick_options().enumerate() {
        let label_center = row_center(i * 2);
        let frame = CGRect {
            origin: CGPoint {
                x: 0.0,
                y: label_center - 30.0 / 2.0,
            },
            size: CGSize {
                width: main_frame.size.width,
                height: 30.0,
            },
        };
        let label: id = msg_class![env; UILabel alloc];
        let label: id = msg![env; label initWithFrame:frame];
        let text = ns_string::get_static_str(env, option.label);
        () = msg![env; label setText:text];
        () = msg![env; label setTextAlignment:UITextAlignmentCenter];
        () = msg![env; content_view addSubview:label];

        let center = row_center(i * 2 + 1);
        match option.kind {
            QuickOptionKind::Choice(choices) => {
                let buttons: Vec<_> = choices
                    .iter()
                    .map(|&(title, _)| (title, "quickOptionChoice:"))
                    .collect();
                controls.push(make_button_row(
                    env,
                    delegate,
                    content_view,
                    main_frame.size,
                    center,
                    &buttons,
                    /* font_size: */ None,
                ));
            }
            QuickOptionKind::Switch { default, .. } => {
                let switch_frame = CGRect {
                    origin: CGPoint {
                        x: main_frame.size.width / 2.0 - 94.0 / 2.0,
                        y: center - 27.0 / 2.0,
                    },
                    size: Default::default(),
                };

                let switch: id = msg_class![env; UISwitch alloc];
                let switch: id = msg![env; switch initWithFrame:switch_frame];
                () = msg![env; switch setOn:default];
                let selector = env.objc.lookup_selector("quickOptionSwitch:").unwrap();
                () = msg![env; switch addTarget:delegate
                                         action:selector
                               forControlEvents:UIControlEventValueChanged];
                () = msg![env; content_view addSubview:switch];
                controls.push(vec![switch]);
            }
        }
    }

    QuickOptionsStuff {
        main_view,
        controls,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_test_directory() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("idared-app-picker-tests")
            .join(format!("{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn test_snapshot(directory: &Path) -> LibrarySnapshot {
        let mut entries = Vec::new();
        snapshot_entries(
            directory,
            "Games",
            |path| {
                path.extension().is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("ipa") || extension.eq_ignore_ascii_case("app")
                })
            },
            &mut entries,
        )
        .unwrap();
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        LibrarySnapshot(entries)
    }

    fn test_app(identifier: &str, name: &str) -> AppInfo {
        AppInfo {
            path: PathBuf::new(),
            bundle_identifier: identifier.to_owned(),
            display_name: name.to_owned(),
            compatibility: classify_picker_compatibility(identifier, name).to_owned(),
            icon: None,
            display_name_ns_string: None,
            icon_ui_image: None,
        }
    }

    #[test]
    fn picker_compatibility_is_display_only_and_conservative() {
        assert_eq!(
            picker_compatibility(&test_app("com.chillingo.spoing", "Spoing")),
            "playable"
        );
        assert_eq!(
            picker_compatibility(&test_app("com.sega.chuchu", "ChuChu Rocket")),
            "playable"
        );
        assert_eq!(
            picker_compatibility(&test_app("com.clickgamer.sticky", "Sticky")),
            "playable"
        );
        assert_eq!(
            picker_compatibility(&test_app("unknown.bundle", "Glowfish HD")),
            "needsWork"
        );
        assert_eq!(
            picker_compatibility(&test_app("unknown.bundle", "Splode")),
            "needsWork"
        );
        assert_eq!(
            picker_compatibility(&test_app("unknown.bundle", "Toy Bot 2")),
            "needsWork"
        );
        assert_eq!(
            picker_compatibility(&test_app("com.popcap.Peggle", "Peggle")),
            "needsWork"
        );
        assert_eq!(
            picker_compatibility(&test_app("unknown.bundle", "New Game")),
            "needsWork"
        );
    }

    #[test]
    fn cheap_library_snapshot_detects_relevant_direct_entry_changes() {
        let directory = snapshot_test_directory();
        let first = directory.join("First.ipa");
        std::fs::write(&first, b"first").unwrap();
        std::fs::write(directory.join("ignored.txt"), b"ignored").unwrap();

        let original = test_snapshot(&directory);
        assert_eq!(original, test_snapshot(&directory));

        let second = directory.join("Second.ipa");
        std::fs::write(&second, b"second").unwrap();
        let added = test_snapshot(&directory);
        assert_ne!(original, added);

        std::fs::remove_file(&second).unwrap();
        // Model a rename as the directory state iDared observes after it:
        // one entry removed and another entry added. Some Windows security
        // configurations deny rename operations in the test temp directory.
        std::fs::remove_file(&first).unwrap();
        std::fs::write(directory.join("Renamed.ipa"), b"first").unwrap();
        let renamed = test_snapshot(&directory);
        assert_ne!(original, renamed);

        let renamed_path = directory.join("Renamed.ipa");
        std::fs::write(&renamed_path, b"replacement with a different size").unwrap();
        let replaced = test_snapshot(&directory);
        assert_ne!(renamed, replaced);

        std::fs::remove_file(renamed_path).unwrap();
        let removed = test_snapshot(&directory);
        assert_ne!(replaced, removed);

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn library_snapshot_detects_nested_app_directory_changes() {
        let directory = snapshot_test_directory();
        let app = directory.join("Loose.app");
        std::fs::create_dir_all(app.join("Resources")).unwrap();
        let resource = app.join("Resources").join("asset.bin");
        std::fs::write(&resource, b"one").unwrap();

        let original = test_snapshot(&directory);
        assert_eq!(original, test_snapshot(&directory));

        std::fs::write(&resource, b"replacement with a different size").unwrap();
        let changed = test_snapshot(&directory);
        assert_ne!(original, changed);

        std::fs::remove_file(&resource).unwrap();
        assert_ne!(changed, test_snapshot(&directory));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn loose_ipa_import_moves_only_new_ipa_files() {
        let directory = snapshot_test_directory();
        let games = directory.join("Games");
        std::fs::create_dir_all(&games).unwrap();
        std::fs::write(directory.join("Imported.ipa"), b"ipa").unwrap();
        std::fs::write(directory.join("ignored.txt"), b"not an ipa").unwrap();
        std::fs::write(directory.join("Existing.ipa"), b"do not replace").unwrap();
        std::fs::write(games.join("Existing.ipa"), b"existing game").unwrap();

        move_loose_ipas_from(&directory, &games);

        assert!(!directory.join("Imported.ipa").exists());
        assert_eq!(std::fs::read(games.join("Imported.ipa")).unwrap(), b"ipa");
        assert!(directory.join("ignored.txt").is_file());
        assert_eq!(
            std::fs::read(games.join("Existing.ipa")).unwrap(),
            b"existing game"
        );
        assert!(directory.join("Existing.ipa").is_file());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn quick_option_args_from_values() {
        let mut values: Vec<usize> = quick_options().map(QuickOption::default_value).collect();
        assert_eq!(quick_option_args(&values).count(), 0);

        values[0] = 3; // Scale hack: 3×
        values[1] = 2; // Orientation: landscape right
        values[2] = 1; // Network access: on
        values[3] = 0; // Analog sticks for tilt controls: off
        values[4] = 2; // Device: iPad
        assert_eq!(
            quick_option_args(&values).collect::<Vec<_>>(),
            [
                "--scale-hack=3",
                "--landscape-right",
                "--allow-network-access",
                "--disable-analog-stick-tilt-controls",
                "--device-family=ipad",
            ],
        );
    }
}
