/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSProcessInfo`.

use super::{NSTimeInterval, NSUInteger};
use crate::frameworks::foundation::ns_string;
use crate::libc::mach::host::PHYSICAL_MEMORY;
use crate::objc::{autorelease, id, msg, msg_class, objc_classes, ClassExports};
use crate::window::DeviceFamily;
use crate::Environment;
use std::time::Instant;
use uuid::Uuid;

#[derive(Default)]
pub struct State {
    /// `NSProcessInfo*`
    process_info: Option<id>,
}

fn assert_process_info_singleton(env: &mut Environment, this: id) {
    assert_eq!(
        this,
        env.framework_state
            .foundation
            .ns_process_info
            .process_info
            .unwrap()
    );
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSProcessInfo: NSObject

+ (id)processInfo {
    if let Some(existing) = env.framework_state.foundation.ns_process_info.process_info {
        existing
    } else {
        let process_info: id = msg![env; this new];
        env.framework_state.foundation.ns_process_info.process_info = Some(process_info);
        process_info
    }
}

- (NSTimeInterval)systemUptime {
    assert_process_info_singleton(env, this); // TODO
    Instant::now().duration_since(env.startup_time).as_secs_f64()
}

- (u64)physicalMemory {
    assert_process_info_singleton(env, this); // TODO
    PHYSICAL_MEMORY.into()
}

- (NSUInteger)processorCount {
    assert_process_info_singleton(env, this); // TODO
    // Just returning 1 for now which matches early iPhones
    1
}

- (id)operatingSystemVersionString {
    assert_process_info_singleton(env, this); // TODO
    // Keep this consistent with UIDevice's emulated systemVersion.
    let version = match env.options.device_family {
        Some(DeviceFamily::iPad) => "Version 3.2 (Build 7B367)",
        _ => "Version 2.0 (Build 5A347)",
    };
    ns_string::get_static_str(env, version)
}

- (id)processName {
    // This function probably just needs to return a unique value
    // Testing on macOS appears CFBundleName is used
    assert_process_info_singleton(env, this); // TODO
    let main_bundle: id = msg_class![env; NSBundle mainBundle];
    let name_key: id = ns_string::get_static_str(env, "CFBundleName");
    msg![env; main_bundle objectForInfoDictionaryKey:name_key]
}

- (id)environment {
    assert_process_info_singleton(env, this);
    // Expose the emulated process environment, never the host process's
    // environment. HOME and TMPDIR are established by Environment startup and
    // setenv()/unsetenv() updates are reflected on later calls.
    let entries: Vec<(String, String)> = env
        .env_vars
        .iter()
        .map(|(name, &value)| {
            (
                String::from_utf8_lossy(name).into_owned(),
                env.mem.cstr_at_utf8(value).unwrap_or("").to_owned(),
            )
        })
        .collect();
    let mutable: id = msg_class![env; NSMutableDictionary dictionary];
    for (name, value) in entries {
        let name = ns_string::from_rust_string(env, name);
        let value = ns_string::from_rust_string(env, value);
        () = msg![env; mutable setObject:value forKey:name];
        crate::objc::release(env, value);
        crate::objc::release(env, name);
    }
    msg_class![env; NSDictionary dictionaryWithDictionary:mutable]
}

- (id)globallyUniqueString {
    assert_process_info_singleton(env, this);
    // NSProcessInfo promises a different, globally unique value for each
    // request, not a stable device identifier. A fresh RFC 4122 v4 value
    // supplies those semantics without exposing host or device identity.
    let value = Uuid::new_v4().hyphenated().to_string().to_uppercase();
    let value = ns_string::from_rust_string(env, value);
    autorelease(env, value)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::{msg, msg_class, release};

    #[test]
    fn environment_reports_emulated_variables_without_host_leakage() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let home = env.mem.alloc_and_write_cstr(b"/guest/home");
        env.env_vars.insert(b"HOME".to_vec(), home);
        let process: id = msg_class![env; NSProcessInfo processInfo];
        let dictionary: id = msg![env; process environment];
        let key = ns_string::from_rust_string(env, "HOME".to_owned());
        let value: id = msg![env; dictionary objectForKey:key];
        assert_eq!(ns_string::to_rust_string(env, value), "/guest/home");
        release(env, key);
    }
}
