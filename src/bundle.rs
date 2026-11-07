/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Utilities for working with bundles. So far we are only interested in the
//! application bundle.
//!
//! Relevant Apple documentation:
//! * [Bundle Programming Guide](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/Introduction/Introduction.html)
//!   * [Anatomy of an iOS Application Bundle](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html)
//! * [Bundle Resources](https://developer.apple.com/documentation/bundleresources?language=objc)

use crate::frameworks::foundation::ns_bundle::LANG_ID_TO_LANG_PROJ;
use crate::fs::{BundleData, Fs, FsNodeType, GuestPath, GuestPathBuf};
use crate::image::Image;
use crate::window::DeviceFamily;
use plist::dictionary::Dictionary;
use plist::Value;
use std::io::Cursor;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default)]
pub struct BundleConstructionTimings {
    pub info_plist_read: Duration,
    pub info_plist_parse: Duration,
    pub filesystem_creation: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IconLoadTimings {
    pub candidate_discovery: Duration,
    pub reads: Duration,
    pub decoding: Duration,
    pub processing: Duration,
    pub candidates_considered: usize,
}

#[derive(Debug)]
pub struct Bundle {
    path: GuestPathBuf,
    plist: Dictionary,
}

impl Bundle {
    /// See [Fs::new] for meaning of `read_only_mode`.
    pub fn new_bundle_and_fs_from_host_path(
        bundle_data: BundleData,
        read_only_mode: bool,
    ) -> Result<(Bundle, Fs), String> {
        Self::new_bundle_and_fs_from_host_path_timed(bundle_data, read_only_mode)
            .map(|(bundle, fs, _)| (bundle, fs))
    }

    pub fn new_bundle_and_fs_from_host_path_timed(
        mut bundle_data: BundleData,
        read_only_mode: bool,
    ) -> Result<(Bundle, Fs, BundleConstructionTimings), String> {
        let mut timings = BundleConstructionTimings::default();
        let started = Instant::now();
        let plist_bytes = bundle_data.read_plist()?;
        timings.info_plist_read = started.elapsed();

        let started = Instant::now();
        let plist = Value::from_reader(Cursor::new(plist_bytes))
            .map_err(|_| "Could not deserialize plist data".to_string())?;

        let plist = plist
            .into_dictionary()
            .ok_or_else(|| "plist root value is not a dictionary".to_string())?;

        let bundle_name = format!(
            "{}.app",
            if let Some(canonical) = plist.get("CFBundleName") {
                canonical.as_string().unwrap()
            } else {
                bundle_data.bundle_name()
            }
        );
        let bundle_id = plist["CFBundleIdentifier"].as_string().unwrap();
        timings.info_plist_parse = started.elapsed();

        let started = Instant::now();
        let (fs, guest_path) = Fs::new(bundle_data, bundle_name, bundle_id, read_only_mode);
        timings.filesystem_creation = started.elapsed();

        let bundle = Bundle {
            path: guest_path,
            plist,
        };

        Ok((bundle, fs, timings))
    }

    /// Create a fake bundle (see [crate::Environment::new_without_app]).
    pub fn new_fake_bundle() -> Bundle {
        Bundle {
            path: GuestPathBuf::from(String::new()),
            plist: Dictionary::new(),
        }
    }

    #[cfg(test)]
    pub fn set_fake_identity_for_test(&mut self) {
        self.plist.insert(
            "CFBundleIdentifier".to_owned(),
            Value::String("com.touchhle.test".to_owned()),
        );
        self.plist.insert(
            "CFBundleVersion".to_owned(),
            Value::String("1".to_owned()),
        );
    }

    pub fn bundle_path(&self) -> &GuestPath {
        &self.path
    }

    pub fn bundle_identifier(&self) -> &str {
        self.plist["CFBundleIdentifier"].as_string().unwrap()
    }

    pub fn bundle_version(&self) -> &str {
        self.plist["CFBundleVersion"].as_string().unwrap()
    }

    /// The bundle version when it is present and is a string.
    ///
    /// Picker metadata treats this as optional because malformed historical
    /// packages must be skippable rather than crashing cache construction.
    pub fn bundle_version_optional(&self) -> Option<&str> {
        self.plist.get("CFBundleVersion").and_then(Value::as_string)
    }

    pub fn bundle_localizations(&self, fs: &Fs) -> Vec<String> {
        let declared = self
            .plist
            .get("CFBundleLocalizations")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(Value::as_string)
            .map(str::to_owned);
        let mut discovered = fs
            .enumerate_with_types(&self.path)
            .into_iter()
            .flatten()
            .filter_map(|(name, node_type)| {
                if matches!(node_type, FsNodeType::Directory) {
                    name.strip_suffix(".lproj").map(str::to_owned)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        discovered.sort();
        let development_region = self
            .plist
            .get("CFBundleDevelopmentRegion")
            .and_then(Value::as_string);
        merge_localizations(declared, discovered, development_region)
    }

    /// Canonical name for the bundle according to Info.plist
    pub fn canonical_bundle_name(&self) -> Option<&str> {
        self.plist
            .get("CFBundleName")
            .map(|name| name.as_string().unwrap())
    }

    /// Canonical bundle name when the metadata field is a string.
    pub fn canonical_bundle_name_optional(&self) -> Option<&str> {
        self.plist.get("CFBundleName").and_then(Value::as_string)
    }

    /// Name for the bundle, either the canonical name or, if there isn't one,
    /// the name this bundle has in the filesystem.
    pub fn bundle_name(&self) -> &str {
        self.path.file_name().unwrap().strip_suffix(".app").unwrap()
    }

    pub fn display_name(&self) -> &str {
        if let Some(display_name) = self.plist.get("CFBundleDisplayName") {
            display_name.as_string().unwrap()
        } else {
            ""
        }
    }

    /// Name shown on the home screen, as on iOS: `CFBundleDisplayName`, then
    /// `CFBundleName`, preferring values from the localized
    /// `InfoPlist.strings`, then the bundle's file name.
    pub fn localized_display_name(&self, fs: &Fs, languages: &[String]) -> String {
        let strings = self.localized_info_plist_strings(fs, languages);
        ["CFBundleDisplayName", "CFBundleName"]
            .iter()
            .find_map(|&key| {
                strings
                    .as_ref()
                    .and_then(|d| d.get(key))
                    .or_else(|| self.plist.get(key))
                    .and_then(|v| v.as_string())
            })
            .unwrap_or_else(|| self.bundle_name())
            .to_owned()
    }

    /// `InfoPlist.strings` for the first preferred language the bundle has,
    /// falling back to `CFBundleDevelopmentRegion`.
    fn localized_info_plist_strings(&self, fs: &Fs, languages: &[String]) -> Option<Dictionary> {
        let dev_region = self
            .plist
            .get("CFBundleDevelopmentRegion")
            .and_then(|v| v.as_string());
        let bytes = languages
            .iter()
            .map(String::as_str)
            .chain(dev_region)
            .flat_map(|lang| {
                let code = lang.split(['-', '_']).next().unwrap();
                LANG_ID_TO_LANG_PROJ
                    .iter()
                    .filter(move |&&(id, _)| id == code)
                    .flat_map(|&(_, lprojs)| lprojs.iter().map(|&l| l.to_owned()))
                    .chain([format!("{lang}.lproj")])
            })
            .find_map(|lproj| {
                fs.read(self.path.join(&lproj).join("InfoPlist.strings"))
                    .ok()
            })?;
        parse_strings(&bytes)
    }

    pub fn minimum_os_version(&self) -> Option<&str> {
        self.plist
            .get("MinimumOSVersion")
            .map(|v| v.as_string().unwrap())
    }

    /// Minimum OS version when the metadata field is a string.
    pub fn minimum_os_version_optional(&self) -> Option<&str> {
        self.plist.get("MinimumOSVersion").and_then(Value::as_string)
    }

    pub fn required_device_capabilities(&self) -> Vec<&str> {
        self.plist
            .get("UIRequiredDeviceCapabilities")
            .map(|v| {
                if let Some(dict) = v.as_dictionary() {
                    // TODO: support undesired capabilities
                    assert!(dict.values().all(|x| x.as_boolean().unwrap()));
                    dict.keys().map(|o| o.as_str()).collect()
                } else {
                    v.as_array()
                        .unwrap()
                        .iter()
                        .map(|o| o.as_string().unwrap())
                        .collect()
                }
            })
            .unwrap_or_default()
    }

    pub fn executable_path(&self) -> GuestPathBuf {
        // FIXME: Is this key optional? All iPhone apps seem to have it.
        self.path
            .join(self.plist["CFBundleExecutable"].as_string().unwrap())
    }

    pub fn launch_image_path(&self) -> GuestPathBuf {
        if let Some(base_name) = self.plist.get("UILaunchImageFile") {
            self.path
                .join(format!("{}.png", base_name.as_string().unwrap()))
        } else {
            self.path.join("Default.png") // not guaranteed to exist!
        }
    }

    pub fn status_bar_hidden(&self) -> bool {
        self.plist
            .get("UIStatusBarHidden")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false)
    }

    /// Font files the app bundles (`UIAppFonts`), relative to the bundle.
    pub fn app_fonts(&self) -> Vec<&str> {
        self.plist
            .get("UIAppFonts")
            .and_then(|v| v.as_array())
            .map(|fonts| fonts.iter().filter_map(|v| v.as_string()).collect())
            .unwrap_or_default()
    }

    fn find_icon(icon_files: &[plist::Value]) -> Option<&plist::Value> {
        ["Icon", "Icon-72"]
            .iter()
            .find_map(|icon| {
                icon_files.iter().find(|found_icon| {
                    found_icon
                        .as_string()
                        .is_some_and(|s| s.trim_end_matches(".png") == *icon)
                })
            })
            .or_else(|| icon_files.first())
    }

    fn icon_path(&self) -> Option<GuestPathBuf> {
        // TODO: Fix this to what an actual iOS device does,
        // including Retina icons and such when we get there.
        // Reference: https://developer.apple.com/library/archive/qa/qa1686/_index.html
        // We check for the icon in the following order:
        // 1. CFBundleIconFile,
        // 2. CFBundleIconFiles for Icon, Icon-72,
        // 3. CFBundleIconFiles[0],
        // 4. CFBundleIcons -> CFBundlePrimaryIcon -> CFBundleIconFiles
        //    for Icon, Icon-72,
        // 5. CFBundleIcons -> CFBundlePrimaryIcon -> CFBundleIconFiles[0],
        // 6. Failsafe Icon.png
        if let Some(filename) = self
            .plist
            .get("CFBundleIconFile")
            .or_else(|| {
                self.plist
                    .get("CFBundleIconFiles")
                    .and_then(|v| v.as_array())
                    .and_then(|a| Self::find_icon(a))
            })
            .or_else(|| {
                self.plist
                    .get("CFBundleIcons")
                    .and_then(|v| v.as_dictionary())
                    .and_then(|d| d.get("CFBundlePrimaryIcon"))
                    .and_then(|v| v.as_dictionary())
                    .and_then(|d| d.get("CFBundleIconFiles"))
                    .and_then(|v| v.as_array())
                    .and_then(|a| Self::find_icon(a))
            })
        {
            if filename
                .as_string()
                .unwrap()
                .to_lowercase()
                .ends_with(".png")
            {
                Some(self.path.join(filename.as_string().unwrap()))
            } else {
                let filename_with_extension = format!("{}.png", filename.as_string().unwrap());
                Some(self.path.join(filename_with_extension))
            }
        } else {
            None
        }
    }

    pub fn load_icon(&self, fs: &Fs) -> Result<Image, String> {
        self.load_icon_timed(fs).map(|(image, _)| image)
    }

    pub fn load_icon_timed(&self, fs: &Fs) -> Result<(Image, IconLoadTimings), String> {
        let mut timings = IconLoadTimings::default();
        let started = Instant::now();
        let icon_path = self.icon_path();
        timings.candidate_discovery = started.elapsed();
        timings.candidates_considered = if icon_path.is_some() { 1 } else { 2 };

        let started = Instant::now();
        let bytes = if let Some(path) = icon_path {
            fs.read(path)
                .map_err(|_| "Could not read icon file".to_string())?
        } else {
            // No explicit icon was specified in the plist; fall back to
            // checking the filesystem directly for Icon.png, then icon.png.
            fs.read(self.path.join("Icon.png"))
                .or_else(|_| fs.read(self.path.join("icon.png")))
                .map_err(|_| "Could not find app icon (Icon.png or icon.png)".to_string())?
        };
        timings.reads = started.elapsed();
        let started = Instant::now();
        let mut image = Image::from_bytes(&bytes)
            .map_err(|e| format!("Could not parse icon image: {e}"))?;
        timings.decoding = started.elapsed();
        let started = Instant::now();
        Self::round_icon(&mut image);
        timings.processing = started.elapsed();
        Ok((image, timings))
    }

    /// Like [Self::load_icon], but uses a bigger version of the same icon if
    /// there is one, e.g. an @2x or iPad icon. Whatever [Self::load_icon]
    /// picks stays the fallback, so its rules still apply.
    pub fn load_best_icon(&self, fs: &Fs) -> Result<Image, String> {
        self.load_best_icon_timed(fs).map(|(image, _)| image)
    }

    pub fn load_best_icon_timed(&self, fs: &Fs) -> Result<(Image, IconLoadTimings), String> {
        let (icon, mut timings) = self.load_icon_timed(fs)?;
        let (width, height) = icon.dimensions();
        if width != height {
            return Ok((icon, timings));
        }
        let mut best: Option<Image> = None;
        let started = Instant::now();
        let candidates = self.icon_candidates();
        timings.candidate_discovery += started.elapsed();
        timings.candidates_considered += candidates.len();
        for path in candidates {
            let started = Instant::now();
            let Ok(bytes) = fs.read(&path) else {
                timings.reads += started.elapsed();
                continue;
            };
            timings.reads += started.elapsed();
            let started = Instant::now();
            let Ok(image) = Image::from_bytes(&bytes) else {
                timings.decoding += started.elapsed();
                continue;
            };
            timings.decoding += started.elapsed();
            // Apps list all sorts of images as icons, so only take a bigger
            // square one, within reason.
            let (w, h) = image.dimensions();
            let biggest = best.as_ref().map_or(width, |b| b.dimensions().0);
            if w == h && w > biggest && w <= 1024 {
                best = Some(image);
            }
        }
        let Some(mut image) = best else {
            return Ok((icon, timings));
        };
        let started = Instant::now();
        Self::round_icon(&mut image);
        timings.processing += started.elapsed();
        Ok((image, timings))
    }

    /// Every file that could be the app's icon at some size: each icon named
    /// in the plist, with and without @2x, @3x, ~ipad and ~iphone (see
    /// QA1686), plus the Icon.png failsafe.
    fn icon_candidates(&self) -> Vec<GuestPathBuf> {
        fn files(v: Option<&plist::Value>) -> impl Iterator<Item = &str> {
            v.and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_string())
        }
        let mut names = vec!["Icon", "icon"];
        for device in ["", "~ipad"] {
            let key = format!("CFBundleIconFiles{device}");
            names.extend(files(self.plist.get(&key)));
            let key = format!("CFBundleIcons{device}");
            names.extend(files(
                self.plist
                    .get(&key)
                    .and_then(|v| v.as_dictionary())
                    .and_then(|d| d.get("CFBundlePrimaryIcon"))
                    .and_then(|v| v.as_dictionary())
                    .and_then(|d| d.get("CFBundleIconFiles")),
            ));
        }
        names.extend(
            self.plist
                .get("CFBundleIconFile")
                .and_then(|v| v.as_string()),
        );

        let mut paths = Vec::new();
        for name in names {
            let ext = name.len().checked_sub(4).filter(|&i| {
                name.get(i..)
                    .is_some_and(|ext| ext.eq_ignore_ascii_case(".png"))
            });
            let base = if let Some(i) = ext {
                paths.push(self.path.join(name));
                &name[..i]
            } else {
                name
            };
            // The scale goes before the device, e.g. Icon@2x~ipad.png.
            let base = base
                .strip_suffix("~ipad")
                .or_else(|| base.strip_suffix("~iphone"))
                .unwrap_or(base);
            for scale in ["", "@2x", "@3x"] {
                for device in ["", "~ipad", "~iphone"] {
                    paths.push(self.path.join(format!("{base}{scale}{device}.png")));
                }
            }
        }
        paths
    }

    fn round_icon(image: &mut Image) {
        // iPhone OS icons are 57px by 57px and the OS always applies a
        // 10px radius rounded corner (see e.g. documentation of
        // UIPrerenderedIcon). If the icon is larger for some reason,
        // let's scale to match.
        let corner_radius = (10.0 / 57.0) * (image.dimensions().0 as f32);
        image.round_corners(corner_radius, /* four_corners: */ true);
    }

    pub fn main_nib_filename(&self, device_family: Option<DeviceFamily>) -> Option<&str> {
        // TODO: extend this logic for all device-specific keys
        if let Some(device_family) = device_family {
            if device_family == DeviceFamily::iPhone
                && self.plist.get("NSMainNibFile~iphone").is_some()
            {
                return self
                    .plist
                    .get("NSMainNibFile~iphone")
                    .map(|v| v.as_string().unwrap());
            }
            if device_family == DeviceFamily::iPad && self.plist.get("NSMainNibFile~ipad").is_some()
            {
                return self
                    .plist
                    .get("NSMainNibFile~ipad")
                    .map(|v| v.as_string().unwrap());
            }
        }
        self.plist
            .get("NSMainNibFile")
            .map(|v| v.as_string().unwrap())
    }

    pub fn supported_interface_orientations(&self) -> Vec<&str> {
        // UIInterfaceOrientation (iPhone OS 2.0) is a single string
        // (or a comma separated list of strings).
        // UISupportedInterfaceOrientations (iOS 3.2) is an array of strings and
        // takes precedence.
        self.plist
            .get("UISupportedInterfaceOrientations")
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|o| o.as_string().unwrap())
                    .collect()
            })
            .unwrap_or_else(|| {
                if let Some(v) = self
                    .plist
                    .get("UIInterfaceOrientation") {
                    let str = v.as_string().unwrap();
                    if str.contains(',') {
                        log!("UIInterfaceOrientation is a comma separated list of strings ({}), splitting!", str);
                    }
                    str.split(',').collect()
                } else {
                    vec!["UIInterfaceOrientationPortrait"]
                }
            })
    }

    pub fn device_family_array(&self) -> Vec<DeviceFamily> {
        self.plist
            .get("UIDeviceFamily")
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|o| {
                        DeviceFamily::try_from(match o {
                            Value::Integer(i) => i.as_unsigned().unwrap(),
                            Value::String(s) => s.parse().unwrap(),
                            _ => unreachable!(),
                        })
                        .unwrap()
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![DeviceFamily::iPhone])
    }
}

fn merge_localizations(
    declared: impl IntoIterator<Item = String>,
    discovered: impl IntoIterator<Item = String>,
    development_region: Option<&str>,
) -> Vec<String> {
    let mut result = Vec::new();
    for localization in declared
        .into_iter()
        .chain(discovered)
        .chain(development_region.map(str::to_owned))
    {
        if !localization.is_empty() && !result.contains(&localization) {
            result.push(localization);
        }
    }
    result
}

/// Parses a `.strings` file: a binary or XML plist, or text in UTF-8 or
/// UTF-16 (with a BOM).
fn parse_strings(bytes: &[u8]) -> Option<Dictionary> {
    if let Ok(Value::Dictionary(dict)) = Value::from_reader(Cursor::new(bytes)) {
        return Some(dict);
    }
    let utf16 = |rest: &[u8], from_bytes: fn([u8; 2]) -> u16| {
        let units: Vec<u16> = rest.as_chunks().0.iter().map(|&c| from_bytes(c)).collect();
        String::from_utf16(&units).ok()
    };
    let text = match bytes {
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes)?,
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes)?,
        _ => {
            let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
            String::from_utf8(bytes.to_vec()).ok()?
        }
    };
    // Text .strings files are a plist dictionary without the braces. The
    // parser reads bytes as Latin-1, so escape anything that isn't ASCII.
    let text: String = text
        .encode_utf16()
        .map(|u| match u {
            0..0x80 => char::from(u as u8).to_string(),
            _ => format!("\\U{u:04x}"),
        })
        .collect();
    Value::from_reader_ascii(format!("{{{text}}}").as_bytes())
        .ok()?
        .into_dictionary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_244_bundle_localizations_merge_declared_lproj_and_development_region() {
        let result = merge_localizations(
            ["fr".to_owned(), "en".to_owned()],
            ["Base".to_owned(), "en".to_owned()],
            Some("de"),
        );
        assert_eq!(result, ["fr", "en", "Base", "de"]);
    }
}
