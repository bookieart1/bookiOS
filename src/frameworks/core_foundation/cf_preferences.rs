/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFPreferences`.
//!
//! According to Apple's docs, it's not toll-free bridged to `NSUserDefaults`,
//! but we are still implementing one atop of another.

use super::cf_string::CFStringRef;
use super::CFTypeRef;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::foundation::ns_string;
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;

type CFPropertyListRef = CFTypeRef;

fn copy_domain_keys(env: &mut Environment, domain: id) -> CFTypeRef {
    let keys: id = if domain == nil {
        msg_class![env; NSArray array]
    } else {
        msg![env; domain allKeys]
    };
    msg![env; keys copy]
}

fn CFPreferencesCopyAppValue(
    env: &mut Environment,
    key: CFStringRef,
    app_id: CFStringRef,
) -> CFPropertyListRef {
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    // TODO: handle other ids
    assert!(msg![env; app_id isEqualToString:current_app]);
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    let value: id = msg![env; user_defaults objectForKey:key];
    msg![env; value copy]
}

fn CFPreferencesSetAppValue(
    env: &mut Environment,
    key: CFStringRef,
    value: CFPropertyListRef,
    app_id: CFStringRef,
) {
    assert!(!value.is_null()); // TODO
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    // TODO: handle other ids
    assert!(msg![env; app_id isEqualToString:current_app]);
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    msg![env; user_defaults setObject:value forKey:key]
}

fn CFPreferencesAppSynchronize(env: &mut Environment, app_id: CFStringRef) -> bool {
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    // TODO: handle other ids
    assert!(msg![env; app_id isEqualToString:current_app]);
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    msg![env; user_defaults synchronize]
}

fn CFPreferencesCopyKeyList(
    env: &mut Environment,
    app_id: CFStringRef,
    user_name: CFStringRef,
    host_name: CFStringRef,
) -> CFTypeRef {
    let current_user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
    let any_host = ns_string::get_static_str(env, kCFPreferencesAnyHost);
    if app_id == nil
        || user_name == nil
        || host_name == nil
        || !msg![env; user_name isEqualToString:current_user]
        || !msg![env; host_name isEqualToString:any_host]
    {
        return nil;
    }

    let defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    let domain: id = if msg![env; app_id isEqualToString:current_app] {
        msg![env; defaults dictionaryRepresentation]
    } else {
        msg![env; defaults persistentDomainForName:app_id]
    };
    let keys: id = copy_domain_keys(env, domain);
    let count: u32 = msg![env; keys count];
    env.diagnostic_campaign_event(
        "preferences",
        "CFPreferencesCopyKeyList app,user,host,count",
        [
            app_id.to_bits(),
            user_name.to_bits(),
            host_name.to_bits(),
            count,
            0,
            0,
            0,
            0,
        ],
    );
    keys
}

pub const kCFPreferencesCurrentApplication: &str = "kCFPreferencesCurrentApplication";
pub const kCFPreferencesCurrentUser: &str = "kCFPreferencesCurrentUser";
pub const kCFPreferencesAnyHost: &str = "kCFPreferencesAnyHost";

pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFPreferencesCurrentApplication",
        HostConstant::NSString(kCFPreferencesCurrentApplication),
    ),
    (
        "_kCFPreferencesCurrentUser",
        HostConstant::NSString(kCFPreferencesCurrentUser),
    ),
    (
        "_kCFPreferencesAnyHost",
        HostConstant::NSString(kCFPreferencesAnyHost),
    ),
];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFPreferencesCopyAppValue(_, _)),
    export_c_func!(CFPreferencesSetAppValue(_, _, _)),
    export_c_func!(CFPreferencesAppSynchronize(_)),
    export_c_func!(CFPreferencesCopyKeyList(_, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::NSUInteger;
    use crate::objc::release;

    #[test]
    fn dev_248_copy_key_list_returns_owned_exact_domain_keys() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let domain: id = msg_class![env; NSMutableDictionary new];
        let key = ns_string::get_static_str(env, "zuma-test-key");
        let value = ns_string::get_static_str(env, "saved");
        () = msg![env; domain setObject:value forKey:key];
        let keys = copy_domain_keys(env, domain);
        assert_ne!(keys, nil);
        let count: NSUInteger = msg![env; keys count];
        assert!(count > 0);
        assert!(msg![env; keys containsObject:key]);
        release(env, keys);
        release(env, domain);
    }
}
