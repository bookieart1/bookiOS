/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSPropertyListSerialization`.

use super::{ns_array, ns_data, ns_dictionary, ns_string, NSUInteger};
use super::{
    ns_array::ArrayHostObject, ns_data::NSDataHostObject, ns_dictionary::DictionaryHostObject,
    ns_value::NSNumberHostObject,
};
use crate::frameworks::core_foundation::time::apple_epoch;
use crate::frameworks::foundation::ns_date::NSDateHostObject;
use crate::fs::GuestPath;
use crate::mem::{MutPtr, MutVoidPtr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, Class, ClassExports,
};
use crate::Environment;
use plist::Value;
use std::collections::HashSet;
use std::io::Cursor;
use std::ops::Add;
use std::time::{Duration, SystemTime};

pub type NSPropertyListMutabilityOptions = NSUInteger;
pub const NSPropertyListImmutable: NSPropertyListMutabilityOptions = 0;
pub const NSPropertyListMutableContainers: NSPropertyListMutabilityOptions = 1;
pub const NSPropertyListMutableContainersAndLeaves: NSPropertyListMutabilityOptions = 2;

pub type NSPropertyListFormat = NSUInteger;
pub const NSPropertyListXMLFormat_v1_0: NSPropertyListFormat = 100;
pub const NSPropertyListBinaryFormat_v1_0: NSPropertyListFormat = 200;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPropertyListSerialization: NSObject

+ (id)dataFromPropertyList:(id)plist
                    format:(NSPropertyListFormat)format
                errorDescription:(MutPtr<id>)error_string { // NSString **
    clear_error_description(env, error_string);

    let value = serialize_plist(env, plist);
    log_dbg!("dataFromPropertyList value {:?}", value);
    let mut buf = Vec::new();
    let result = match format {
        NSPropertyListXMLFormat_v1_0 => value.to_writer_xml(&mut buf),
        NSPropertyListBinaryFormat_v1_0 => value.to_writer_binary(&mut buf),
        _ => {
            if !error_string.is_null() {
                let error_message = ns_string::from_rust_string(
                    env,
                    format!("Unsupported property-list format {format}"),
                );
                env.mem.write(error_string, error_message);
                autorelease(env, error_message);
            }
            return nil;
        }
    };
    if let Err(error) = result {
        if !error_string.is_null() {
            let error_message = ns_string::from_rust_string(
                env,
                format!("Failed to serialize property list: {error}"),
            );
            env.mem.write(error_string, error_message);
            autorelease(env, error_message);
        }
        return nil;
    }
    let len: u32 = buf.len().try_into().unwrap();
    log_dbg!("dataFromPropertyList buf len {}", len);
    let ptr = env.mem.alloc(len);
    env.mem.bytes_at_mut(ptr.cast(), len).copy_from_slice(&buf[..]);
    msg_class![env; NSData dataWithBytesNoCopy:ptr length:len]
}

+ (id)propertyListFromData:(id)data // NSData *
          mutabilityOption:(NSPropertyListMutabilityOptions)opt
                    format:(MutPtr<NSPropertyListFormat>)format
          errorDescription:(MutPtr<id>)error_string { // NSString **
    clear_error_description(env, error_string);
    let slice = ns_data::to_rust_slice(env, data);

    if let Ok(root) = Value::from_reader_xml(Cursor::new(slice)) {
        assert!(root.as_array().is_some() || root.as_dictionary().is_some());
        if !format.is_null() {
            env.mem.write(format, NSPropertyListXMLFormat_v1_0);
        }
        let property_list = deserialize_plist(env, &root, opt);
        return autorelease(env, property_list)
    }

    if let Ok(root) = Value::from_reader(Cursor::new(slice)) {
        assert!(root.as_array().is_some() || root.as_dictionary().is_some());
        if !format.is_null() {
            env.mem.write(format, NSPropertyListBinaryFormat_v1_0);
        }
        let property_list = deserialize_plist(env, &root, opt);
        return autorelease(env, property_list)
    }

    if !error_string.is_null() {
        let error_message = ns_string::from_rust_string(env, String::from("Failed to parse plist"));
        env.mem.write(error_string, error_message);
        autorelease(env, error_message);
    }

    nil
}

@end

};

fn clear_error_description(env: &mut Environment, error_string: MutPtr<id>) {
    if !error_string.is_null() {
        env.mem.write(error_string, nil);
    }
}

/// Internals of `initWithContentsOfFile:` on `NSArray` and `NSDictionary`.
/// Returns `nil` on failure.
pub(super) fn deserialize_plist_from_file(
    env: &mut Environment,
    path: &GuestPath,
    array_expected: bool,
) -> id {
    deserialize_plist_from_file_with_mutability(env, path, array_expected, NSPropertyListImmutable)
}

/// Internal-only variant used by bounded framework compatibility profiles.
/// Public `NSDictionary`/`NSArray` file loading continues to call the wrapper
/// above and therefore retains its existing immutable nested-container route.
pub(super) fn deserialize_plist_from_file_with_mutability(
    env: &mut Environment,
    path: &GuestPath,
    array_expected: bool,
    mut_options: NSPropertyListMutabilityOptions,
) -> id {
    log_dbg!("Reading plist from {:?}.", path);
    let Ok(bytes) = env.fs.read(path) else {
        log_dbg!("Couldn't read file, returning nil.");
        return nil;
    };

    let root = match Value::from_reader(Cursor::new(bytes)) {
        Ok(root) => root,
        Err(err) => {
            log_dbg!("Couldn't parse plist, returning nil: {}", err);
            return nil;
        }
    };

    if array_expected && root.as_array().is_none() {
        log_dbg!("Plist root is not array, returning nil.");
        return nil;
    }
    if !array_expected && root.as_dictionary().is_none() {
        log_dbg!("Plist root is not dictionary, returning nil.");
        return nil;
    }

    // Note: The top-most container mutability may change
    // depending on the caller.
    // (see `NSMutableArray` and `NSMutableDictionary` implementations)
    deserialize_plist(env, &root, mut_options)
}

/// Produce an independent immutable property-list graph without changing the
/// public NSPropertyListSerialization APIs. The returned object is owned by
/// the caller. This intentionally validates the complete graph by converting
/// through the same property-list model used for persistence.
pub(super) fn deep_immutable_property_list_snapshot(
    env: &mut Environment,
    property_list: id,
) -> id {
    if let Err(reason) = validate_property_list_graph(env, property_list, &mut HashSet::new()) {
        env.diagnostic_property_list_rejection(property_list);
        panic!("NSUserDefaults rejected invalid property-list value: {reason}");
    }
    let value = serialize_plist(env, property_list);
    deserialize_plist(env, &value, NSPropertyListImmutable)
}

/// Validate before serializing so malformed graphs fail at a bounded point,
/// before NSUserDefaults changes its application-domain dictionary. Repeated
/// aliases are valid; only an object encountered again in the active ancestor
/// chain is a cycle.
fn validate_property_list_graph(
    env: &mut Environment,
    property_list: id,
    active_containers: &mut HashSet<u32>,
) -> Result<(), String> {
    if property_list == nil {
        return Err("nil nested value".to_owned());
    }
    let class: Class = msg![env; property_list class];
    let dict_class = env.objc.get_known_class("NSDictionary", &mut env.mem);
    let arr_class = env.objc.get_known_class("NSArray", &mut env.mem);
    let str_class = env.objc.get_known_class("NSString", &mut env.mem);
    let data_class = env.objc.get_known_class("NSData", &mut env.mem);

    if env.objc.class_is_subclass_of(class, dict_class) {
        let identity = property_list.to_bits();
        if !active_containers.insert(identity) {
            return Err(format!("dictionary cycle at object 0x{identity:x}"));
        }
        let dict_host_obj: DictionaryHostObject =
            std::mem::take(env.objc.borrow_mut(property_list));
        let mut pairs = Vec::with_capacity(dict_host_obj.count as usize);
        for collisions in dict_host_obj.map.values() {
            pairs.extend(collisions.iter().copied());
        }
        *env.objc.borrow_mut(property_list) = dict_host_obj;
        for (key, value) in pairs {
            let key_class: Class = msg![env; key class];
            if !env.objc.class_is_subclass_of(key_class, str_class) {
                active_containers.remove(&identity);
                return Err("dictionary key is not an NSString".to_owned());
            }
            validate_property_list_graph(env, value, active_containers)?;
        }
        active_containers.remove(&identity);
        Ok(())
    } else if env.objc.class_is_subclass_of(class, arr_class) {
        let identity = property_list.to_bits();
        if !active_containers.insert(identity) {
            return Err(format!("array cycle at object 0x{identity:x}"));
        }
        let arr_host_obj: ArrayHostObject = std::mem::take(env.objc.borrow_mut(property_list));
        let values = arr_host_obj.array.clone();
        *env.objc.borrow_mut(property_list) = arr_host_obj;
        for value in values {
            validate_property_list_graph(env, value, active_containers)?;
        }
        active_containers.remove(&identity);
        Ok(())
    } else if env.objc.class_is_subclass_of(class, str_class)
        || env.objc.class_is_subclass_of(class, data_class)
        || class == env.objc.get_known_class("NSNumber", &mut env.mem)
        || class == env.objc.get_known_class("NSDate", &mut env.mem)
    {
        Ok(())
    } else {
        Err(format!(
            "unsupported class {}",
            env.objc.get_class_name(class)
        ))
    }
}

fn deserialize_plist(
    env: &mut Environment,
    value: &Value,
    mut_options: NSPropertyListMutabilityOptions,
) -> id {
    match value {
        Value::Array(array) => {
            let array = array
                .iter()
                .map(|value| deserialize_plist(env, value, mut_options))
                .collect();
            match mut_options {
                NSPropertyListImmutable => ns_array::from_vec(env, array),
                NSPropertyListMutableContainers | NSPropertyListMutableContainersAndLeaves => {
                    ns_array::mutable_from_vec(env, array)
                }
                _ => unreachable!(),
            }
        }
        Value::Dictionary(dict) => {
            let pairs: Vec<_> = dict
                .iter()
                .map(|(key, value)| {
                    (
                        ns_string::from_rust_string(env, key.clone()),
                        deserialize_plist(env, value, mut_options),
                    )
                })
                .collect();
            // Unlike ns_array::from_vec and ns_string::from_rust_string,
            // this will retain the keys and values!
            let ns_dict = match mut_options {
                NSPropertyListImmutable => ns_dictionary::dict_from_keys_and_objects(env, &pairs),
                NSPropertyListMutableContainers | NSPropertyListMutableContainersAndLeaves => {
                    ns_dictionary::mutable_dict_from_keys_and_objects(env, &pairs)
                }
                _ => unreachable!(),
            };
            env.diagnostic_dictionary_origin(
                ns_dict,
                matches!(
                    mut_options,
                    NSPropertyListMutableContainers | NSPropertyListMutableContainersAndLeaves
                ),
                "property-list",
                mut_options,
            );
            // ...so they need to be released.
            for (key, value) in pairs {
                release(env, key);
                release(env, value);
            }
            ns_dict
        }
        Value::Boolean(b) => {
            let number: id = msg_class![env; NSNumber alloc];
            let b: bool = *b;
            msg![env; number initWithBool:b]
        }
        Value::Data(d) => {
            let length: NSUInteger = d.len().try_into().unwrap();
            let alloc: MutVoidPtr = env.mem.alloc(length);
            env.mem
                .bytes_at_mut(alloc.cast(), length)
                .copy_from_slice(d);
            let ns_data = match mut_options {
                NSPropertyListImmutable | NSPropertyListMutableContainers => {
                    msg_class![env; NSData alloc]
                }
                NSPropertyListMutableContainersAndLeaves => msg_class![env; NSMutableData alloc],
                _ => unreachable!(),
            };
            msg![env; ns_data initWithBytesNoCopy:alloc length:length]
        }
        Value::Date(date_val) => {
            let time: SystemTime = (*date_val).into();
            let time_interval = time.duration_since(apple_epoch()).unwrap().as_secs_f64();
            let date: id = msg_class![env; NSDate alloc];
            msg![env; date initWithTimeIntervalSinceReferenceDate:time_interval]
        }
        Value::Integer(int) => {
            let number: id = msg_class![env; NSNumber alloc];
            // TODO: is this the correct order of preference? does it matter?
            if let Some(int64) = int.as_signed() {
                let longlong: i64 = int64;
                msg![env; number initWithLongLong:longlong]
            } else if let Some(uint64) = int.as_unsigned() {
                let ulonglong: u64 = uint64;
                msg![env; number initWithUnsignedLongLong:ulonglong]
            } else {
                unreachable!(); // according to plist crate docs
            }
        }
        Value::Real(real) => {
            let number: id = msg_class![env; NSNumber alloc];
            let double: f64 = *real;
            msg![env; number initWithDouble:double]
        }
        Value::String(s) => match mut_options {
            NSPropertyListImmutable | NSPropertyListMutableContainers => {
                ns_string::from_rust_string(env, s.clone())
            }
            NSPropertyListMutableContainersAndLeaves => {
                ns_string::mutable_from_rust_string(env, s.clone())
            }
            _ => unreachable!(),
        },
        Value::Uid(_) => {
            // These are probably only used by NSKeyedUnarchiver, which does not
            // currently use this code in our implementation.
            unimplemented!("deserialize plist value: {:?}", value);
        }
        _ => {
            unreachable!() // enum is marked inexhaustive, but shouldn't be
        }
    }
}

fn serialize_plist(env: &mut Environment, plist: id) -> Value {
    let class: Class = msg![env; plist class];

    let dict_class = env.objc.get_known_class("NSDictionary", &mut env.mem);
    let arr_class = env.objc.get_known_class("NSArray", &mut env.mem);
    let str_class = env.objc.get_known_class("NSString", &mut env.mem);
    let data_class = env.objc.get_known_class("NSData", &mut env.mem);

    if env.objc.class_is_subclass_of(class, dict_class) {
        // only our internal implementation is supported
        assert!(env.objc.get_class_name(class).starts_with("_touchHLE_NS"));

        let mut dict = plist::dictionary::Dictionary::new();
        let dict_host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(plist));
        let mut key_vals = Vec::with_capacity(dict_host_obj.count as usize);
        for collisions in dict_host_obj.map.values() {
            for &(key, value) in collisions {
                key_vals.push((key, value));
            }
        }
        *env.objc.borrow_mut(plist) = dict_host_obj;
        for (key, val) in key_vals {
            let key_class: Class = msg![env; key class];

            // only string keys are supported
            assert!(env.objc.class_is_subclass_of(key_class, str_class));
            assert!(env
                .objc
                .get_class_name(key_class)
                .starts_with("_touchHLE_NS"));

            let key_string = ns_string::to_rust_string(env, key);
            let val_plist = serialize_plist(env, val);
            dict.insert(String::from(key_string), val_plist);
        }
        Value::Dictionary(dict)
    } else if env.objc.class_is_subclass_of(class, arr_class) {
        // only our internal implementation is supported
        assert!(env.objc.get_class_name(class).starts_with("_touchHLE_NS"));

        let arr_host_obj: ArrayHostObject = std::mem::take(env.objc.borrow_mut(plist));
        let arr: Vec<Value> = arr_host_obj
            .array
            .iter()
            .map(|&value| serialize_plist(env, value))
            .collect();
        *env.objc.borrow_mut(plist) = arr_host_obj;
        Value::Array(arr)
    } else if env.objc.class_is_subclass_of(class, str_class) {
        // only our internal implementation is supported
        assert!(env.objc.get_class_name(class).starts_with("_touchHLE_NS"));

        let s = ns_string::to_rust_string(env, plist);
        Value::String(s.to_string())
    } else if class == env.objc.get_known_class("NSNumber", &mut env.mem) {
        let num = env.objc.borrow::<NSNumberHostObject>(plist);
        match num {
            NSNumberHostObject::Bool(b) => Value::Boolean(*b),
            NSNumberHostObject::Int(i) => Value::from(*i),
            NSNumberHostObject::UnsignedInt(ui) => Value::from(*ui),
            NSNumberHostObject::Float(f) => Value::from(*f),
            NSNumberHostObject::Double(d) => Value::from(*d),
            NSNumberHostObject::LongLong(ll) => Value::from(*ll),
            NSNumberHostObject::Short(s) => Value::from(*s),
            NSNumberHostObject::Char(c) => Value::from(*c),
            _ => todo!("num {:?}", num),
        }
    } else if env.objc.class_is_subclass_of(class, data_class) {
        let data = env.objc.borrow::<NSDataHostObject>(plist);
        let buffer_slice = env.mem.bytes_at(data.bytes.cast(), data.length);
        Value::Data(buffer_slice.to_vec())
    } else if class == env.objc.get_known_class("NSDate", &mut env.mem) {
        let date = env.objc.borrow::<NSDateHostObject>(plist);
        let time = apple_epoch().add(Duration::from_secs_f64(date.time_interval));
        Value::Date(time.into())
    } else {
        env.diagnostic_property_list_rejection(plist);
        unimplemented!("class {}", env.objc.get_class_name(class))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::guest_size_of;

    #[test]
    fn successful_property_list_calls_clear_nonnull_error_output() {
        let mut env = Environment::new_fake_for_test();
        let error_string: MutPtr<id> = env.mem.alloc(guest_size_of::<id>()).cast();
        env.mem.write(error_string, id::from_bits(0x1234));
        clear_error_description(&mut env, error_string);
        assert_eq!(env.mem.read(error_string), nil);
        clear_error_description(&mut env, MutPtr::null());
        env.mem.free(error_string.cast());
    }

    #[test]
    fn xml_serialization_emits_xml_and_reports_no_error() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        // This test exercises serialization, not autorelease-pool or allocator
        // teardown. The fake environment owns guest allocations until drop.
        let _pool: id = msg_class![env; NSAutoreleasePool new];
        let value = ns_string::get_static_str(env, "value");
        let array: id = msg_class![env; NSMutableArray new];
        () = msg![env; array addObject:value];
        let error_string: MutPtr<id> = env.mem.alloc(guest_size_of::<id>()).cast();
        env.mem.write(error_string, id::from_bits(0x1234));
        let data: id = msg_class![env; NSPropertyListSerialization
            dataFromPropertyList:array
            format:NSPropertyListXMLFormat_v1_0
            errorDescription:error_string];
        assert_ne!(data, nil);
        assert_eq!(env.mem.read(error_string), nil);
        assert!(ns_data::to_rust_slice(env, data).starts_with(b"<?xml"));
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn mutable_data_leaf_serializes_as_data() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let _pool: id = msg_class![env; NSAutoreleasePool new];
        let bytes = env.mem.alloc(4);
        env.mem
            .bytes_at_mut(bytes.cast(), 4)
            .copy_from_slice(&[0xdeu8, 0xad, 0xbe, 0xef]);
        let bytes = bytes.cast_const();
        let leaf: id = msg_class![env; NSMutableData dataWithBytes:bytes length:4u32];
        let array: id = msg_class![env; NSMutableArray new];
        () = msg![env; array addObject:leaf];
        let error_description: MutPtr<id> = MutPtr::null();
        let data: id = msg_class![env; NSPropertyListSerialization
            dataFromPropertyList:array
            format:NSPropertyListBinaryFormat_v1_0
            errorDescription:error_description];
        assert_ne!(data, nil);
        assert!(!ns_data::to_rust_slice(env, data).is_empty());
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn property_list_mutability_options_remain_distinct() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let child_key = ns_string::get_static_str(env, "child");
        let leaf_key = ns_string::get_static_str(env, "leaf");
        let mutable_dict_class = env
            .objc
            .get_known_class("NSMutableDictionary", &mut env.mem);
        let mutable_string_class = env.objc.get_known_class("NSMutableString", &mut env.mem);
        let mut child = plist::dictionary::Dictionary::new();
        child.insert("leaf".to_owned(), Value::String("value".to_owned()));
        let mut root = plist::dictionary::Dictionary::new();
        root.insert("child".to_owned(), Value::Dictionary(child));
        let value = Value::Dictionary(root);

        for (option, containers_mutable, leaves_mutable) in [
            (NSPropertyListImmutable, false, false),
            (NSPropertyListMutableContainers, true, false),
            (NSPropertyListMutableContainersAndLeaves, true, true),
        ] {
            let loaded = deserialize_plist(env, &value, option);
            let loaded_child: id = msg![env; loaded objectForKey:child_key];
            let loaded_leaf: id = msg![env; loaded_child objectForKey:leaf_key];
            let root_class: Class = msg![env; loaded class];
            let child_class: Class = msg![env; loaded_child class];
            let leaf_class: Class = msg![env; loaded_leaf class];
            assert_eq!(
                env.objc
                    .class_is_subclass_of(root_class, mutable_dict_class),
                containers_mutable
            );
            assert_eq!(
                env.objc
                    .class_is_subclass_of(child_class, mutable_dict_class),
                containers_mutable
            );
            assert_eq!(
                env.objc
                    .class_is_subclass_of(leaf_class, mutable_string_class),
                leaves_mutable
            );
            release(env, loaded);
        }
    }

    #[test]
    fn rfq_file_route_reconstructs_mutable_containers_without_leaking_to_generic_route() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let host_root = std::env::temp_dir().join(format!(
            "idared-rfq-plist-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&host_root).unwrap();
        *env.fs = crate::fs::Fs::new_fake_fs_with_writable_host_root(&host_root);
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];

        let child_key = ns_string::get_static_str(env, "child");
        let value_key = ns_string::get_static_str(env, "value");
        let new_key = ns_string::get_static_str(env, "new-key");
        let original_value = ns_string::get_static_str(env, "original");
        let changed_value = ns_string::get_static_str(env, "changed");
        let child: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; child setObject:original_value forKey:value_key];
        let root: id = msg_class![env; NSMutableDictionary new];
        () = msg![env; root setObject:child forKey:child_key];

        let property_list = serialize_plist(env, root);
        let mut bytes = Vec::new();
        property_list.to_writer_binary(&mut bytes).unwrap();
        let directory = GuestPath::new("/Library/Preferences");
        env.fs.create_dir_all(directory).unwrap();
        let path = GuestPath::new("/Library/Preferences/rfq-test.plist");
        env.fs.write(path, &bytes).unwrap();

        let loaded = deserialize_plist_from_file_with_mutability(
            env,
            path,
            /* array_expected: */ false,
            NSPropertyListMutableContainers,
        );
        assert_ne!(loaded, nil);
        let loaded_child: id = msg![env; loaded objectForKey:child_key];
        () = msg![env; loaded setObject:changed_value forKey:new_key];
        () = msg![env; loaded_child setObject:changed_value forKey:value_key];
        let changed: id = msg![env; loaded_child objectForKey:value_key];
        assert_eq!(changed, changed_value);

        // Explicit mutableCopy remains shallow: it replaces the outer
        // container but retains the reconstructed child container.
        let copied: id = msg![env; loaded mutableCopy];
        let copied_child: id = msg![env; copied objectForKey:child_key];
        assert_ne!(copied, loaded);
        assert_eq!(copied_child, loaded_child);
        let copy_only_key = ns_string::get_static_str(env, "copy-only");
        () = msg![env; copied setObject:changed_value forKey:copy_only_key];
        let original_copy_only: id = msg![env; loaded objectForKey:copy_only_key];
        assert_eq!(original_copy_only, nil);

        // The existing generic file-loading route remains unchanged.
        let generic = deserialize_plist_from_file(env, path, /* array_expected: */ false);
        assert_ne!(generic, nil);
        let generic_child: id = msg![env; generic objectForKey:child_key];
        let generic_root_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; generic setObject:changed_value forKey:new_key];
        }));
        assert!(generic_root_mutation.is_err());
        let generic_child_mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            () = msg![env; generic_child setObject:changed_value forKey:value_key];
        }));
        assert!(generic_child_mutation.is_err());

        release(env, generic);
        release(env, copied);
        release(env, loaded);
        release(env, root);
        release(env, child);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
        std::fs::remove_dir_all(&host_root).unwrap();
    }
}
