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

fn is_current_application(env: &mut Environment, app_id: CFStringRef) -> bool {
    if app_id == nil {
        return false;
    }
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    if msg![env; app_id isEqualToString:current_app] {
        return true;
    }
    ns_string::to_rust_string(env, app_id) == env.bundle.bundle_identifier()
}

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
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    let value: id = if is_current_application(env, app_id) {
        msg![env; user_defaults objectForKey:key]
    } else {
        let domain: id = msg![env; user_defaults persistentDomainForName:app_id];
        msg![env; domain objectForKey:key]
    };
    let current = is_current_application(env, app_id);
    env.diagnostic_campaign_event(
        "preferences",
        "CFPreferencesCopyAppValue key,app,value,current",
        [
            key.to_bits(),
            app_id.to_bits(),
            value.to_bits(),
            u32::from(current),
            0,
            0,
            0,
            0,
        ],
    );
    msg![env; value copy]
}

fn CFPreferencesSetAppValue(
    env: &mut Environment,
    key: CFStringRef,
    value: CFPropertyListRef,
    app_id: CFStringRef,
) {
    assert!(app_id != nil);
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    if is_current_application(env, app_id) {
        if value == nil {
            msg![env; user_defaults removeObjectForKey:key]
        } else {
            msg![env; user_defaults setObject:value forKey:key]
        }
    } else {
        let existing: id = msg![env; user_defaults persistentDomainForName:app_id];
        let domain: id = if existing == nil {
            msg_class![env; NSMutableDictionary new]
        } else {
            msg![env; existing mutableCopy]
        };
        if value == nil {
            () = msg![env; domain removeObjectForKey:key];
        } else {
            () = msg![env; domain setObject:value forKey:key];
        }
        () = msg![env; user_defaults setPersistentDomain:domain forName:app_id];
        crate::objc::release(env, domain);
    }
}

fn CFPreferencesSetValue(
    env: &mut Environment,
    key: CFStringRef,
    value: CFPropertyListRef,
    app_id: CFStringRef,
    user_name: CFStringRef,
    host_name: CFStringRef,
) {
    let current_user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
    let any_host = ns_string::get_static_str(env, kCFPreferencesAnyHost);
    assert!(key != nil);
    assert!(app_id != nil);
    assert!(user_name != nil && msg![env; user_name isEqualToString:current_user]);
    assert!(host_name != nil && msg![env; host_name isEqualToString:any_host]);

    // NSUserDefaults is touchHLE's existing durable store for the current
    // application's current-user/any-host preferences domain. Do not silently
    // redirect other CFPreferences domains into that store.
    CFPreferencesSetAppValue(env, key, value, app_id)
}

fn CFPreferencesAppSynchronize(env: &mut Environment, app_id: CFStringRef) -> bool {
    assert!(app_id != nil);
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    msg![env; user_defaults synchronize]
}

fn CFPreferencesSynchronize(
    env: &mut Environment,
    app_id: CFStringRef,
    user_name: CFStringRef,
    host_name: CFStringRef,
) -> bool {
    let current_user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
    let any_host = ns_string::get_static_str(env, kCFPreferencesAnyHost);
    if app_id == nil
        || user_name == nil
        || host_name == nil
        || !msg![env; user_name isEqualToString:current_user]
        || !msg![env; host_name isEqualToString:any_host]
    {
        return false;
    }
    CFPreferencesAppSynchronize(env, app_id)
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
    export_c_func!(CFPreferencesSetValue(_, _, _, _, _)),
    export_c_func!(CFPreferencesAppSynchronize(_)),
    export_c_func!(CFPreferencesSynchronize(_, _, _)),
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

    #[test]
    fn dev_249_bundle_identifier_is_the_current_preferences_domain() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.bundle.set_fake_identity_for_test();
        let env = &mut env;
        let bundle_identifier = env.bundle.bundle_identifier().to_owned();
        let app_id = ns_string::from_rust_string(env, bundle_identifier);
        assert!(is_current_application(env, app_id));
    }

    #[test]
    fn development_256_set_value_writes_and_removes_current_domain() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.bundle.set_fake_identity_for_test();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        crate::frameworks::foundation::ns_user_defaults::install_isolated_standard_defaults_for_test(env);
        let key = ns_string::get_static_str(env, "development-256-preferences-key");
        let value = ns_string::get_static_str(env, "saved");
        let app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
        let user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
        let host = ns_string::get_static_str(env, kCFPreferencesAnyHost);

        CFPreferencesSetValue(env, key, value, app, user, host);
        let written = CFPreferencesCopyAppValue(env, key, app);
        assert_ne!(written, nil);
        assert!(msg![env; written isEqual:value]);
        release(env, written);

        CFPreferencesSetValue(env, key, nil, app, user, host);
        let removed = CFPreferencesCopyAppValue(env, key, app);
        assert_eq!(removed, nil);
    }

    #[test]
    fn development_256_named_application_domain_isolated_relaunch_and_removal() {
        let host_root = std::env::temp_dir().join(format!(
            "idared-development-256-cfpreferences-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&host_root).unwrap();
        let domain_name = "com.example.legacy-preferences-suite";

        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let key = ns_string::get_static_str(env, "named-domain-key");
            let value = ns_string::get_static_str(env, "saved");
            let app = ns_string::get_static_str(env, domain_name);
            let user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
            let host = ns_string::get_static_str(env, kCFPreferencesAnyHost);

            CFPreferencesSetValue(env, key, value, app, user, host);
            let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
            assert_eq!(CFPreferencesCopyAppValue(env, key, current_app), nil);
            assert!(CFPreferencesSynchronize(env, app, user, host));
            let unsupported_host = ns_string::get_static_str(env, "unsupported-host");
            assert!(!CFPreferencesSynchronize(
                env,
                app,
                user,
                unsupported_host
            ));
        }

        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let key = ns_string::get_static_str(env, "named-domain-key");
            let value = ns_string::get_static_str(env, "saved");
            let app = ns_string::get_static_str(env, domain_name);
            let user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
            let host = ns_string::get_static_str(env, kCFPreferencesAnyHost);

            let reloaded = CFPreferencesCopyAppValue(env, key, app);
            assert_ne!(reloaded, nil);
            assert!(msg![env; reloaded isEqual:value]);
            release(env, reloaded);
            CFPreferencesSetValue(env, key, nil, app, user, host);
            assert!(CFPreferencesSynchronize(env, app, user, host));
        }

        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let key = ns_string::get_static_str(env, "named-domain-key");
            let app = ns_string::get_static_str(env, domain_name);
            assert_eq!(CFPreferencesCopyAppValue(env, key, app), nil);
        }
    }

    #[test]
    #[should_panic]
    fn development_256_set_value_does_not_redirect_unsupported_host_domain() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.bundle.set_fake_identity_for_test();
        let env = &mut env;
        let key = ns_string::get_static_str(env, "development-256-domain-key");
        let value = ns_string::get_static_str(env, "saved");
        let app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
        let user = ns_string::get_static_str(env, kCFPreferencesCurrentUser);
        let unsupported_host = ns_string::get_static_str(env, "unsupported-host");
        CFPreferencesSetValue(env, key, value, app, user, unsupported_host);
    }
}
