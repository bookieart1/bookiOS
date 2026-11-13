/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Logging and terminal output macros.

use std::fs::File;
use std::sync::{LazyLock, Mutex};

const LOG_SITE_INITIAL_BURST: u64 = 32;
const ECHO_SITE_INITIAL_BURST: u64 = 256;

type OutputSite = (&'static str, &'static str, &'static str, u32);

fn should_emit_occurrence(occurrence: u64, initial_burst: u64) -> bool {
    occurrence <= initial_burst || occurrence.is_power_of_two()
}

fn next_output_site_occurrence(site: OutputSite) -> u64 {
    static COUNTS: LazyLock<Mutex<std::collections::HashMap<OutputSite, u64>>> =
        LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
    let mut counts = COUNTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let count = counts.entry(site).or_default();
    *count = count.saturating_add(1);
    *count
}

/// Get a handle to the log file. This is only for use by logging macros!
///
/// All the logging macros print to stderr or (on Android) logcat, but this
/// is not convenient for users who aren't accustomed to command-line tools or
/// who don't have access to ADB, so we also write to a log file.
///
/// The log from the previous run is kept (see [crate::paths::log_files]),
/// replacing any older one, so there is only ever one previous log. On iOS
/// and Android, getting past a hang or black screen means relaunching, which
/// would otherwise overwrite the log of the run that went wrong.
/// A module's path without the crate's name (touchHLE), which users see in
/// the log.
pub fn module_name(path: &'static str) -> &'static str {
    match path {
        "touchHLE" => "iDared 32bit",
        _ => path.strip_prefix("touchHLE::").unwrap_or(path),
    }
}

pub fn get_log_file() -> &'static File {
    static LOG_FILE: LazyLock<File> = LazyLock::new(|| {
        let base_path = crate::paths::user_data_base_path();
        let [log, previous] = crate::paths::log_files();
        let path = base_path.join(log);
        // This replaces an existing previous log on all platforms (including
        // Windows, where Rust's rename replaces the destination too). It fails
        // harmlessly if there's no log yet, e.g. on the first run.
        let _ = std::fs::rename(&path, base_path.join(previous));
        File::create(path).unwrap()
    });

    &LOG_FILE
}

/// Emit one already-formatted line to the terminal and persistent log.
pub fn emit_line(formatted_str: &str) {
    #[cfg(target_os = "android")]
    {
        sdl2::log::log(formatted_str);
    }
    #[cfg(not(target_os = "android"))]
    eprintln!("{}", formatted_str);

    use std::io::Write;
    let mut log_file = get_log_file();
    let _ = log_file.write_all(formatted_str.as_bytes());
    let _ = log_file.write_all(b"\n");
}

/// Emit a warning/error call site with a bounded, deterministic repeat policy.
///
/// The first events retain ordinary detail. After that, powers-of-two samples
/// preserve evidence that the condition is still recurring without allowing a
/// tight guest loop to grow the user-facing log without bound.
pub fn emit_log_at_site(
    module: &'static str,
    file: &'static str,
    line: u32,
    message: std::fmt::Arguments<'_>,
) {
    let occurrence = next_output_site_occurrence(("log", module, file, line));
    if !should_emit_occurrence(occurrence, LOG_SITE_INITIAL_BURST) {
        return;
    }
    if occurrence <= LOG_SITE_INITIAL_BURST {
        emit_line(&format!("{}: {}", module_name(module), message));
    } else {
        emit_line(&format!(
            "{}: [call-site occurrence #{occurrence}; intermediate repeats suppressed] {}",
            module_name(module),
            message
        ));
    }
}

/// Emit direct user-facing or diagnostic output with a larger per-call-site
/// burst than ordinary warnings, followed by the same exponential sampling.
pub fn emit_echo_at_site(file: &'static str, line: u32, message: std::fmt::Arguments<'_>) {
    let occurrence = next_output_site_occurrence(("echo", "", file, line));
    if !should_emit_occurrence(occurrence, ECHO_SITE_INITIAL_BURST) {
        return;
    }
    if occurrence <= ECHO_SITE_INITIAL_BURST {
        emit_line(&message.to_string());
    } else {
        emit_line(&format!(
            "[output call-site occurrence #{occurrence}; intermediate repeats suppressed] {message}"
        ));
    }
}

/// Prints a log message unconditionally. Use this for errors or warnings.
///
/// The message is prefixed with the module path, so it is clear where it comes
/// from.
macro_rules! log {
    ($($arg:tt)+) => {
        $crate::log::emit_log_at_site(
            module_path!(),
            file!(),
            line!(),
            format_args!($($arg)+),
        );
    }
}

/// Same as [log], but silently fails on panic instead of
/// panicking.
macro_rules! log_no_panic {
    ($($arg:tt)+) => {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $crate::log::emit_log_at_site(
                module_path!(),
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }));
    }
}

/// Like [log], but prints the message only if debugging is enabled for the
/// module where it is used. This can be used for verbose things only needed
/// when debugging.
macro_rules! log_dbg {
    ($($arg:tt)+) => {
        if $crate::log::ENABLED_MODULES.contains(&module_path!()) {
            log!($($arg)*);
        }
    }
}

/// Like [log], but messages only log once and cannot have formatting.
/// To be used for log messages that are known to spam the log file (like those
/// logged every frame).
macro_rules! log_once {
    ($msg:literal) => {{
        static LOG_ONCE: std::sync::Once = std::sync::Once::new();
        LOG_ONCE.call_once(|| {
            log!("{} [this log will only be shown once]", $msg);
        });
    }};
}

/// Print a message (with implicit newline). This should be used for all
/// touchHLE output that isn't coming from the app itself.
///
/// Prefer use [log] or [log_dbg] for errors and warnings during emulation.
macro_rules! echo {
    ($($arg:tt)+) => {
        {
            $crate::log::emit_echo_at_site(
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
    () => {
        {
            $crate::log::emit_echo_at_site(file!(), line!(), format_args!(""));
        }
    }
}

/// Same as [echo], but silently fails on panic instead of
/// panicking.
macro_rules! echo_no_panic {
    ($($arg:tt)*) => {
        {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                $crate::log::emit_echo_at_site(
                    file!(),
                    line!(),
                    format_args!($($arg)*),
                );
            }));
        }
    }
}

/// Put modules to enable [log_dbg] for here, e.g. "touchHLE::mem" to see when
/// memory is allocated and freed.
pub const ENABLED_MODULES: &[&str] = &[];

#[cfg(test)]
#[test]
fn test_module_name() {
    assert_eq!(
        module_name("touchHLE::frameworks::uikit"),
        "frameworks::uikit"
    );
    assert_eq!(module_name("touchHLE"), "iDared 32bit");
    assert_eq!(module_name("other_crate::thing"), "other_crate::thing");
    assert_eq!(module_name("touchHLE_version"), "touchHLE_version");
}

#[cfg(test)]
#[test]
fn development_256_repeated_log_call_sites_are_bounded_and_sampled() {
    assert!((1..=LOG_SITE_INITIAL_BURST)
        .all(|count| should_emit_occurrence(count, LOG_SITE_INITIAL_BURST)));
    assert!(!should_emit_occurrence(
        LOG_SITE_INITIAL_BURST + 1,
        LOG_SITE_INITIAL_BURST
    ));
    assert!(!should_emit_occurrence(63, LOG_SITE_INITIAL_BURST));
    assert!(should_emit_occurrence(64, LOG_SITE_INITIAL_BURST));
    assert!(!should_emit_occurrence(65, LOG_SITE_INITIAL_BURST));
    assert!(should_emit_occurrence(1 << 20, LOG_SITE_INITIAL_BURST));

    let emitted_through_doodle_grub_count = (1..=720_639)
        .filter(|&count| should_emit_occurrence(count, LOG_SITE_INITIAL_BURST))
        .count();
    assert_eq!(emitted_through_doodle_grub_count, 46);

    let emitted_direct_output_through_same_count = (1..=720_639)
        .filter(|&count| should_emit_occurrence(count, ECHO_SITE_INITIAL_BURST))
        .count();
    assert_eq!(emitted_direct_output_through_same_count, 267);

    let flooded_site = ("test-log", "test-module", "log-volume-test.rs", 1);
    let independent_terminal_site = ("test-log", "test-module", "log-volume-test.rs", 2);
    for expected in 1..=720_639 {
        assert_eq!(next_output_site_occurrence(flooded_site), expected);
    }
    let terminal_occurrence = next_output_site_occurrence(independent_terminal_site);
    assert_eq!(terminal_occurrence, 1);
    assert!(should_emit_occurrence(
        terminal_occurrence,
        LOG_SITE_INITIAL_BURST
    ));
}
