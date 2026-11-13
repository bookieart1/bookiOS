/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSUserDefaults`.
//!
//! References:
//! - Apple's [Preferences and Settings Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/UserDefaults/AboutPreferenceDomains/AboutPreferenceDomains.html).

use super::ns_property_list_serialization::{
    deep_immutable_property_list_snapshot, deserialize_plist_from_file_with_mutability,
    NSPropertyListMutableContainers,
};
use super::{ns_string, NSInteger, NSUInteger};
use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, Class, ClassExports, HostObject,
    NSZonePtr,
};

use crate::Environment;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    /// `NSUserDefaults*`
    standard_defaults: Option<id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut State {
        &mut env.framework_state.foundation.ns_user_defaults
    }
}

struct NSUserDefaultsHostObject {
    /// Defaults meant to be seen by all applications.
    /// *Does NOT* persist on disk.
    /// `NSMutableDictionary *`
    global_domain_dict: id,
    /// Application own preferences.
    /// *Does* persist on disk.
    /// `NSMutableDictionary *`
    app_domain_dict: id,
    /// Application temporary defaults.
    /// *Does NOT* persist on disk.
    /// Used if not found in other dictionaries.
    /// `NSMutableDictionary *`
    registration_domain_dict: id,
    /// Additional application suites addressed through CFPreferences.
    /// Values are owned mutable dictionaries and persist as separate plists.
    named_domain_dicts: HashMap<String, id>,
}
impl HostObject for NSUserDefaultsHostObject {}

#[cfg(test)]
pub(crate) fn install_isolated_standard_defaults_for_test(env: &mut Environment) -> id {
    let defaults: id = msg_class![env; NSUserDefaults alloc];
    let app: id = msg_class![env; NSMutableDictionary new];
    let global: id = msg_class![env; NSMutableDictionary new];
    let registered: id = msg_class![env; NSMutableDictionary new];
    *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
        app_domain_dict: app,
        global_domain_dict: global,
        registration_domain_dict: registered,
        named_domain_dicts: HashMap::new(),
    };
    State::get(env).standard_defaults = Some(defaults);
    defaults
}

const NSUserDefaultsDidChangeNotification: &str = "NSUserDefaultsDidChangeNotification";
pub const CONSTANTS: ConstantExports = &[(
    "_NSUserDefaultsDidChangeNotification",
    HostConstant::NSString(NSUserDefaultsDidChangeNotification),
)];

fn post_did_change_notification(env: &mut Environment, this: id) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = ns_string::get_static_str(env, NSUserDefaultsDidChangeNotification);
    () = msg![env; center postNotificationName:name object:this];
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSUserDefaults: NSObject

+ (id)standardUserDefaults {
    if let Some(existing) = State::get(env).standard_defaults {
        existing
    } else {
        let defaults = msg![env; this alloc];
        let defaults = msg![env; defaults init];
        State::get(env).standard_defaults = Some(defaults);
        defaults
    }
}

// Saves the shared defaults and lets them go, so the next
// standardUserDefaults reads them again.
+ (())resetStandardUserDefaults {
    if let Some(defaults) = State::get(env).standard_defaults.take() {
        let _: bool = msg![env; defaults synchronize];
        autorelease(env, defaults);
    }
}

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSUserDefaultsHostObject {
        global_domain_dict: nil,
        app_domain_dict: nil,
        registration_domain_dict: nil,
        named_domain_dicts: HashMap::new(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: plist methods etc

- (id)init {
    // First, init globals
    // TODO: init globals once per app run
    // TODO: Are there other default keys we need to set?
    let langs_value: id = msg_class![env; NSLocale preferredLanguages];
    let langs_key: id = ns_string::get_static_str(env, "AppleLanguages");

    let dict = msg_class![env; NSMutableDictionary new];
    () = msg![env; dict setObject:langs_value forKey:langs_key];

    env.objc.borrow_mut::<NSUserDefaultsHostObject>(this).global_domain_dict = dict;

    // Now, load from disk and init app's own preferences.
    let plist_file_name = format!("{}.plist", env.bundle.bundle_identifier());
    let plist_file_path_buf = env.fs.home_directory()
        .join("Library")
        .join("Preferences")
        .join(plist_file_name);
    let rfq_profile = env
        .options
        .user_defaults_compatibility_profile
        .rfq_snapshot_and_mutable_reload();
    if rfq_profile {
        log!(
            "NSUserDefaults compatibility profile active: {}",
            env.options.user_defaults_compatibility_profile.as_str()
        );
    }
    let dict: id = if rfq_profile {
        deserialize_plist_from_file_with_mutability(
            env,
            &plist_file_path_buf,
            /* array_expected: */ false,
            NSPropertyListMutableContainers,
        )
    } else {
        let plist_file_path =
            ns_string::from_rust_string(env, plist_file_path_buf.as_str().to_string());
        msg_class![env; NSDictionary dictionaryWithContentsOfFile:plist_file_path]
    };

    let dict: id = if dict == nil {
        msg_class![env; NSMutableDictionary new]
    } else if rfq_profile {
        // The profile-specific loader already reconstructed the complete
        // collection graph using mutable containers. It returns an owned root.
        dict
    } else {
        msg![env; dict mutableCopy]
    };
    env.objc.borrow_mut::<NSUserDefaultsHostObject>(this).app_domain_dict = dict;

    this
}

- (())dealloc {
    let app_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    release(env, app_domain_dict);
    let global_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).global_domain_dict;
    release(env, global_domain_dict);
    let registration_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).registration_domain_dict;
    release(env, registration_domain_dict);
    let named_domains = std::mem::take(
        &mut env
            .objc
            .borrow_mut::<NSUserDefaultsHostObject>(this)
            .named_domain_dicts,
    );
    for (_, domain) in named_domains {
        release(env, domain);
    }

    env.objc.dealloc_object(this, &mut env.mem);
}

- (id)dictionaryRepresentation { // NSDictionary *
    let dict = msg_class![env; NSMutableDictionary new];
    let registration_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).registration_domain_dict;
    if registration_domain_dict != nil {
        () = msg![env; dict addEntriesFromDictionary:registration_domain_dict];
    }
    let global_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).global_domain_dict;
    () = msg![env; dict addEntriesFromDictionary:global_domain_dict];
    let app_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    () = msg![env; dict addEntriesFromDictionary:app_domain_dict];
    autorelease(env, dict)
}

- (id)persistentDomainForName:(id)domain_name { // NSString *
    if domain_name == nil {
        return nil;
    }
    let name = to_rust_string(env, domain_name).into_owned();
    let dict = if name == "NSGlobalDomain" {
        env.objc.borrow::<NSUserDefaultsHostObject>(this).global_domain_dict
    } else if !env.bundle.bundle_path().as_str().is_empty()
        && name == env.bundle.bundle_identifier()
    {
        env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict
    } else if let Some(&dict) = env
        .objc
        .borrow::<NSUserDefaultsHostObject>(this)
        .named_domain_dicts
        .get(&name)
    {
        dict
    } else {
        let plist_file_path_buf = env.fs.home_directory()
            .join("Library")
            .join("Preferences")
            .join(format!("{}.plist", name));
        let plist_file_path =
            ns_string::from_rust_string(env, plist_file_path_buf.as_str().to_string());
        let loaded: id = msg_class![env; NSDictionary dictionaryWithContentsOfFile:plist_file_path];
        if loaded == nil {
            return nil;
        }
        let mutable: id = msg![env; loaded mutableCopy];
        env.objc
            .borrow_mut::<NSUserDefaultsHostObject>(this)
            .named_domain_dicts
            .insert(name, mutable);
        mutable
    };
    if dict == nil {
        return nil;
    }
    let count: NSUInteger = msg![env; dict count];
    if count == 0 {
        return nil;
    }
    let copy: id = msg![env; dict copy];
    autorelease(env, copy)
}

- (())setPersistentDomain:(id)domain forName:(id)domain_name { // NSDictionary *, NSString *
    assert!(domain != nil && domain_name != nil);
    let name = to_rust_string(env, domain_name).into_owned();
    assert!(name != "NSGlobalDomain");
    let mutable: id = msg![env; domain mutableCopy];
    let previous = env
        .objc
        .borrow_mut::<NSUserDefaultsHostObject>(this)
        .named_domain_dicts
        .insert(name, mutable);
    if let Some(previous) = previous {
        release(env, previous);
    }
    post_did_change_notification(env, this);
}

- (())removePersistentDomainForName:(id)domain_name { // NSString *
    assert!(domain_name != nil);
    let name = to_rust_string(env, domain_name).into_owned();
    if let Some(previous) = env
        .objc
        .borrow_mut::<NSUserDefaultsHostObject>(this)
        .named_domain_dicts
        .remove(&name)
    {
        release(env, previous);
        post_did_change_notification(env, this);
    }
}

- (id)valueForKey:(id)key { // NSString*
    // This is not documented, but apparently this method
    // is overridden for NSUserDefaults.
    // Behaviour was confirmed on macOS.
    // TODO: should we call `valueForKey:` on the app_domain_dict instead?
    msg![env; this objectForKey:key]
}
- (())setValue:(id)val
        forKey:(id)key { // NSString*
    // This is not documented, but apparently this method
    // is overridden for NSUserDefaults.
    // Behaviour was confirmed on macOS.
    // Only app domain gets affected here (this part wasn't verified).
    let dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    () = msg![env; this willChangeValueForKey:key];
    () = msg![env; dict setValue:val forKey:key];
    msg![env; this didChangeValueForKey:key]
}

- (id)objectForKey:(id)key { // NSString*
    // TODO: check if order of searching is correct
    let app_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    let res: id = msg![env; app_domain_dict objectForKey:key];
    if res != nil {
        env.diagnostic_user_defaults_lookup(
            this.to_bits(), key.to_bits(), res.to_bits(), "application"
        );
        return res;
    }
    let global_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).global_domain_dict;
    let res: id = msg![env; global_domain_dict objectForKey:key];
    if res != nil {
        env.diagnostic_user_defaults_lookup(
            this.to_bits(), key.to_bits(), res.to_bits(), "global"
        );
        return res;
    }
    let registration_domain_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).registration_domain_dict;
    let res: id = msg![env; registration_domain_dict objectForKey:key];
    env.diagnostic_user_defaults_lookup(
        this.to_bits(),
        key.to_bits(),
        res.to_bits(),
        if res == nil { "missing" } else { "registration" },
    );
    res
}

- (())registerDefaults:(id)registration_dictionary {
    let reg_dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).registration_domain_dict;
    let dict = if reg_dict != nil {
        reg_dict
    } else {
        let new_dict = msg_class![env; NSMutableDictionary new];
        env.objc.borrow_mut::<NSUserDefaultsHostObject>(this).registration_domain_dict = new_dict;
        new_dict
    };

    // Add new defaults and replace any already defined ones
    () = msg![env; dict addEntriesFromDictionary:registration_dictionary];
}

- (())setObject:(id)object
         forKey:(id)key { // NSString*
    // In defaults, nil removes the application-domain value. It is not a
    // dictionary insertion; lower-priority domains remain available.
    if object == nil {
        return msg![env; this removeObjectForKey:key];
    }
    env.diagnostic_user_defaults_write(
        "set", this.to_bits(), key.to_bits(), object.to_bits()
    );
    // Only app domain gets affected!
    let dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    let previous: id = msg![env; dict objectForKey:key];
    let rfq_profile = env
        .options
        .user_defaults_compatibility_profile
        .rfq_snapshot_and_mutable_reload();
    let stored_object = if rfq_profile {
        deep_immutable_property_list_snapshot(env, object)
    } else {
        object
    };
    let value_unchanged = previous != nil && msg![env; previous isEqual:stored_object];
    if value_unchanged && !rfq_profile {
        return;
    }
    if !value_unchanged {
        () = msg![env; this willChangeValueForKey:key];
    }
    () = msg![env; dict setObject:stored_object forKey:key];
    if rfq_profile {
        release(env, stored_object);
    }
    if !value_unchanged {
        () = msg![env; this didChangeValueForKey:key];
        post_did_change_notification(env, this)
    }
}

- (())removeObjectForKey:(id)key {
    env.diagnostic_user_defaults_write("remove", this.to_bits(), key.to_bits(), 0);
    // Only app domain gets affected!
    let dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    let previous: id = msg![env; dict objectForKey:key];
    if previous == nil {
        return;
    }
    () = msg![env; this willChangeValueForKey:key];
    () = msg![env; dict removeObjectForKey:key];
    () = msg![env; this didChangeValueForKey:key];
    post_did_change_notification(env, this)
}

- (id)dataForKey:(id)key {
    let val: id = msg![env; this objectForKey:key];
    if val == nil {
        return nil;
    }
    let class: Class = msg![env; val class];
    assert!(class != nil);
    let ns_data_class = env.objc.get_known_class("NSData", &mut env.mem);
    assert!(env.objc.class_is_subclass_of(class, ns_data_class));
    val
}

- (bool)boolForKey:(id)key { // NSString *
    let val: id = msg![env; this objectForKey:key];
    msg![env; val boolValue]
}
- (())setBool:(bool)value
       forKey:(id)key { // NSString *
    let num: id = msg_class![env; NSNumber numberWithBool:value];
    msg![env; this setObject:num forKey:key]
}

- (NSInteger)integerForKey:(id)key {
    let val: id = msg![env; this objectForKey:key];
    msg![env; val integerValue]
}
- (())setInteger:(NSInteger)value
          forKey:(id)key {
    let num: id = msg_class![env; NSNumber numberWithInteger:value];
    msg![env; this setObject:num forKey:key]
}

- (f32)floatForKey:(id)key {
    let val: id = msg![env; this objectForKey:key];
    msg![env; val floatValue]
}
- (())setFloat:(f32)value
        forKey:(id)key {
    let num: id = msg_class![env; NSNumber numberWithFloat:value];
    msg![env; this setObject:num forKey:key]
}

- (f64)doubleForKey:(id)key {
    let val: id = msg![env; this objectForKey:key];
    msg![env; val doubleValue]
}
- (())setDouble:(f64)value
          forKey:(id)key {
    let num: id = msg_class![env; NSNumber numberWithDouble:value];
    msg![env; this setObject:num forKey:key]
}

- (id)stringForKey:(id)key {
    log_dbg!("NSUserDefaults stringForKey:{}", to_rust_string(env, key));
    let val: id = msg![env; this objectForKey:key];
    if val == nil {
        return nil;
    }
    let val_class: Class = msg![env; val class];
    let ns_string_class = env.objc.get_known_class("NSString", &mut env.mem);
    if env.objc.class_is_subclass_of(val_class, ns_string_class) {
        return val;
    }
    let ns_number_class = env.objc.get_known_class("NSNumber", &mut env.mem);
    if env.objc.class_is_subclass_of(val_class, ns_number_class) {
        return msg![env; val stringValue];
    }
    nil
}

- (id)arrayForKey:(id)key {
    log_dbg!("NSUserDefaults arrayForKey:{}", to_rust_string(env, key));
    let val: id = msg![env; this objectForKey:key];
    if val == nil {
        return nil;
    }
    let val_class: Class = msg![env; val class];
    let ns_array_class = env.objc.get_known_class("NSArray", &mut env.mem);
    if env.objc.class_is_subclass_of(val_class, ns_array_class) {
        return val;
    }
    nil
}

- (id)dictionaryForKey:(id)key {
    log_dbg!("NSUserDefaults dictionaryForKey:{}", to_rust_string(env, key));
    let val: id = msg![env; this objectForKey:key];
    if val == nil {
        return nil;
    }
    let val_class: Class = msg![env; val class];
    let ns_dictionary_class = env.objc.get_known_class("NSDictionary", &mut env.mem);
    if env.objc.class_is_subclass_of(val_class, ns_dictionary_class) {
        return val;
    }
    nil
}

- (bool)synchronize {
    let plist_file_path_dir = env.fs.home_directory()
        .join("Library")
        .join("Preferences");
    // TODO: can we avoid this creation call on each sync?
    _ = env.fs.create_dir_all(plist_file_path_dir.clone());
    let plist_file_name = format!("{}.plist", env.bundle.bundle_identifier());
    let plist_file_path_buf = plist_file_path_dir.join(plist_file_name);
    let plist_file_path = ns_string::from_rust_string(env, plist_file_path_buf.as_str().to_string());
    let dict = env.objc.borrow::<NSUserDefaultsHostObject>(this).app_domain_dict;
    let mut success: bool = msg![env; dict writeToFile:plist_file_path atomically:true];

    let named_domains: Vec<(String, id)> = env
        .objc
        .borrow::<NSUserDefaultsHostObject>(this)
        .named_domain_dicts
        .iter()
        .map(|(name, &dict)| (name.clone(), dict))
        .collect();
    for (name, dict) in named_domains {
        let path = plist_file_path_dir.join(format!("{}.plist", name));
        let path = ns_string::from_rust_string(env, path.as_str().to_string());
        let written: bool = msg![env; dict writeToFile:path atomically:true];
        success &= written;
    }
    success
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::UserDefaultsCompatibilityProfile;

    #[test]
    fn nil_setting_removes_only_application_value_and_reveals_fallbacks() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        // Isolate the domains without touching the developer's preferences.
        // All reads/mutations below go through real Objective-C dispatch.
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let key = ns_string::get_static_str(env, "trpg/gcID");
        let missing = ns_string::get_static_str(env, "absent");
        let unrelated = ns_string::get_static_str(env, "saved-level");
        let value = ns_string::get_static_str(env, "saved-value");
        let fallback = ns_string::get_static_str(env, "registered-value");
        () = msg![env; registered setObject:fallback forKey:key];
        () = msg![env; defaults setObject:value forKey:key];
        () = msg![env; defaults setObject:value forKey:unrelated];
        let actual: id = msg![env; defaults objectForKey:key];
        let value_is_equal: bool = msg![env; actual isEqual:value];
        assert!(value_is_equal);
        () = msg![env; defaults setObject:nil forKey:key];
        let actual: id = msg![env; app objectForKey:key];
        assert_eq!(actual, nil);
        let actual: id = msg![env; defaults objectForKey:key];
        assert_eq!(actual, fallback);
        () = msg![env; global setObject:value forKey:key];
        () = msg![env; defaults setObject:nil forKey:key];
        let actual: id = msg![env; defaults objectForKey:key];
        assert_eq!(actual, value);
        () = msg![env; defaults setObject:nil forKey:missing];
        let actual: id = msg![env; defaults objectForKey:missing];
        assert_eq!(actual, nil);
        let actual: id = msg![env; defaults objectForKey:unrelated];
        let value_is_equal: bool = msg![env; actual isEqual:value];
        assert!(value_is_equal);
        // The same nil remains invalid in NSMutableDictionary itself.
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; app setObject:nil forKey:key];
        }));
        assert!(failure.is_err());
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn persistent_domain_returns_an_immutable_snapshot_for_known_domains() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let key = ns_string::get_static_str(env, "checkpoint");
        let value = ns_string::get_static_str(env, "saved");
        () = msg![env; global setObject:value forKey:key];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: nil,
            named_domain_dicts: HashMap::new(),
        };
        let domain = ns_string::get_static_str(env, "NSGlobalDomain");
        let snapshot: id = msg![env; defaults persistentDomainForName:domain];
        assert_ne!(snapshot, nil);
        assert_ne!(snapshot, global);
        let actual: id = msg![env; snapshot objectForKey:key];
        assert_eq!(actual, value);
        let unknown = ns_string::get_static_str(env, "not.a.real.domain");
        let missing: id = msg![env; defaults persistentDomainForName:unknown];
        assert_eq!(missing, nil);
        release(env, defaults);
    }

    #[test]
    fn dev_245_change_notification_posts_only_for_effective_mutations() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let notifications: id = msg_class![env; NSMutableArray new];
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        let selector = env.objc.lookup_selector("addObject:").unwrap();
        let name = ns_string::get_static_str(env, "NSUserDefaultsDidChangeNotification");
        () =
            msg![env; center addObserver:notifications selector:selector name:name object:defaults];
        let key = ns_string::get_static_str(env, "difficulty");
        let value = ns_string::get_static_str(env, "hard");

        () = msg![env; defaults setObject:value forKey:key];
        let count: NSUInteger = msg![env; notifications count];
        assert_eq!(count, 1);
        () = msg![env; defaults setObject:value forKey:key];
        let count: NSUInteger = msg![env; notifications count];
        assert_eq!(count, 1);
        () = msg![env; defaults removeObjectForKey:key];
        let count: NSUInteger = msg![env; notifications count];
        assert_eq!(count, 2);
        () = msg![env; defaults removeObjectForKey:key];
        let count: NSUInteger = msg![env; notifications count];
        assert_eq!(count, 2);

        () = msg![env; center removeObserver:notifications];
        release(env, notifications);
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn baseline_profile_preserves_dev_247_live_object_aliasing() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.user_defaults_compatibility_profile =
            UserDefaultsCompatibilityProfile::Baseline;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        assert_eq!(
            env.options.user_defaults_compatibility_profile,
            UserDefaultsCompatibilityProfile::Baseline
        );
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let key = ns_string::get_static_str(env, "rfq-root");
        let source: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; defaults setObject:source forKey:key];
        let returned: id = msg![env; defaults objectForKey:key];
        assert_eq!(returned, source);
        release(env, source);
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn rfq_profile_snapshots_the_complete_graph_at_set_time() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.user_defaults_compatibility_profile =
            UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };

        let defaults_key = ns_string::get_static_str(env, "rfq-root");
        let child_key = ns_string::get_static_str(env, "child");
        let array_key = ns_string::get_static_str(env, "array");
        let data_key = ns_string::get_static_str(env, "data");
        let number_key = ns_string::get_static_str(env, "number");
        let date_key = ns_string::get_static_str(env, "date");
        let value_key = ns_string::get_static_str(env, "value");
        let later_key = ns_string::get_static_str(env, "later");
        let original_value = ns_string::get_static_str(env, "original");
        let changed_value = ns_string::get_static_str(env, "changed");
        let source_child: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; source_child setObject:original_value forKey:value_key];
        let source_root: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; source_root setObject:source_child forKey:child_key];
        let source_grandchild: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; source_grandchild setObject:original_value forKey:value_key];
        let source_array: id = msg_class![env; NSMutableArray new];
        () = msg![env; source_array addObject:source_grandchild];
        () = msg![env; source_root setObject:source_array forKey:array_key];
        let raw_data = env.mem.alloc(4);
        env.mem
            .bytes_at_mut(raw_data.cast(), 4)
            .copy_from_slice(&[1, 2, 3, 4]);
        let raw_data_const = raw_data.cast_const();
        let source_data: id = msg_class![env; NSMutableData
            dataWithBytes:raw_data_const
            length:4u32];
        env.mem.free(raw_data);
        let number: id = msg_class![env; NSNumber numberWithInt:42i32];
        let date: id = msg_class![env; NSDate dateWithTimeIntervalSinceReferenceDate:12345.25f64];
        () = msg![env; source_root setObject:source_data forKey:data_key];
        () = msg![env; source_root setObject:number forKey:number_key];
        () = msg![env; source_root setObject:date forKey:date_key];

        () = msg![env; defaults setObject:source_root forKey:defaults_key];
        let returned_root: id = msg![env; defaults objectForKey:defaults_key];
        let returned_root_again: id = msg![env; defaults objectForKey:defaults_key];
        let returned_via_typed_accessor: id = msg![env; defaults dictionaryForKey:defaults_key];
        assert_eq!(returned_root_again, returned_root);
        assert_eq!(returned_via_typed_accessor, returned_root);
        let returned_child: id = msg![env; returned_root objectForKey:child_key];
        let returned_array: id = msg![env; returned_root objectForKey:array_key];
        let returned_grandchild: id = msg![env; returned_array objectAtIndex:0u32];
        let returned_data: id = msg![env; returned_root objectForKey:data_key];
        let returned_number: id = msg![env; returned_root objectForKey:number_key];
        let returned_date: id = msg![env; returned_root objectForKey:date_key];
        assert_ne!(returned_root, source_root);
        assert_ne!(returned_child, source_child);
        assert_ne!(returned_array, source_array);
        assert_ne!(returned_grandchild, source_grandchild);
        assert_ne!(returned_data, source_data);
        let number_equal: bool = msg![env; returned_number isEqual:number];
        let returned_date_value: f64 = msg![env; returned_date timeIntervalSinceReferenceDate];
        assert!(number_equal);
        assert_eq!(returned_date_value, 12345.25);

        () = msg![env; source_child setObject:changed_value forKey:value_key];
        () = msg![env; source_root setObject:changed_value forKey:later_key];
        () = msg![env; source_grandchild setObject:changed_value forKey:value_key];
        () = msg![env; source_array addObject:changed_value];
        () = msg![env; source_data setLength:0u32];
        let snapshot_value: id = msg![env; returned_child objectForKey:value_key];
        let snapshot_grandchild_value: id = msg![env; returned_grandchild objectForKey:value_key];
        let snapshot_later: id = msg![env; returned_root objectForKey:later_key];
        let value_is_equal: bool = msg![env; snapshot_value isEqual:original_value];
        let grandchild_value_is_equal: bool =
            msg![env; snapshot_grandchild_value isEqual:original_value];
        assert!(value_is_equal);
        assert!(grandchild_value_is_equal);
        assert_eq!(snapshot_later, nil);
        let snapshot_array_count: NSUInteger = msg![env; returned_array count];
        let snapshot_data_length: NSUInteger = msg![env; returned_data length];
        assert_eq!(snapshot_array_count, 1);
        assert_eq!(snapshot_data_length, 4);

        let root_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; returned_root setObject:changed_value forKey:later_key];
        }));
        assert!(root_mutation.is_err());
        let child_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; returned_child setObject:changed_value forKey:value_key];
        }));
        assert!(child_mutation.is_err());
        let array_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; returned_array addObject:changed_value];
        }));
        assert!(array_mutation.is_err());
        let grandchild_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; returned_grandchild setObject:changed_value forKey:value_key];
        }));
        assert!(grandchild_mutation.is_err());

        release(env, source_root);
        release(env, source_child);
        release(env, source_array);
        release(env, source_grandchild);
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn rfq_profile_replaces_equal_values_without_duplicate_notifications() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.user_defaults_compatibility_profile =
            UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let notifications: id = msg_class![env; NSMutableArray new];
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        let selector = env.objc.lookup_selector("addObject:").unwrap();
        let notification_name = ns_string::get_static_str(env, NSUserDefaultsDidChangeNotification);
        () = msg![env; center
            addObserver:notifications
            selector:selector
            name:notification_name
            object:defaults];

        let key = ns_string::get_static_str(env, "equal-replacement");
        let child_key = ns_string::get_static_str(env, "answer");
        let value = ns_string::get_static_str(env, "same");
        let first: id = msg_class![env; NSMutableDictionary new];
        let second: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; first setObject:value forKey:child_key];
        () = msg![env; second setObject:value forKey:child_key];
        () = msg![env; defaults setObject:first forKey:key];
        let first_snapshot: id = msg![env; defaults objectForKey:key];
        () = msg![env; defaults setObject:second forKey:key];
        let second_snapshot: id = msg![env; defaults objectForKey:key];
        assert_ne!(first_snapshot, second_snapshot);
        let count: NSUInteger = msg![env; notifications count];
        assert_eq!(count, 1);

        () = msg![env; center removeObserver:notifications];
        release(env, first);
        release(env, second);
        release(env, notifications);
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn rfq_profile_rejects_invalid_or_cyclic_graphs_before_changing_defaults() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.user_defaults_compatibility_profile =
            UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };

        let invalid_defaults_key = ns_string::get_static_str(env, "invalid-graph");
        let invalid_leaf_defaults_key = ns_string::get_static_str(env, "invalid-leaf");
        let invalid_key_defaults_key = ns_string::get_static_str(env, "invalid-key");
        let nested_key = ns_string::get_static_str(env, "nested-invalid");
        let invalid_leaf: id = msg_class![env; NSObject new];

        let invalid_leaf_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; defaults setObject:invalid_leaf forKey:invalid_leaf_defaults_key];
        }));
        assert!(invalid_leaf_result.is_err());
        let stored_after_invalid_leaf: id =
            msg![env; defaults objectForKey:invalid_leaf_defaults_key];
        assert_eq!(stored_after_invalid_leaf, nil);

        let invalid_root: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; invalid_root setObject:invalid_leaf forKey:nested_key];
        let invalid_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; defaults setObject:invalid_root forKey:invalid_defaults_key];
        }));
        assert!(invalid_result.is_err());
        let stored_after_invalid: id = msg![env; defaults objectForKey:invalid_defaults_key];
        assert_eq!(stored_after_invalid, nil);

        let invalid_key_root: id = msg_class![env; NSMutableDictionary new];
        let non_string_key: id = msg_class![env; NSNumber numberWithInt:7i32];
        let valid_value = ns_string::get_static_str(env, "value");
        () = msg![env; invalid_key_root setObject:valid_value forKey:non_string_key];
        let invalid_key_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; defaults setObject:invalid_key_root forKey:invalid_key_defaults_key];
        }));
        assert!(invalid_key_result.is_err());
        let stored_after_invalid_key: id =
            msg![env; defaults objectForKey:invalid_key_defaults_key];
        assert_eq!(stored_after_invalid_key, nil);

        let cycle_defaults_key = ns_string::get_static_str(env, "cyclic-graph");
        let self_key = ns_string::get_static_str(env, "self");
        let cycle: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; cycle setObject:cycle forKey:self_key];
        let cycle_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; defaults setObject:cycle forKey:cycle_defaults_key];
        }));
        assert!(cycle_result.is_err());
        let stored_after_cycle: id = msg![env; defaults objectForKey:cycle_defaults_key];
        assert_eq!(stored_after_cycle, nil);

        release(env, invalid_root);
        release(env, invalid_key_root);
        release(env, invalid_leaf);
        // `cycle` deliberately retains itself. Releasing our ownership leaves
        // one unreachable reference, matching the malformed input supplied by
        // the test without recursively deallocating it.
        release(env, cycle);
        release(env, defaults);
    }

    #[test]
    #[ignore = "explicit desktop performance probe; run with --ignored --nocapture"]
    fn rfq_profile_desktop_snapshot_performance_probe() {
        use std::time::{Duration, Instant};

        fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
            samples.sort_unstable();
            let index = ((samples.len() - 1) * percentile) / 100;
            samples[index]
        }

        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let defaults_key = ns_string::get_static_str(env, "performance-graph");

        let qh_like: id = msg_class![env; NSMutableDictionary new];
        for index in 0..128u32 {
            let key = ns_string::from_rust_string(env, format!("hook/{index}"));
            let index_i32 = index as i32;
            let number: id = msg_class![env; NSNumber numberWithInt:index_i32];
            () = msg![env; qh_like setObject:number forKey:key];
            release(env, key);
        }

        let thousand_leaf: id = msg_class![env; NSMutableDictionary new];
        for group in 0..10u32 {
            let group_key = ns_string::from_rust_string(env, format!("group/{group}"));
            let child: id = msg_class![env; NSMutableDictionary new];
            for index in 0..100u32 {
                let key = ns_string::from_rust_string(env, format!("item/{index}"));
                let index_i32 = index as i32;
                let number: id = msg_class![env; NSNumber numberWithInt:index_i32];
                () = msg![env; child setObject:number forKey:key];
                release(env, key);
            }
            () = msg![env; thousand_leaf setObject:child forKey:group_key];
            release(env, child);
            release(env, group_key);
        }

        for (label, graph) in [("qh-like-128", qh_like), ("nested-1000", thousand_leaf)] {
            // Measure the default-off path before the enabled path so the
            // comparator cannot inherit an A1 snapshot from an earlier mode.
            // A real application has one fixed profile for its whole run.
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Baseline;
            let mut disabled = Vec::with_capacity(30);
            for _ in 0..30 {
                let start = Instant::now();
                () = msg![env; defaults setObject:graph forKey:defaults_key];
                disabled.push(start.elapsed());
            }
            let disabled_median = percentile(&mut disabled, 50);
            let disabled_p95 = percentile(&mut disabled, 95);

            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
            let mut enabled = Vec::with_capacity(30);
            for _ in 0..30 {
                let start = Instant::now();
                () = msg![env; defaults setObject:graph forKey:defaults_key];
                enabled.push(start.elapsed());
            }
            let enabled_median = percentile(&mut enabled, 50);
            let enabled_p95 = percentile(&mut enabled, 95);
            eprintln!(
                "RFQ_DESKTOP_PERF {label} enabled_median_us={} enabled_p95_us={} disabled_median_us={} disabled_p95_us={}",
                enabled_median.as_micros(),
                enabled_p95.as_micros(),
                disabled_median.as_micros(),
                disabled_p95.as_micros()
            );
        }

        release(env, qh_like);
        release(env, thousand_leaf);
        release(env, defaults);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn rfq_profile_hundred_cycle_live_object_stability() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.user_defaults_compatibility_profile =
            UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;

        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let app: id = msg_class![env; NSMutableDictionary new];
        let global: id = msg_class![env; NSMutableDictionary new];
        let registered: id = msg_class![env; NSMutableDictionary new];
        *env.objc.borrow_mut::<NSUserDefaultsHostObject>(defaults) = NSUserDefaultsHostObject {
            app_domain_dict: app,
            global_domain_dict: global,
            registration_domain_dict: registered,
            named_domain_dicts: HashMap::new(),
        };
        let defaults_key = ns_string::get_static_str(env, "stability-graph");
        let child_key = ns_string::get_static_str(env, "child");
        let value_key = ns_string::get_static_str(env, "value");
        let mut counts = Vec::with_capacity(100);

        for cycle in 0..100i32 {
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let root: id = msg_class![env; NSMutableDictionary new];
            let child: id = msg_class![env; NSMutableDictionary new];
            let number: id = msg_class![env; NSNumber numberWithInt:cycle];
            () = msg![env; child setObject:number forKey:value_key];
            () = msg![env; root setObject:child forKey:child_key];
            () = msg![env; defaults setObject:root forKey:defaults_key];
            release(env, child);
            release(env, root);
            release(env, pool);
            counts.push(env.objc.live_object_count());
        }

        // Ignore the first twenty cycles so one-time class/static caches cannot
        // masquerade as a leak. Replacing the same defaults key must then have
        // bounded live-object occupancy rather than monotonic growth.
        let settled = &counts[20..];
        let min = *settled.iter().min().unwrap();
        let max = *settled.iter().max().unwrap();
        eprintln!(
            "RFQ_OBJECT_STABILITY cycles=100 settled_cycles=80 min={} max={} delta={}",
            min,
            max,
            max - min
        );
        assert!(max - min <= 2, "live-object counts grew: {counts:?}");
        assert!(settled.windows(2).any(|pair| pair[1] <= pair[0]));

        release(env, defaults);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn rfq_a1_only_reversal_keeps_the_persisted_value_immutable() {
        let host_root = std::env::temp_dir().join(format!(
            "idared-rfq-a1-only-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&host_root).unwrap();

        // Write through A1 with the research profile selected.
        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let defaults: id = msg_class![env; NSUserDefaults alloc];
            let defaults: id = msg![env; defaults init];
            let root_key = ns_string::get_static_str(env, "a1-root");
            let value_key = ns_string::get_static_str(env, "value");
            let value = ns_string::get_static_str(env, "saved");
            let root: id = msg_class![env; NSMutableDictionary new];
            () = msg![env; root setObject:value forKey:value_key];
            () = msg![env; defaults setObject:root forKey:root_key];
            let synchronized: bool = msg![env; defaults synchronize];
            assert!(synchronized);
            release(env, root);
            release(env, defaults);
            release(env, pool);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
            env.mem.free(stack);
        }

        // Disable A2 for the new process by using the baseline loader. The
        // persisted root must return to Dev 2.47's immutable failure shape.
        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Baseline;
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let defaults: id = msg_class![env; NSUserDefaults alloc];
            let defaults: id = msg![env; defaults init];
            let root_key = ns_string::get_static_str(env, "a1-root");
            let added_key = ns_string::get_static_str(env, "added");
            let value = ns_string::get_static_str(env, "value");
            let root: id = msg![env; defaults objectForKey:root_key];
            let mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                () = msg![env; root setObject:value forKey:added_key];
            }));
            assert!(mutation.is_err());
            release(env, defaults);
            release(env, pool);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
            env.mem.free(stack);
        }

        std::fs::remove_dir_all(host_root).unwrap();
    }

    #[test]
    fn rfq_profile_persists_and_reconstructs_mutable_containers_across_three_environments() {
        let host_root = std::env::temp_dir().join(format!(
            "idared-rfq-defaults-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&host_root).unwrap();

        // Environment 1: store a nested mutable graph. A1 must immediately
        // expose an independent immutable snapshot, then persist it.
        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let defaults: id = msg_class![env; NSUserDefaults alloc];
            let defaults: id = msg![env; defaults init];
            let root_key = ns_string::get_static_str(env, "root");
            let child_key = ns_string::get_static_str(env, "child");
            let array_key = ns_string::get_static_str(env, "array");
            let value_key = ns_string::get_static_str(env, "value");
            let value = ns_string::get_static_str(env, "first");
            let child: id = msg_class![env; NSMutableDictionary new];
            () = msg![env; child setObject:value forKey:value_key];
            let root: id = msg_class![env; NSMutableDictionary new];
            () = msg![env; root setObject:child forKey:child_key];
            let array: id = msg_class![env; NSMutableArray new];
            () = msg![env; array addObject:value];
            () = msg![env; root setObject:array forKey:array_key];
            () = msg![env; defaults setObject:root forKey:root_key];
            let immediate: id = msg![env; defaults objectForKey:root_key];
            let immediate_child: id = msg![env; immediate objectForKey:child_key];
            assert_ne!(immediate, root);
            assert_ne!(immediate_child, child);
            let synchronized: bool = msg![env; defaults synchronize];
            assert!(synchronized);
            release(env, root);
            release(env, child);
            release(env, array);
            release(env, defaults);
            release(env, pool);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
            env.mem.free(stack);
        }

        // Environment 2: A2 must reconstruct both the persisted root value
        // and its nested child as mutable containers. mutableCopy remains a
        // shallow outer copy and must not alias outer mutations.
        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let defaults: id = msg_class![env; NSUserDefaults alloc];
            let defaults: id = msg![env; defaults init];
            let root_key = ns_string::get_static_str(env, "root");
            let child_key = ns_string::get_static_str(env, "child");
            let array_key = ns_string::get_static_str(env, "array");
            let value_key = ns_string::get_static_str(env, "value");
            let added_key = ns_string::get_static_str(env, "added");
            let copy_only_key = ns_string::get_static_str(env, "copy-only");
            let changed = ns_string::get_static_str(env, "second");
            let root: id = msg![env; defaults objectForKey:root_key];
            let child: id = msg![env; root objectForKey:child_key];
            let array: id = msg![env; root objectForKey:array_key];
            () = msg![env; root setObject:changed forKey:added_key];
            () = msg![env; child setObject:changed forKey:value_key];
            () = msg![env; array addObject:changed];
            let copied: id = msg![env; root mutableCopy];
            let copied_child: id = msg![env; copied objectForKey:child_key];
            assert_eq!(copied_child, child);
            () = msg![env; copied setObject:changed forKey:copy_only_key];
            let original_copy_only: id = msg![env; root objectForKey:copy_only_key];
            assert_eq!(original_copy_only, nil);
            // Re-store after mutation so the changed graph is serialized by
            // the same public NSUserDefaults path an application would use.
            () = msg![env; defaults setObject:root forKey:root_key];
            let synchronized: bool = msg![env; defaults synchronize];
            assert!(synchronized);
            release(env, copied);
            release(env, defaults);
            release(env, pool);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
            env.mem.free(stack);
        }

        // Environment 3: confirm that both second-environment mutations were
        // persisted and remain editable after another reconstruction.
        {
            let mut env = Environment::new_fake_for_test_with_thread();
            *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
            env.bundle.set_fake_identity_for_test();
            env.options.user_defaults_compatibility_profile =
                UserDefaultsCompatibilityProfile::Rfq10B500OldLinkedV1;
            env.options.preferred_languages = Some(vec!["en".to_owned()]);
            let env = &mut env;
            let stack = env.mem.alloc(0x1000);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let defaults: id = msg_class![env; NSUserDefaults alloc];
            let defaults: id = msg![env; defaults init];
            let root_key = ns_string::get_static_str(env, "root");
            let child_key = ns_string::get_static_str(env, "child");
            let array_key = ns_string::get_static_str(env, "array");
            let value_key = ns_string::get_static_str(env, "value");
            let added_key = ns_string::get_static_str(env, "added");
            let changed = ns_string::get_static_str(env, "second");
            let root: id = msg![env; defaults objectForKey:root_key];
            let child: id = msg![env; root objectForKey:child_key];
            let array: id = msg![env; root objectForKey:array_key];
            let persisted_child_value: id = msg![env; child objectForKey:value_key];
            let persisted_root_value: id = msg![env; root objectForKey:added_key];
            let child_equal: bool = msg![env; persisted_child_value isEqual:changed];
            let root_equal: bool = msg![env; persisted_root_value isEqual:changed];
            assert!(child_equal && root_equal);
            let persisted_array_count: NSUInteger = msg![env; array count];
            assert_eq!(persisted_array_count, 2);
            () = msg![env; root setObject:changed forKey:added_key];
            () = msg![env; child setObject:changed forKey:value_key];
            () = msg![env; array addObject:changed];
            release(env, defaults);
            release(env, pool);
            env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
            env.mem.free(stack);
        }

        std::fs::remove_dir_all(host_root).unwrap();
    }
}
