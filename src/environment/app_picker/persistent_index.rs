//! Disposable cross-process metadata/icon cache for the native iDared picker.
//!
//! This cache is never runtime authority. Every selected package is reopened by
//! `Environment::new` before guest execution; these records only avoid
//! repeating picker discovery for an unchanged package.

use super::{AppInfo, LibraryEntryFingerprint};
use crate::image::Image;
use crate::paths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(super) const INDEX_SCHEMA_VERSION: u32 = 1;
const PICKER_METADATA_VERSION: u32 = 1;
const CACHE_DIRECTORY: &str = "iDaredPickerIndex";
const INDEX_FILENAME: &str = "library-index.binary.plist";
const ICON_DIRECTORY: &str = "icons-v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct IndexFile {
    schema_version: u32,
    picker_metadata_version: u32,
    preferred_languages: Vec<String>,
    packages: Vec<CachedPackage>,
    integrity_sha256: String,
}

#[derive(Serialize)]
struct IndexIntegrity<'a> {
    schema_version: u32,
    picker_metadata_version: u32,
    preferred_languages: &'a [String],
    packages: &'a [CachedPackage],
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CachedPackage {
    fingerprint: LibraryEntryFingerprint,
    inspection_succeeded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    app: Option<CachedApp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CachedApp {
    bundle_identifier: String,
    display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    minimum_os_version: Option<String>,
    compatibility: String,
    had_icon: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_cache_key: Option<String>,
}

pub(super) struct SourcePackage {
    pub(super) path: PathBuf,
    pub(super) relative_path: String,
    /// `None` means the package can still be inspected for this launch, but it
    /// is deliberately not reused or persisted because its identity was not
    /// safe to establish.
    pub(super) fingerprint: Option<LibraryEntryFingerprint>,
}

pub(super) struct InspectedApp {
    pub(super) app: AppInfo,
    pub(super) canonical_name: Option<String>,
    pub(super) bundle_version: Option<String>,
    pub(super) minimum_os_version: Option<String>,
}

pub(super) enum InspectionResult {
    App(InspectedApp),
    Rejected(String),
}

#[derive(Default)]
pub(super) struct ReconcileStats {
    pub(super) index_state: String,
    pub(super) index_existed: bool,
    pub(super) index_read: Duration,
    pub(super) index_decode: Duration,
    pub(super) reconciliation: Duration,
    pub(super) icon_cache: Duration,
    pub(super) index_write: Duration,
    pub(super) successful_hits: usize,
    pub(super) negative_hits: usize,
    pub(super) new_misses: usize,
    pub(super) changed_misses: usize,
    pub(super) uncacheable_misses: usize,
    pub(super) icon_misses: usize,
    pub(super) inspected_successes: usize,
    pub(super) inspected_failures: usize,
    pub(super) deleted_entries: usize,
    pub(super) icons_removed: usize,
    pub(super) persisted_entries: usize,
    pub(super) write_error: Option<String>,
}

pub(super) struct ReconcileResult {
    pub(super) apps: Vec<AppInfo>,
    pub(super) stats: ReconcileStats,
}

pub(super) fn default_cache_root() -> PathBuf {
    let documents = paths::user_data_base_path().into_owned();
    #[cfg(target_os = "ios")]
    {
        // `user_data_base_path()` is the application container's Documents
        // directory on iOS. Keep disposable launcher state in Library/Caches,
        // separate from imported packages and guest save sandboxes.
        return documents
            .parent()
            .unwrap_or(&documents)
            .join("Library")
            .join("Caches")
            .join(CACHE_DIRECTORY);
    }
    #[cfg(not(target_os = "ios"))]
    documents.join(format!(".{CACHE_DIRECTORY}"))
}

fn index_path(cache_root: &Path) -> PathBuf {
    cache_root.join(INDEX_FILENAME)
}

fn icon_directory(cache_root: &Path) -> PathBuf {
    cache_root.join(ICON_DIRECTORY)
}

fn valid_index(index: &IndexFile) -> Result<(), &'static str> {
    if index.schema_version != INDEX_SCHEMA_VERSION {
        return Err("schema-mismatch");
    }
    if index.picker_metadata_version != PICKER_METADATA_VERSION {
        return Err("picker-metadata-mismatch");
    }
    let mut paths = HashSet::new();
    for package in &index.packages {
        if !paths.insert(&package.fingerprint.relative_path) {
            return Err("duplicate-package-path");
        }
        let valid_outcome = if package.inspection_succeeded {
            package.app.is_some() && package.failure.is_none()
        } else {
            package.app.is_none() && package.failure.is_some()
        };
        if !valid_outcome {
            return Err("invalid-package-outcome");
        }
        if let Some(app) = package.app.as_ref() {
            if !app.had_icon && app.icon_cache_key.is_some() {
                return Err("invalid-icon-outcome");
            }
            if app.icon_cache_key.as_deref().is_some_and(|key| {
                key.len() != 68
                    || !key.ends_with(".png")
                    || !key[..64].bytes().all(|byte| byte.is_ascii_hexdigit())
            }) {
                return Err("invalid-icon-reference");
            }
        }
    }
    Ok(())
}

fn index_integrity(index: &IndexFile) -> Result<String, String> {
    let payload = IndexIntegrity {
        schema_version: index.schema_version,
        picker_metadata_version: index.picker_metadata_version,
        preferred_languages: &index.preferred_languages,
        packages: &index.packages,
    };
    let mut bytes = Vec::new();
    plist::to_writer_binary(&mut bytes, &payload)
        .map_err(|error| format!("couldn't encode index integrity payload: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn load_index(
    cache_root: &Path,
    preferred_languages: &[String],
    stats: &mut ReconcileStats,
) -> Option<IndexFile> {
    let path = index_path(cache_root);
    stats.index_existed = path.is_file();
    if !stats.index_existed {
        stats.index_state = "missing".to_owned();
        return None;
    }

    let started = Instant::now();
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            stats.index_read = started.elapsed();
            stats.index_state = format!("read-error ({error})");
            return None;
        }
    };
    stats.index_read = started.elapsed();

    let started = Instant::now();
    let index: IndexFile = match plist::from_bytes(&bytes) {
        Ok(index) => index,
        Err(error) => {
            stats.index_decode = started.elapsed();
            stats.index_state = format!("corrupt ({error})");
            return None;
        }
    };
    stats.index_decode = started.elapsed();

    if let Err(reason) = valid_index(&index) {
        stats.index_state = reason.to_owned();
        return None;
    }
    match index_integrity(&index) {
        Ok(integrity) if integrity == index.integrity_sha256 => {}
        Ok(_) => {
            stats.index_state = "integrity-mismatch".to_owned();
            return None;
        }
        Err(error) => {
            stats.index_state = format!("integrity-error ({error})");
            return None;
        }
    }
    if index.preferred_languages != preferred_languages {
        stats.index_state = "preferred-languages-changed".to_owned();
        return None;
    }
    stats.index_state = "loaded".to_owned();
    Some(index)
}

fn icon_key(fingerprint: &LibraryEntryFingerprint) -> String {
    let mut digest = Sha256::new();
    digest.update(INDEX_SCHEMA_VERSION.to_le_bytes());
    digest.update(fingerprint.relative_path.as_bytes());
    digest.update(fingerprint.package_type.as_bytes());
    digest.update([fingerprint.file_kind]);
    digest.update(fingerprint.len.to_le_bytes());
    digest.update(fingerprint.modified_secs.to_le_bytes());
    digest.update(fingerprint.modified_nanos.to_le_bytes());
    digest.update(fingerprint.device_id.to_le_bytes());
    digest.update(fingerprint.file_id.to_le_bytes());
    digest.update(fingerprint.directory_entries.to_le_bytes());
    digest.update(fingerprint.directory_bytes.to_le_bytes());
    digest.update(fingerprint.directory_metadata_sha256.as_bytes());
    format!("{:x}.png", digest.finalize())
}

fn truncate_failure(error: &str) -> String {
    const LIMIT: usize = 512;
    let mut chars = error.chars();
    let mut result: String = chars.by_ref().take(LIMIT).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

fn atomic_write(
    path: &Path,
    write: impl FnOnce(&mut File) -> Result<(), String>,
) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("cache path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("couldn't create {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("cache"),
        std::process::id()
    ));
    let result = (|| {
        let mut file = File::create(&temporary)
            .map_err(|error| format!("couldn't create {}: {error}", temporary.display()))?;
        write(&mut file)?;
        file.flush()
            .map_err(|error| format!("couldn't flush {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("couldn't sync {}: {error}", temporary.display()))?;
        // Windows will not replace/rename an open file. Closing explicitly is
        // also a clearer durability boundary before the atomic install on iOS.
        drop(file);
        #[cfg(windows)]
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|error| format!("couldn't replace {}: {error}", path.display()))?;
        }
        std::fs::rename(&temporary, path).map_err(|error| {
            format!(
                "couldn't atomically install {} as {}: {error}",
                temporary.display(),
                path.display()
            )
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn write_icon(cache_root: &Path, key: &str, icon: &Image) -> Result<(), String> {
    let png = icon
        .to_png()
        .ok_or_else(|| "couldn't encode processed picker icon as PNG".to_owned())?;
    atomic_write(&icon_directory(cache_root).join(key), |file| {
        file.write_all(&png)
            .map_err(|error| format!("couldn't write icon cache: {error}"))
    })
}

fn load_icon(cache_root: &Path, key: &str) -> Result<Image, String> {
    let path = icon_directory(cache_root).join(key);
    let mut bytes = Vec::new();
    File::open(&path)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|error| format!("couldn't read {}: {error}", path.display()))?;
    Image::from_bytes(&bytes)
        .map_err(|error| format!("couldn't decode {}: {error}", path.display()))
}

fn write_index(cache_root: &Path, index: &IndexFile) -> Result<(), String> {
    let mut index = index.clone();
    index.integrity_sha256 = index_integrity(&index)?;
    atomic_write(&index_path(cache_root), |file| {
        plist::to_writer_binary(file, &index)
            .map_err(|error| format!("couldn't encode persistent picker index: {error}"))
    })
}

fn clean_unused_icons(cache_root: &Path, referenced: &HashSet<String>) -> Result<usize, String> {
    let directory = icon_directory(cache_root);
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Ok(0);
    };
    let mut removed = 0;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "couldn't enumerate icon cache {}: {error}",
                directory.display()
            )
        })?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.ends_with(".png") && !referenced.contains(&name) {
            std::fs::remove_file(entry.path()).map_err(|error| {
                format!(
                    "couldn't remove stale icon {}: {error}",
                    entry.path().display()
                )
            })?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub(super) fn reconcile(
    cache_root: &Path,
    sources: Vec<SourcePackage>,
    preferred_languages: &[String],
    mut inspect: impl FnMut(&Path) -> InspectionResult,
) -> ReconcileResult {
    let mut stats = ReconcileStats::default();
    let old_index = load_index(cache_root, preferred_languages, &mut stats);
    let mut old_by_path: HashMap<String, CachedPackage> = old_index
        .map(|index| {
            index
                .packages
                .into_iter()
                .map(|package| (package.fingerprint.relative_path.clone(), package))
                .collect()
        })
        .unwrap_or_default();
    let started = Instant::now();
    let mut apps = Vec::new();
    let mut packages = Vec::new();

    for source in sources {
        let old = old_by_path.remove(&source.relative_path);
        if let (Some(fingerprint), Some(cached)) = (&source.fingerprint, old.as_ref()) {
            if &cached.fingerprint == fingerprint {
                if cached.inspection_succeeded {
                    if let Some(cached_app) = cached.app.as_ref() {
                        let icon_started = Instant::now();
                        let icon = cached_app.had_icon.then(|| {
                            cached_app
                                .icon_cache_key
                                .as_deref()
                                .ok_or_else(|| "cached icon reference is missing".to_owned())
                                .and_then(|key| load_icon(cache_root, key))
                        });
                        stats.icon_cache += icon_started.elapsed();
                        let icon = match icon {
                            Some(Ok(icon)) => Some(icon),
                            Some(Err(_)) => {
                                stats.icon_misses += 1;
                                None
                            }
                            None => None,
                        };
                        if !cached_app.had_icon || icon.is_some() {
                            apps.push(AppInfo {
                                path: source.path.clone(),
                                bundle_identifier: cached_app.bundle_identifier.clone(),
                                display_name: cached_app.display_name.clone(),
                                compatibility: cached_app.compatibility.clone(),
                                icon,
                                display_name_ns_string: None,
                                icon_ui_image: None,
                            });
                            packages.push(cached.clone());
                            stats.successful_hits += 1;
                            continue;
                        }
                    }
                } else {
                    packages.push(cached.clone());
                    stats.negative_hits += 1;
                    continue;
                }
            }
        }

        match (&source.fingerprint, old) {
            (None, _) => stats.uncacheable_misses += 1,
            (Some(_), Some(_)) => stats.changed_misses += 1,
            (Some(_), None) => stats.new_misses += 1,
        }

        match inspect(&source.path) {
            InspectionResult::App(inspected) => {
                stats.inspected_successes += 1;
                let had_icon = inspected.app.icon.is_some();
                let mut icon_cache_key = None;
                if let (Some(fingerprint), Some(icon)) =
                    (source.fingerprint.as_ref(), inspected.app.icon.as_ref())
                {
                    let key = icon_key(fingerprint);
                    let icon_started = Instant::now();
                    if write_icon(cache_root, &key, icon).is_ok() {
                        icon_cache_key = Some(key);
                    }
                    stats.icon_cache += icon_started.elapsed();
                }
                if let Some(fingerprint) = source.fingerprint.clone() {
                    packages.push(CachedPackage {
                        fingerprint,
                        inspection_succeeded: true,
                        app: Some(CachedApp {
                            bundle_identifier: inspected.app.bundle_identifier.clone(),
                            display_name: inspected.app.display_name.clone(),
                            canonical_name: inspected.canonical_name,
                            bundle_version: inspected.bundle_version,
                            minimum_os_version: inspected.minimum_os_version,
                            compatibility: inspected.app.compatibility.clone(),
                            had_icon,
                            icon_cache_key,
                        }),
                        failure: None,
                    });
                }
                apps.push(inspected.app);
            }
            InspectionResult::Rejected(error) => {
                stats.inspected_failures += 1;
                if let Some(fingerprint) = source.fingerprint {
                    packages.push(CachedPackage {
                        fingerprint,
                        inspection_succeeded: false,
                        app: None,
                        failure: Some(truncate_failure(&error)),
                    });
                }
            }
        }
    }

    stats.deleted_entries = old_by_path.len();
    stats.reconciliation = started.elapsed();
    stats.persisted_entries = packages.len();

    let index = IndexFile {
        schema_version: INDEX_SCHEMA_VERSION,
        picker_metadata_version: PICKER_METADATA_VERSION,
        preferred_languages: preferred_languages.to_vec(),
        packages,
        integrity_sha256: String::new(),
    };
    let write_started = Instant::now();
    if let Err(error) = write_index(cache_root, &index) {
        stats.write_error = Some(error);
    } else {
        let referenced: HashSet<String> = index
            .packages
            .iter()
            .filter_map(|package| package.app.as_ref())
            .filter_map(|app| app.icon_cache_key.clone())
            .collect();
        match clean_unused_icons(cache_root, &referenced) {
            Ok(removed) => stats.icons_removed = removed,
            Err(error) => stats.write_error = Some(error),
        }
    }
    stats.index_write = write_started.elapsed();

    ReconcileResult { apps, stats }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    fn temp_directory(name: &str) -> PathBuf {
        // Keep rename/replace tests inside the active checkout. Some managed
        // Windows environments expose a readable temp directory but deny
        // atomic renames within it.
        let path = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("idared-index-tests")
            .join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn fingerprint(path: &str, len: u64) -> LibraryEntryFingerprint {
        LibraryEntryFingerprint {
            relative_path: path.to_owned(),
            package_type: Path::new(path)
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase(),
            file_kind: 1,
            len,
            modified_secs: 100,
            modified_nanos: 50,
            device_id: 1,
            file_id: len,
            directory_entries: 0,
            directory_bytes: 0,
            directory_metadata_sha256: String::new(),
        }
    }

    fn source(root: &Path, name: &str, len: u64) -> SourcePackage {
        let relative_path = format!("Games/{name}");
        SourcePackage {
            path: root.join(name),
            fingerprint: Some(fingerprint(&relative_path, len)),
            relative_path,
        }
    }

    fn app(path: &Path, name: &str) -> InspectionResult {
        InspectionResult::App(InspectedApp {
            app: AppInfo {
                path: path.to_path_buf(),
                bundle_identifier: format!("com.example.{name}"),
                display_name: name.to_owned(),
                compatibility: "needsWork".to_owned(),
                icon: Some(Image::from_pixel_vec(vec![255; 16], (2, 2))),
                display_name_ns_string: None,
                icon_ui_image: None,
            },
            canonical_name: Some(name.to_owned()),
            bundle_version: Some("1".to_owned()),
            minimum_os_version: Some("3.0".to_owned()),
        })
    }

    #[test]
    fn index_creation_reuse_negative_cache_and_incremental_changes() {
        let root = temp_directory("reconcile");
        let languages = vec!["en".to_owned()];
        let sources = vec![source(&root, "Good.ipa", 10), source(&root, "Bad.ipa", 20)];
        let mut inspected = 0;
        let first = reconcile(&root, sources, &languages, |path| {
            inspected += 1;
            if path.ends_with("Bad.ipa") {
                InspectionResult::Rejected("damaged archive".to_owned())
            } else {
                app(path, "Good")
            }
        });
        assert_eq!(inspected, 2);
        assert_eq!(first.apps.len(), 1);
        assert_eq!(first.stats.inspected_successes, 1);
        assert_eq!(first.stats.inspected_failures, 1);
        assert!(
            index_path(&root).is_file(),
            "index write failed: {:?}",
            first.stats.write_error
        );

        inspected = 0;
        let second = reconcile(
            &root,
            vec![source(&root, "Good.ipa", 10), source(&root, "Bad.ipa", 20)],
            &languages,
            |_| {
                inspected += 1;
                panic!("unchanged success and failure records must be reused")
            },
        );
        assert_eq!(inspected, 0);
        assert_eq!(second.stats.successful_hits, 1);
        assert_eq!(second.stats.negative_hits, 1);
        assert_eq!(second.apps[0].path, root.join("Good.ipa"));

        inspected = 0;
        let changed = reconcile(
            &root,
            vec![
                source(&root, "Good.ipa", 11),
                source(&root, "Bad.ipa", 21),
                source(&root, "New.ipa", 30),
            ],
            &languages,
            |path| {
                inspected += 1;
                app(path, path.file_stem().unwrap().to_str().unwrap())
            },
        );
        assert_eq!(inspected, 3);
        assert_eq!(changed.stats.changed_misses, 2);
        assert_eq!(changed.stats.new_misses, 1);
        assert_eq!(changed.apps.len(), 3);
        assert_eq!(changed.stats.negative_hits, 0);

        let deleted = reconcile(
            &root,
            vec![source(&root, "New.ipa", 30)],
            &languages,
            |_| panic!("unchanged remaining package must be reused"),
        );
        assert_eq!(deleted.stats.deleted_entries, 2);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_icon_corrupt_index_and_schema_mismatch_reinspect_safely() {
        let root = temp_directory("recovery");
        let languages = vec!["en".to_owned()];
        let packages = || vec![source(&root, "Game.ipa", 10)];
        reconcile(&root, packages(), &languages, |path| app(path, "Game"));

        for entry in std::fs::read_dir(icon_directory(&root)).unwrap() {
            std::fs::remove_file(entry.unwrap().path()).unwrap();
        }
        let mut inspected = 0;
        let missing_icon = reconcile(&root, packages(), &languages, |path| {
            inspected += 1;
            app(path, "Game")
        });
        assert_eq!(inspected, 1);
        assert_eq!(missing_icon.stats.icon_misses, 1);

        std::fs::write(index_path(&root), b"truncated").unwrap();
        inspected = 0;
        let corrupt = reconcile(&root, packages(), &languages, |path| {
            inspected += 1;
            app(path, "Game")
        });
        assert_eq!(inspected, 1);
        assert!(corrupt.stats.index_state.starts_with("corrupt"));

        let incompatible = IndexFile {
            schema_version: INDEX_SCHEMA_VERSION + 1,
            picker_metadata_version: PICKER_METADATA_VERSION,
            preferred_languages: languages.clone(),
            packages: Vec::new(),
            integrity_sha256: String::new(),
        };
        write_index(&root, &incompatible).unwrap();
        inspected = 0;
        let schema = reconcile(&root, packages(), &languages, |path| {
            inspected += 1;
            app(path, "Game")
        });
        assert_eq!(inspected, 1);
        assert_eq!(schema.stats.index_state, "schema-mismatch");

        let mut altered: IndexFile =
            plist::from_file(index_path(&root)).expect("read the generated index");
        altered.packages[0]
            .app
            .as_mut()
            .expect("successful cached app")
            .display_name = "silently altered".to_owned();
        plist::to_file_binary(index_path(&root), &altered)
            .expect("write a structurally valid but altered index");
        inspected = 0;
        let integrity = reconcile(&root, packages(), &languages, |path| {
            inspected += 1;
            app(path, "Game")
        });
        assert_eq!(inspected, 1);
        assert_eq!(integrity.stats.index_state, "integrity-mismatch");

        inspected = 0;
        let different_languages = reconcile(&root, packages(), &["fr".to_owned()], |path| {
            inspected += 1;
            app(path, "Jeu")
        });
        assert_eq!(inspected, 1);
        assert_eq!(
            different_languages.stats.index_state,
            "preferred-languages-changed"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
