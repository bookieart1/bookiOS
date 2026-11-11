/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSDictionary` class cluster, including `NSMutableDictionary`.

use super::ns_array::ArrayHostObject;
use super::ns_property_list_serialization::{
    deserialize_plist_from_file, NSPropertyListBinaryFormat_v1_0,
};
use super::ns_string::{from_rust_string, get_static_str, to_rust_string};
use super::{
    _nib_archive_decoder, ns_array, ns_keyed_unarchiver, ns_string, ns_url, NSComparisonResult,
    NSUInteger,
};
use crate::abi::{CallFromHost, GuestFunction, VaList};
use crate::cpu::Cpu;
use crate::frameworks::core_foundation::{CFHashCode, CFIndex};
use crate::frameworks::foundation::ns_enumerator::{
    fast_enumeration_helper, NSFastEnumerationState,
};
use crate::frameworks::foundation::ns_file_manager::{
    NSFileCreationDate, NSFileModificationDate, NSFileSize, NSFileType,
};
use crate::frameworks::foundation::ns_keyed_archiver::{
    encode_object, get_value_to_encode_for_current_key,
};
use crate::fs::GuestPath;
use crate::libc::stdlib::qsort::qsort_generic;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, nil, objc_classes, release, retain, Class,
    ClassExports, HostObject, NSZonePtr, TrivialHostObject, SEL,
};
use crate::{impl_HostObject_with_superclass, Environment};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[repr(C)]
struct BlockLiteral {
    _isa: MutVoidPtr,
    _flags: i32,
    _reserved: i32,
    invoke: GuestFunction,
    _descriptor: MutVoidPtr,
}
unsafe impl SafeRead for BlockLiteral {}

/// Alias for the return type of the `hash` method of the `NSObject` protocol.
type Hash = NSUInteger;

/// Belongs to _touchHLE_NSDictionary, also used by _touchHLE_NSSet
#[derive(Debug, Default)]
pub(super) struct DictionaryHostObject {
    /// Since we need custom hashing and custom equality, and these both need a
    /// `&mut Environment`, we can't just use a `HashMap<id, id>`.
    /// So here we are using a `HashMap` as a primitive for implementing a
    /// hash-map, which is not ideally efficient. :)
    /// The keys are the hash values, the values are a list of key-value pairs
    /// where the keys have the same hash value.
    pub(super) map: HashMap<Hash, Vec<(id, id)>>,
    pub(super) count: NSUInteger,
}
impl HostObject for DictionaryHostObject {}
impl DictionaryHostObject {
    pub(super) fn lookup(&self, env: &mut Environment, key: id) -> id {
        let hash: Hash = msg![env; key hash];
        let Some(collisions) = self.map.get(&hash) else {
            return nil;
        };
        for &(candidate_key, value) in collisions {
            if candidate_key == key || msg![env; candidate_key isEqual:key] {
                return value;
            }
        }
        nil
    }
    pub(super) fn insert(&mut self, env: &mut Environment, key: id, value: id, copy_key: bool) {
        let key: id = if copy_key {
            msg![env; key copy]
        } else {
            retain(env, key)
        };
        let hash: Hash = msg![env; key hash];

        let value = retain(env, value);

        let Some(collisions) = self.map.get_mut(&hash) else {
            self.map.insert(hash, vec![(key, value)]);
            self.count += 1;
            return;
        };
        for &mut (candidate_key, ref mut existing_value) in collisions.iter_mut() {
            if candidate_key == key || msg![env; candidate_key isEqual:key] {
                release(env, *existing_value);
                *existing_value = value;
                return;
            }
        }
        collisions.push((key, value));
        self.count += 1;
    }
    pub(super) fn remove(&mut self, env: &mut Environment, key: id) {
        let hash: Hash = msg![env; key hash];
        let Some(collisions) = self.map.get_mut(&hash) else {
            return;
        };
        let Some(idx) = collisions.iter().position(|&(candidate_key, _)| {
            candidate_key == key || msg![env; candidate_key isEqual:key]
        }) else {
            return;
        };
        let (existing_key, value) = collisions[idx];
        release(env, existing_key);
        release(env, value);
        collisions.remove(idx);
        self.count -= 1;
    }
    pub(super) fn release(&mut self, env: &mut Environment) {
        for collisions in self.map.values() {
            for &(key, value) in collisions {
                release(env, key);
                release(env, value);
            }
        }
    }
    pub(super) fn iter_keys(&self) -> impl Iterator<Item = id> + '_ {
        self.map.values().flatten().map(|&(key, _value)| key)
    }
}

// TODO: move those definitions to cf_dictionary.rs
// Right now they are here because we're too tied to
// NSDictionary internals, but separation could be cleaner?
#[repr(C, packed)]
pub struct CFDictionaryKeyCallBacks {
    pub version: CFIndex,         // version
    pub retain: GuestFunction,    // const void *(*retain)(CFAllocatorRef, const void *value)
    pub release: GuestFunction,   // void (*release)(CFAllocatorRef alloc, const void *val)
    pub copy_desc: GuestFunction, // CFStringRef (*copy_desc)(const void *val)
    pub equal: GuestFunction,     // Boolean (*equal)(const void *val1, const void *val2)
    pub hash: GuestFunction,      // CFHashCode (*hash)(const void *val)
}
unsafe impl SafeRead for CFDictionaryKeyCallBacks {}

#[repr(C, packed)]
pub struct CFDictionaryValueCallBacks {
    pub version: CFIndex,         // version
    pub retain: GuestFunction,    // const void *(*retain)(CFAllocatorRef, const void *value)
    pub release: GuestFunction,   // void (*release)(CFAllocatorRef alloc, const void *val)
    pub copy_desc: GuestFunction, // CFStringRef (*copy_desc)(const void *val)
    pub equal: GuestFunction,     // Boolean (*equal)(const void *val1, const void *val2)
}
unsafe impl SafeRead for CFDictionaryValueCallBacks {}

/// The choice of implementing CFDictionary as subclass
/// of NSDictionary is not a hard truth but a reflection
/// on the omnipresence of current NSDictionary implementation
/// as base of NSSet or usage of internals for property lists.
/// It's probably desirable to implement NSDictionary _atop of_
/// CFDictionary instead, but this requires considerable
/// refactoring, which I'm not very comfortable to do on
/// partially tested codebase (we do not have ability right
/// now to test NS objects directly, only CF variants ;( )
/// See TODO comment on the impl too.
pub struct CFDictionaryHostObject {
    superclass: DictionaryHostObject,
    /// `CFDictionaryKeyCallBacks`
    key_callbacks: CFDictionaryKeyCallBacks,
    /// `CFDictionaryValueCallBacks`
    value_callbacks: CFDictionaryValueCallBacks,
}
impl_HostObject_with_superclass!(CFDictionaryHostObject);
impl Default for CFDictionaryHostObject {
    fn default() -> Self {
        CFDictionaryHostObject {
            superclass: Default::default(),
            key_callbacks: CFDictionaryKeyCallBacks {
                version: 0, // version is always 0
                retain: GuestFunction::null_ptr(),
                release: GuestFunction::null_ptr(),
                copy_desc: GuestFunction::null_ptr(),
                equal: GuestFunction::null_ptr(),
                hash: GuestFunction::null_ptr(),
            },
            value_callbacks: CFDictionaryValueCallBacks {
                version: 0, // version is always 0
                retain: GuestFunction::null_ptr(),
                release: GuestFunction::null_ptr(),
                copy_desc: GuestFunction::null_ptr(),
                equal: GuestFunction::null_ptr(),
            },
        }
    }
}
// TODO: Unify implementations of NSDictionary and CFDictionary
impl CFDictionaryHostObject {
    fn lookup(&self, env: &mut Environment, key: id) -> id {
        let hash = self.hash(env, key);
        let Some(collisions) = self.superclass.map.get(&hash) else {
            return nil;
        };
        for &(candidate_key, value) in collisions {
            if self.equal_keys(env, candidate_key, key) {
                return value;
            }
        }
        nil
    }
    fn contains_key(&self, env: &mut Environment, key: id) -> bool {
        let hash = self.hash(env, key);
        let Some(collisions) = self.superclass.map.get(&hash) else {
            return false;
        };
        collisions
            .iter()
            .any(|&(candidate_key, _)| self.equal_keys(env, candidate_key, key))
    }
    fn insert(&mut self, env: &mut Environment, key: id, value: id) {
        let hash = self.hash(env, key);
        let key = self.retain_key(env, key);
        let value = self.retain_value(env, value);
        self.superclass.count += 1;
        if let Entry::Vacant(e) = self.superclass.map.entry(hash) {
            e.insert(vec![(key, value)]);
            return;
        };
        // remove if present (count will be decremented if necessary)
        self.remove(env, key);
        self.superclass
            .map
            .get_mut(&hash)
            .unwrap()
            .push((key, value));
    }
    fn remove(&mut self, env: &mut Environment, key: id) -> bool {
        let hash = self.hash(env, key);
        let Some(collisions) = self.superclass.map.get(&hash) else {
            return false;
        };
        let maybe_pos = collisions
            .iter()
            .position(|&(candidate_key, _)| self.equal_keys(env, candidate_key, key));
        if let Some(pos) = maybe_pos {
            let (existing_key, existing_value) =
                self.superclass.map.get_mut(&hash).unwrap().remove(pos);
            self.release_key(env, existing_key);
            self.release_value(env, existing_value);
            self.superclass.count -= 1;
            true
        } else {
            false
        }
    }
    // helpers
    fn hash(&self, env: &mut Environment, key: id) -> CFHashCode {
        let hash_func = self.key_callbacks.hash;
        if hash_func.to_ptr().is_null() {
            // use the pointer value as a hash code
            key.to_bits()
        } else {
            hash_func.call_from_host(env, (key,))
        }
    }
    fn equal_keys(&self, env: &mut Environment, key1: id, key2: id) -> bool {
        let equal_func = self.key_callbacks.equal;
        if equal_func.to_ptr().is_null() {
            // pointer equality
            key1 == key2
        } else {
            equal_func.call_from_host(env, (key1, key2))
        }
    }
    fn retain_key(&mut self, env: &mut Environment, key: id) -> id {
        let key_retain_func = self.key_callbacks.retain;
        if key_retain_func.to_ptr().is_null() {
            key
        } else {
            // TODO: custom dict allocator
            key_retain_func.call_from_host(env, (nil, key))
        }
    }
    fn release_key(&mut self, env: &mut Environment, key: id) {
        let key_release_func = self.key_callbacks.release;
        if !key_release_func.to_ptr().is_null() {
            // TODO: custom dict allocator
            key_release_func.call_from_host(env, (nil, key))
        }
    }
    fn retain_value(&mut self, env: &mut Environment, value: id) -> id {
        let value_retain_func = self.value_callbacks.retain;
        if value_retain_func.to_ptr().is_null() {
            value
        } else {
            // TODO: custom dict allocator
            value_retain_func.call_from_host(env, (nil, value))
        }
    }
    fn release_value(&mut self, env: &mut Environment, value: id) {
        let value_release_func = self.value_callbacks.release;
        if !value_release_func.to_ptr().is_null() {
            // TODO: custom dict allocator
            value_release_func.call_from_host(env, (nil, value))
        }
    }
}

/// Helper to enable sharing `dictionaryWithObjectsAndKeys:` and
/// `initWithObjectsAndKeys:`' implementations without vararg passthrough.
pub fn init_with_objects_and_keys(
    env: &mut Environment,
    this: id,
    first_object: id,
    mut va_args: VaList,
) -> id {
    let first_key: id = va_args.next(env);
    assert!(first_key != nil); // TODO: raise proper exception

    let mut host_object = <DictionaryHostObject as Default>::default();
    host_object.insert(env, first_key, first_object, /* copy_key: */ true);

    loop {
        let object: id = va_args.next(env);
        if object == nil {
            break;
        }
        let key: id = va_args.next(env);
        assert!(key != nil); // TODO: raise proper exception
        host_object.insert(env, key, object, /* copy_key: */ true);
    }

    *env.objc.borrow_mut(this) = host_object;

    this
}

/// Helper function to share `initWithDictionary:` implementations
fn init_with_dictionary_common(env: &mut Environment, this: id, other_dict: id) -> id {
    // Objective-C messaging to nil makes the source behave like an empty
    // dictionary here. This is relied on by legacy offline/error paths and is
    // distinct from inserting a nil key or value, which remains invalid.
    if other_dict == nil {
        *env.objc.borrow_mut(this) = <DictionaryHostObject as Default>::default();
        return this;
    }
    let other_host_object: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(other_dict));

    let mut host_object = <DictionaryHostObject as Default>::default();

    for key in other_host_object.iter_keys() {
        let object = other_host_object.lookup(env, key);
        host_object.insert(env, key, object, /* copy_key: */ true);
    }

    *env.objc.borrow_mut(this) = host_object;
    *env.objc.borrow_mut(other_dict) = other_host_object;
    this
}

/// Helper function so share `initWithObjects:ForKeys:` implementations
fn init_with_objects_for_keys_common(env: &mut Environment, this: id, objects: id, keys: id) -> id {
    let keys_size: NSUInteger = msg![env; keys count];
    let objects_size: NSUInteger = msg![env; objects count];
    assert_eq!(keys_size, objects_size); // TODO: raise proper exception

    let mut host_object = <DictionaryHostObject as Default>::default();

    let objects_enumerator: id = msg![env; objects objectEnumerator];
    let keys_enumerator: id = msg![env; keys objectEnumerator];

    loop {
        let next_key: id = msg![env; keys_enumerator nextObject];
        let next_object: id = msg![env; objects_enumerator nextObject];
        if next_key == nil {
            assert_eq!(next_object, nil);
            break;
        }
        host_object.insert(env, next_key, next_object, /* copy_key: */ true);
    }
    *env.objc.borrow_mut(this) = host_object;
    this
}

fn init_with_object_and_key_pointers_common(
    env: &mut Environment,
    this: id,
    objects: ConstPtr<id>,
    keys: ConstPtr<id>,
    count: NSUInteger,
) -> id {
    if count != 0 {
        assert!(!objects.is_null());
        assert!(!keys.is_null());
    }

    let mut host_object = <DictionaryHostObject as Default>::default();
    for index in 0..count {
        let object = env.mem.read(objects + index);
        let key = env.mem.read(keys + index);
        assert_ne!(object, nil); // TODO: raise NSInvalidArgumentException
        assert_ne!(key, nil); // TODO: raise NSInvalidArgumentException
        host_object.insert(env, key, object, /* copy_key: */ true);
    }
    *env.objc.borrow_mut(this) = host_object;
    this
}

/// Helper function to share `allKeys` implementations
fn all_keys_common(env: &mut Environment, this: id) -> id {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let keys: Vec<id> = host_obj
        .map
        .values()
        .flatten()
        .map(|&(key, _value)| key)
        .collect();
    *env.objc.borrow_mut(this) = host_obj;
    for &key in &keys {
        retain(env, key);
    }
    let res = ns_array::from_vec(env, keys);
    autorelease(env, res)
}

fn escape_strings_file_component(component: &str) -> String {
    let mut escaped = String::with_capacity(component.len());
    for character in component.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn alloc_mutable_dictionary_class_cluster_member(
    env: &mut Environment,
    class: Class,
    zone: NSZonePtr,
) -> id {
    if class == env.objc.get_known_class("NSMutableDictionary", &mut env.mem) {
        msg_class![env; _touchHLE_NSMutableDictionary allocWithZone:zone]
    } else {
        env.objc
            .alloc_object(class, Box::new(TrivialHostObject), &mut env.mem)
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSDictionary is an abstract class. A subclass must provide:
// - (id)initWithObjects:(id*)forKeys:(id*)count:(NSUInteger)
// - (NSUInteger)count
// - (id)objectForKey:(id)
// - (NSEnumerator*)keyEnumerator
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSDictionary.
@implementation NSDictionary: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    if this == env.objc.get_known_class("NSDictionary", &mut env.mem) {
        msg_class![env; _touchHLE_NSDictionary allocWithZone:zone]
    } else {
        // Real subclasses must retain their class identity rather than being
        // replaced with touchHLE's private concrete NSDictionary class.
        env.objc.alloc_object(this, Box::new(TrivialHostObject), &mut env.mem)
    }
}

+ (id)dictionary {
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict init];
    autorelease(env, new_dict)
}

+ (id)dictionaryWithObject:(id)object forKey:(id)key {
    assert_ne!(key, nil); // TODO: raise proper exception

    let new_dict = dict_from_keys_and_objects(env, &[(key, object)]);
    autorelease(env, new_dict)
}

+ (id)dictionaryWithObjectsAndKeys:(id)first_object, ...dots {
    let new_dict: id = msg![env; this alloc];
    let new_dict = init_with_objects_and_keys(env, new_dict, first_object, dots.start());
    autorelease(env, new_dict)
}

+ (id)dictionaryWithObjects:(ConstPtr<id>)objects
                    forKeys:(ConstPtr<id>)keys
                      count:(NSUInteger)count {
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict initWithObjects:objects forKeys:keys count:count];
    autorelease(env, new_dict)
}

// These probably comes from some category related to plists.
+ (id)dictionaryWithContentsOfFile:(id)path { // NSString*
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict initWithContentsOfFile:path];
    autorelease(env, new_dict)
}
+ (id)dictionaryWithContentsOfURL:(id)url { // NSURL*
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict initWithContentsOfURL:url];
    autorelease(env, new_dict)
}

+ (id)dictionaryWithObjects:(id)objects //NSArray *
                    forKeys:(id)keys { //NSArray *
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict initWithObjects:objects forKeys:keys];
    autorelease(env, new_dict)
}

+ (id)dictionaryWithDictionary:(id)dict { // NSDictionary*
    let new_dict: id = msg![env; this alloc];
    let new_dict: id = msg![env; new_dict initWithDictionary:dict];
    autorelease(env, new_dict)
}

- (id)init {
    // NSDictionary is an abstract class cluster root. A genuine subclass owns
    // its own primitive storage, so its superclass initializer must preserve
    // the allocated object rather than substitute touchHLE's concrete class.
    this
}

// Private bridge used by CFDictionaryGetValueIfPresent. Ordinary
// NSDictionary instances cannot contain nil values, so objectForKey: is
// sufficient here. The CFDictionary-backed subclass overrides this because a
// dictionary created with NULL callbacks may contain a NULL value.
- (bool)_touchHLEContainsKey:(id)key {
    let value: id = msg![env; this objectForKey:key];
    value != nil
}

- (id)objectForKeyedSubscript:(id)key {
    msg![env; this objectForKey:key]
}

// These probably comes from some category related to plists.
- (id)initWithContentsOfFile:(id)path { // NSString*
    release(env, this);
    if path == nil {
        return nil;
    }
    let path = ns_string::to_rust_string(env, path);
    deserialize_plist_from_file(
        env,
        GuestPath::new(&path),
        /* array_expected: */ false,
    )
}
- (id)initWithContentsOfURL:(id)url { // NSURL*
    release(env, this);
    if url == nil {
        return nil;
    }
    let path = ns_url::to_rust_path(env, url);
    deserialize_plist_from_file(env, &path, /* array_expected: */ false)
}

- (bool)writeToFile:(id)path // NSString*
         atomically:(bool)atomically {
    let error_desc: MutPtr<id> = Ptr::null();
    let data: id = msg_class![env; NSPropertyListSerialization
            dataFromPropertyList:this
                          format:NSPropertyListBinaryFormat_v1_0
                errorDescription:error_desc];
    let res = msg![env; data writeToFile:path atomically:atomically];
    log_dbg!(
        "[(NSDictionary *){:?} writeToFile:{:?} atomically:{}] -> {}",
        this,
        to_rust_string(env, path),
        atomically,
        res
    );
    res
}

- (bool)writeToURL:(id)url // NSURL*
        atomically:(bool)atomically {
    if url == nil {
        return false;
    }
    let is_file_url: bool = msg![env; url isFileURL];
    if !is_file_url {
        log_dbg!(
            "[(NSDictionary *){:?} writeToURL:{:?} atomically:{}] -> false (non-file URL)",
            this,
            url,
            atomically
        );
        return false;
    }
    let path: id = msg![env; url path];
    if path == nil {
        return false;
    }
    let res: bool = msg![env; this writeToFile:path atomically:atomically];
    log_dbg!(
        "[(NSDictionary *){:?} writeToURL:{:?} atomically:{}] -> {}",
        this,
        url,
        atomically,
        res
    );
    res
}

// TODO

- (id)valueForKey:(id)key { // NSString*
    if key == nil {
        return nil;
    }
    let key_str = to_rust_string(env, key);
    // TODO: strip '@' and call super
    assert!(!key_str.starts_with('@'));
    msg![env; this objectForKey:key]
}

// Generic implementation for real NSDictionary subclasses. Concrete
// touchHLE dictionaries override this with a direct, faster implementation.
- (id)allKeys {
    let keys: id = msg_class![env; NSMutableArray array];
    let enumerator: id = msg![env; this keyEnumerator];
    loop {
        let key: id = msg![env; enumerator nextObject];
        if key == nil {
            break;
        }
        () = msg![env; keys addObject:key];
    }
    let immutable_keys: id = msg![env; keys copy];
    autorelease(env, immutable_keys)
}

// Generic implementation for immutable dictionaries and real NSDictionary
// subclasses. The concrete mutable dictionary keeps its direct implementation
// below, but an immutable _touchHLE_NSDictionary must inherit this contract
// too.
- (id)allValues {
    let values: id = msg_class![env; NSMutableArray array];
    let enumerator: id = msg![env; this keyEnumerator];
    loop {
        let key: id = msg![env; enumerator nextObject];
        if key == nil {
            break;
        }
        let value: id = msg![env; this objectForKey:key];
        // NSDictionary cannot contain nil values. Keep this assertion here so
        // broken custom subclasses fail at the point where they violate the
        // collection contract rather than inside NSMutableArray.
        assert!(value != nil);
        () = msg![env; values addObject:value];
    }
    let immutable_values: id = msg![env; values copy];
    autorelease(env, immutable_values)
}

- (id)descriptionInStringsFileFormat {
    let string_class: Class = msg_class![env; NSString class];
    let number_class: Class = msg_class![env; NSNumber class];
    let keys: id = msg![env; this allKeys];
    let count: NSUInteger = msg![env; keys count];
    let mut output = String::new();
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        let value: id = msg![env; this objectForKey:key];
        assert!(key != nil && value != nil && msg![env; key isKindOfClass:string_class],
            "descriptionInStringsFileFormat requires NSString keys and property-list values");
        let key = escape_strings_file_component(&to_rust_string(env, key));
        output.push('"');
        output.push_str(&key);
        output.push_str("\" = ");
        if msg![env; value isKindOfClass:string_class] {
            let value = escape_strings_file_component(&to_rust_string(env, value));
            output.push('"');
            output.push_str(&value);
            output.push('"');
        } else if msg![env; value isKindOfClass:number_class] {
            let description: id = msg![env; value description];
            output.push_str(&to_rust_string(env, description));
        } else {
            panic!("descriptionInStringsFileFormat received unsupported property-list value");
        }
        output.push_str(";\n");
    }
    let description = from_rust_string(env, output);
    autorelease(env, description)
}

- (id)keysSortedByValueUsingSelector:(SEL)comparator {
    assert!(!comparator.is_null(), "keysSortedByValueUsingSelector: requires a selector");

    // Work through NSDictionary's primitive methods so this also behaves
    // correctly for genuine guest subclasses. The returned array owns the
    // retained keys; values remain owned by the dictionary.
    let all_keys: id = msg![env; this allKeys];
    let count: NSUInteger = msg![env; all_keys count];
    let mut keys = Vec::with_capacity(count as usize);
    for index in 0..count {
        let key: id = msg![env; all_keys objectAtIndex:index];
        keys.push(retain(env, key));
    }

    let mut user_data = (env, this, &mut keys);
    qsort_generic(
        &mut user_data,
        count,
        &mut |(env, dictionary, keys), left, right| {
            let dictionary = *dictionary;
            let left_key = keys[left as usize];
            let right_key = keys[right as usize];
            let left_value: id = msg![env; dictionary objectForKey:left_key];
            let right_value: id = msg![env; dictionary objectForKey:right_key];
            assert!(left_value != nil && right_value != nil);
            let result: NSComparisonResult =
                msg_send(env, (left_value, comparator, right_value));
            result
        },
        &mut |(_, _, keys), left, right| {
            keys.swap(left as usize, right as usize);
        },
    );
    let (env, _, keys) = user_data;
    let sorted_keys = ns_array::from_vec(env, std::mem::take(keys));
    autorelease(env, sorted_keys)
}

- (())enumerateKeysAndObjectsUsingBlock:(MutPtr<BlockLiteral>)block {
    assert!(!block.is_null(), "enumerateKeysAndObjectsUsingBlock: requires a block");
    let invoke = env.mem.read(block).invoke;
    assert_ne!(invoke.addr_without_thumb_bit(), 0);

    // Snapshot keys before invoking guest code. NSDictionary is immutable;
    // this also gives genuine subclasses their primitive-method behavior.
    let keys: id = msg![env; this allKeys];
    let count: NSUInteger = msg![env; keys count];
    let stop = env.mem.alloc_and_write(false);
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        let value: id = msg![env; this objectForKey:key];
        assert_ne!(value, nil, "NSDictionary subclass returned nil during enumeration");
        () = invoke.call_from_host(env, (block.cast_void(), key, value, stop));
        if env.mem.read(stop) {
            break;
        }
    }
    env.mem.free(stop.cast());
}

- (NSUInteger)hash {
    // TODO: define better hash
    msg![env; this count]
}
- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSDictionary class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
    msg![env; this isEqualToDictionary:other]
}
- (bool)isEqualToDictionary:(id)other { // NSDictionary *
    if other == nil {
        return false;
    }
    let count: NSUInteger = msg![env; this count];
    let other_count: NSUInteger = msg![env; other count];
    if count != other_count {
        return false;
    }
    let keys_arr = msg![env; this allKeys];
    let keys_count: NSUInteger = msg![env; keys_arr count];
    for i in 0..keys_count {
        let key: id = msg![env; keys_arr objectAtIndex:i];
        let value: id = msg![env; this objectForKey:key];
        let other_value: id = msg![env; other objectForKey:key];
        let equal: bool = msg![env; value isEqual:other_value];
        if !equal {
            return false;
        }
    }
    true
}

// NSDictionary(NSFileAttributes) category
// TODO: implement categories properly
- (id)fileCreationDate {
    let creation_date_key = get_static_str(env, NSFileCreationDate);
    msg![env; this objectForKey:creation_date_key]
}
- (id)fileModificationDate {
    let modif_date_key = get_static_str(env, NSFileModificationDate);
    msg![env; this objectForKey:modif_date_key]
}
- (u64)fileSize {
    let size_key = get_static_str(env, NSFileSize);
    let num = msg![env; this objectForKey:size_key];
    if num != nil {
        msg![env; num unsignedLongLongValue]
    } else {
        // GnuStep docs claiming to return NSNotFound here [ref](https://www.gnustep.org/resources/documentation/Developer/Base/Reference/NSFileManager.html#method$NSDictionary(NSFileAttributes)-fileSize)
        // But as seen on iPhone Simulator, it's returning 0 with an empty dict
        0
    }
}
- (id)fileType {
    let file_type_key = get_static_str(env, NSFileType);
    msg![env; this objectForKey:file_type_key]
}

@end

// NSMutableDictionary is an abstract class. A subclass must provide everything
// NSDictionary provides, plus:
// - (void)setObject:(id)object forKey:(id)key;
// - (void)removeObjectForKey:(id)key;
// Note that it inherits from NSDictionary, so we must ensure we override
// any default methods that would be inappropriate for mutability.
@implementation NSMutableDictionary: NSDictionary

+ (id)allocWithZone:(NSZonePtr)zone {
    // True subclasses implement the abstract primitives themselves and must
    // retain their identity and guest ivars.
    alloc_mutable_dictionary_class_cluster_member(env, this, zone)
}

+ (id)dictionaryWithCapacity:(NSUInteger)capacity {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCapacity:capacity];
    autorelease(env, new)
}

// NSDictionary's convenience implementation predates subclass-aware
// construction and always creates the immutable concrete class. Override it
// here so callers of [NSMutableDictionary dictionaryWithObject:forKey:] get
// the mutable class they requested.
+ (id)dictionaryWithObject:(id)object forKey:(id)key {
    assert_ne!(key, nil); // TODO: raise proper exception
    assert_ne!(object, nil); // TODO: raise proper exception

    let new_dict = mutable_dict_from_keys_and_objects(env, &[(key, object)]);
    autorelease(env, new_dict)
}

// NSMutableDictionary's KVC assignment contract maps keys to dictionary
// entries. As on Foundation, assigning nil removes the entry rather than
// attempting to store a nil object.
- (())setValue:(id)value forKey:(id)key {
    if value == nil {
        () = msg![env; this removeObjectForKey:key];
    } else {
        () = msg![env; this setObject:value forKey:key];
    }
}

- (())removeObjectsForKeys:(id)keys { // NSArray *
    if keys == nil {
        return;
    }
    let count: NSUInteger = msg![env; keys count];
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        () = msg![env; this removeObjectForKey:key];
    }
}

// These probably comes from some category related to plists.
- (id)initWithContentsOfFile:(id)path { // NSString*
    release(env, this);
    if path == nil {
        return nil;
    }
    let path = ns_string::to_rust_string(env, path);
    let tmp = deserialize_plist_from_file(
        env,
        GuestPath::new(&path),
        /* array_expected: */ false
    );
    if tmp == nil {
        return nil;
    }
    // We should respect mutability of the top most container!
    let res = msg_class![env; NSMutableDictionary alloc];
    let res = msg![env; res initWithDictionary:tmp];
    release(env, tmp);
    res
}
- (id)initWithContentsOfURL:(id)url { // NSURL*
    release(env, this);
    if url == nil {
        return nil;
    }
    let path = ns_url::to_rust_path(env, url);
    let tmp = deserialize_plist_from_file(env, &path, /* array_expected: */ false);
    if tmp == nil {
        return nil;
    }
    // We should respect mutability of the top most container!
    let res = msg_class![env; NSMutableDictionary alloc];
    let res = msg![env; res initWithDictionary:tmp];
    release(env, tmp);
    res
}

@end

// Our private subclass that is the single implementation of NSDictionary for
// the time being.
@implementation _touchHLE_NSDictionary: NSDictionary

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<DictionaryHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())dealloc {
    std::mem::take(env.objc.borrow_mut::<DictionaryHostObject>(this)).release(env);

    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithObjectsAndKeys:(id)first_object, ...dots {
    init_with_objects_and_keys(env, this, first_object, dots.start())
}

- (id)init {
    *env.objc.borrow_mut(this) = <DictionaryHostObject as Default>::default();
    this
}

- (id)initWithDictionary:(id)dictionary {
    init_with_dictionary_common(env, this, dictionary)
}

- (id)initWithObjects:(id)objects //NSArray *
              forKeys:(id)keys { //NSArray *
    init_with_objects_for_keys_common(env, this, objects, keys)
}

- (id)initWithObjects:(ConstPtr<id>)objects
              forKeys:(ConstPtr<id>)keys
                count:(NSUInteger)count {
    init_with_object_and_key_pointers_common(env, this, objects, keys, count)
}

// TODO: enumeration, more init methods, etc

- (NSUInteger)count {
    env.objc.borrow::<DictionaryHostObject>(this).count
}
- (id)objectForKey:(id)key {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let res = host_obj.lookup(env, key);
    *env.objc.borrow_mut(this) = host_obj;
    res
}

- (id)allKeys {
    all_keys_common(env, this)
}

- (id)keyEnumerator {
    let keys: id = msg![env; this allKeys];
    msg![env; keys objectEnumerator]
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    // We assume that order in which objects are reported is consistent
    // between calls!
    let objects: id = msg![env; this allKeys];
    let count: NSUInteger = msg![env; objects count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; objects objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    let mut_dict: id = msg_class![env; NSMutableDictionary alloc];
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    for (k, v) in host_obj.map.values().flatten() {
        () = msg![env; mut_dict setObject:(*v) forKey:(*k)];
    }
    *env.objc.borrow_mut(this) = host_obj;
    mut_dict
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    init_with_coder_inner(env, this, coder)
}
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

- (id)description {
    build_description(env, this)
}

@end

// Our private subclass that is the single implementation of
// NSMutableDictionary for the time being.
@implementation _touchHLE_NSMutableDictionary: NSMutableDictionary

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<DictionaryHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())dealloc {
    std::mem::take(env.objc.borrow_mut::<DictionaryHostObject>(this)).release(env);

    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithObjectsAndKeys:(id)first_object, ...dots {
    init_with_objects_and_keys(env, this, first_object, dots.start())
}

- (id)initWithDictionary:(id)dictionary {
    init_with_dictionary_common(env, this, dictionary)
}

- (id)init {
    *env.objc.borrow_mut(this) = <DictionaryHostObject as Default>::default();
    this
}

- (id)initWithCapacity:(NSUInteger)_capacity {
    // TODO: capacity
    msg![env; this init]
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    init_with_coder_inner(env, this, coder)
}
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

- (id)initWithObjects:(id)objects //NSArray *
              forKeys:(id)keys { //NSArray *
    init_with_objects_for_keys_common(env, this, objects, keys)
}

- (id)initWithObjects:(ConstPtr<id>)objects
              forKeys:(ConstPtr<id>)keys
                count:(NSUInteger)count {
    init_with_object_and_key_pointers_common(env, this, objects, keys, count)
}

// TODO: enumeration, more init methods, etc

- (NSUInteger)count {
    env.objc.borrow::<DictionaryHostObject>(this).count
}
- (id)objectForKey:(id)key {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let res = host_obj.lookup(env, key);
    *env.objc.borrow_mut(this) = host_obj;
    res
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    // TODO: check that dict wasn't mutated!
    // We assume that order in which objects are reported is consistent
    // between calls!
    let objects: id = msg![env; this allKeys];
    let count: NSUInteger = msg![env; objects count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; objects objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    let entries: Vec<_> =
        env.objc.borrow_mut::<DictionaryHostObject>(this).map.values().flatten().copied().collect();
    dict_from_keys_and_objects(env, &entries)
}

// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    let mut_dict: id = msg_class![env; NSMutableDictionary alloc];
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    for (k, v) in host_obj.map.values().flatten() {
        () = msg![env; mut_dict setObject:(*v) forKey:(*k)];
    }
    *env.objc.borrow_mut(this) = host_obj;
    mut_dict
}

- (())setValue:(id)value
        forKey:(id)key { // NSString *
    // TODO: assert that key is a string when using key-value coding
    if value == nil {
        msg![env; this removeObjectForKey:key]
    } else {
        msg![env; this setObject:value forKey:key]
    }
}

- (())setObject:(id)object
         forKey:(id)key {
    // TODO: raise NSInvalidArgumentException
    if object == nil {
        let regs = env.cpu.regs();
        let (pc, lr) = (regs[Cpu::PC], regs[Cpu::LR]);
        let key_details = if key == nil {
            "nil key".to_string()
        } else if env.objc.get_host_object(key).is_none() {
            format!("unregistered key pointer {key:?}")
        } else {
            let class = crate::objc::ObjC::read_isa(key, &env.mem);
            let class_name = env
                .objc
                .try_get_class_name(class)
                .unwrap_or("<unknown>")
                .to_string();
            let ns_string_class = env.objc.get_known_class("NSString", &mut env.mem);
            if env.objc.class_is_subclass_of(class, ns_string_class) {
                format!(
                    "key {key:?}, class {class_name:?}, string {:?}",
                    ns_string::to_rust_string(env, key),
                )
            } else {
                format!("key {key:?}, class {class_name:?}")
            }
        };
        panic!(
            "NSMutableDictionary {this:?} rejected a nil object for {key_details}; guest thread {}, PC {pc:#010x}, LR {lr:#010x}",
            env.current_thread,
        );
    }
    // TODO: raise NSInvalidArgumentException
    assert_ne!(key, nil);
    let mut host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    host_obj.insert(env, key, object, /* copy_key: */ true);
    *env.objc.borrow_mut(this) = host_obj;
}

- (())removeObjectForKey:(id)key {
    assert!(!key.is_null());
    let mut host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    host_obj.remove(env, key);
    *env.objc.borrow_mut(this) = host_obj;
}

- (())removeAllObjects {
    let mut old_host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    old_host_obj.release(env);
}

- (())setDictionary:(id)other { // NSDictionary *
    () = msg![env; this removeAllObjects];
    () = msg![env; this addEntriesFromDictionary:other];
}

- (())addEntriesFromDictionary:(id)other { // NSDictionary *
    if other == nil  {
        return
    }
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(other));
    for (k, v) in host_obj.map.values().flatten() {
        () = msg![env; this setObject:(*v) forKey:(*k)];
    }
    *env.objc.borrow_mut(other) = host_obj;
}

- (id)description {
    build_description(env, this)
}

- (id)allKeys {
    all_keys_common(env, this)
}

- (id)keyEnumerator {
    let keys: id = msg![env; this allKeys];
    msg![env; keys objectEnumerator]
}

- (id)allValues {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let values: Vec<id> = host_obj.map.values().flatten().map(|&(_key, value)| value).collect();
    *env.objc.borrow_mut(this) = host_obj;

    for &val in &values {
        retain(env, val);
    }
    let res = ns_array::from_vec(env, values);
    autorelease(env, res)
}

- (id)allKeysForObject:(id)obj {
    let res: id = msg_class![env; NSMutableArray new];

    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    host_obj.map.values().flatten().for_each(|&(key, value)| {
        let equal = msg![env; obj isEqual:value];
        if equal {
            () = msg![env; res addObject:key];
        }
    });
    *env.objc.borrow_mut(this) = host_obj;

    let res_imm = msg![env; res copy];
    release(env, res);
    autorelease(env, res_imm)
}

- (id)objectEnumerator { // NSEnumerator*
    let values: id = msg![env; this allValues];
    msg![env; values objectEnumerator]
}

@end

// Special variant for use by CFDictionary with NULL callbacks: objects aren't
// necessarily Objective-C objects and won't be retained/released.
// TODO: refactor with lookup/insert methods to use callbacks
@implementation _touchHLE_NSMutableDictionary_non_retaining: _touchHLE_NSMutableDictionary

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<CFDictionaryHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// our custom init, not a part of API
- (id)initWithKeyCallbacks:(ConstPtr<CFDictionaryKeyCallBacks>)key_callbacks
         andValueCallbacks:(ConstPtr<CFDictionaryValueCallBacks>)value_callbacks {
    if !key_callbacks.is_null() {
        env.objc.borrow_mut::<CFDictionaryHostObject>(this).key_callbacks =
            env.mem.read(key_callbacks);
    }
    if !value_callbacks.is_null() {
        env.objc.borrow_mut::<CFDictionaryHostObject>(this).value_callbacks =
            env.mem.read(value_callbacks);
    }
    this
}

- (())dealloc {
    let host_obj = env.objc.borrow::<CFDictionaryHostObject>(this);
    let callbacks = [
        host_obj.key_callbacks.retain.addr_with_thumb_bit(),
        host_obj.key_callbacks.release.addr_with_thumb_bit(),
        host_obj.value_callbacks.retain.addr_with_thumb_bit(),
        host_obj.value_callbacks.release.addr_with_thumb_bit(),
    ];
    env.diagnostic_collection_event(
        "dealloc-callback-dictionary",
        this.to_bits(),
        0,
        0,
        callbacks,
    );
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithObjectsAndKeys:(id)_first_object, ..._dots {
    todo!();
}
- (id)description {
    todo!();
}
- (id)copyWithZone:(NSZonePtr)_zone {
    todo!();
}
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    todo!();
}

- (id)objectForKey:(id)key {
    let host_obj: CFDictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let res = host_obj.lookup(env, key);
    let callbacks = [
        host_obj.key_callbacks.retain.addr_with_thumb_bit(),
        host_obj.key_callbacks.release.addr_with_thumb_bit(),
        host_obj.value_callbacks.retain.addr_with_thumb_bit(),
        host_obj.value_callbacks.release.addr_with_thumb_bit(),
    ];
    *env.objc.borrow_mut(this) = host_obj;
    env.diagnostic_collection_event(
        "lookup-callback-dictionary",
        this.to_bits(),
        key.to_bits(),
        res.to_bits(),
        callbacks,
    );
    res
}

- (bool)_touchHLEContainsKey:(id)key {
    let host_obj: CFDictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let present = host_obj.contains_key(env, key);
    *env.objc.borrow_mut(this) = host_obj;
    present
}

- (id)valueForKey:(id)_key {
    panic!("Unexpected call to valueForKey: for _touchHLE_NSMutableDictionary_non_retaining object {this:?}");
}

- (())setObject:(id)object
         forKey:(id)key {
    assert!(!key.is_null());
    let mut host_obj: CFDictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let callbacks = [
        host_obj.key_callbacks.retain.addr_with_thumb_bit(),
        host_obj.key_callbacks.release.addr_with_thumb_bit(),
        host_obj.value_callbacks.retain.addr_with_thumb_bit(),
        host_obj.value_callbacks.release.addr_with_thumb_bit(),
    ];
    host_obj.insert(env, key, object);
    *env.objc.borrow_mut(this) = host_obj;
    env.diagnostic_collection_event(
        "insert-callback-dictionary",
        this.to_bits(),
        key.to_bits(),
        object.to_bits(),
        callbacks,
    );
}

- (())removeObjectForKey:(id)key {
    assert!(!key.is_null());
    let mut host_obj: CFDictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let callbacks = [
        host_obj.key_callbacks.retain.addr_with_thumb_bit(),
        host_obj.key_callbacks.release.addr_with_thumb_bit(),
        host_obj.value_callbacks.retain.addr_with_thumb_bit(),
        host_obj.value_callbacks.release.addr_with_thumb_bit(),
    ];
    host_obj.remove(env, key);
    *env.objc.borrow_mut(this) = host_obj;
    env.diagnostic_collection_event(
        "remove-callback-dictionary",
        this.to_bits(),
        key.to_bits(),
        0,
        callbacks,
    );
}

- (id)allKeys {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(this));
    let keys: Vec<id> = host_obj.map.values().flatten().map(|&(key, _value)| key).collect();
    *env.objc.borrow_mut(this) = host_obj;

    let array: id = msg_class![env; _touchHLE_NSArray_non_retaining alloc];
    env.objc.borrow_mut::<ArrayHostObject>(array).array = keys;
    array
}

@end

};

/// Direct constructor for use by host code, similar to
/// `[[NSDictionary alloc] initWithObjectsAndKeys:]` but without variadics and
/// with a more intuitive argument order. Unlike [super::ns_array::from_vec],
/// this **does** copy and retain!
pub fn dict_from_keys_and_objects(env: &mut Environment, keys_and_objects: &[(id, id)]) -> id {
    let dict: id = msg_class![env; NSDictionary alloc];

    let mut host_object = <DictionaryHostObject as Default>::default();
    for &(key, object) in keys_and_objects {
        host_object.insert(env, key, object, /* copy_key: */ true);
    }
    *env.objc.borrow_mut(dict) = host_object;

    dict
}

/// Direct constructor for use by host code, similar to
/// `[[NSMutableDictionary alloc] initWithObjectsAndKeys:]` but without
/// variadics and with a more intuitive argument order.
/// Unlike [super::ns_array::mutable_from_vec], this **does** copy and retain!
pub fn mutable_dict_from_keys_and_objects(
    env: &mut Environment,
    keys_and_objects: &[(id, id)],
) -> id {
    let dict: id = msg_class![env; NSMutableDictionary alloc];

    let mut host_object = <DictionaryHostObject as Default>::default();
    for &(key, object) in keys_and_objects {
        host_object.insert(env, key, object, /* copy_key: */ true);
    }
    *env.objc.borrow_mut(dict) = host_object;

    dict
}

/// A helper to build a description NSString
/// for a NSDictionary or a NSMutableDictionary.
fn build_description(env: &mut Environment, dict: id) -> id {
    // According to docs, this description should be formatted as property list.
    // But by the same docs, it's meant to be used for debugging purposes only.
    let desc: id = msg_class![env; NSMutableString new];
    let prefix: id = from_rust_string(env, "{\n".to_string());
    () = msg![env; desc appendString:prefix];
    release(env, prefix);
    let keys: Vec<id> = env
        .objc
        .borrow_mut::<DictionaryHostObject>(dict)
        .iter_keys()
        .collect();
    for key in keys {
        let key_desc: id = msg![env; key description];
        let value: id = msg![env; dict objectForKey:key];
        let val_desc: id = msg![env; value description];
        // TODO: respect nesting and padding
        let format = format!(
            "\t{} = {};\n",
            to_rust_string(env, key_desc),
            to_rust_string(env, val_desc)
        );
        let format = from_rust_string(env, format);
        () = msg![env; desc appendString:format];
        release(env, format);
    }
    let suffix: id = from_rust_string(env, "}".to_string());
    () = msg![env; desc appendString:suffix];
    release(env, suffix);
    let desc_imm = msg![env; desc copy];
    release(env, desc);
    autorelease(env, desc_imm)
}

fn init_with_coder_inner(env: &mut Environment, dict: id, coder: id) -> id {
    let class: Class = msg![env; coder class];
    let keyed_unarch_class: Class = msg_class![env; NSKeyedUnarchiver class];
    let nib_archive_class: Class = msg_class![env; _touchHLE_NIBArchiveDecoder class];
    // It seems that every NSDictionary item in an NSKeyedArchiver plist looks
    // like:
    // {
    //   "$class" => (uid of NSDictionary class goes here),
    //   "NS.keys" => [
    //     // keys here
    //   ]
    //   "NS.objects" => [
    //     // objects here
    //   ]
    // }
    let tuples = if env.objc.class_is_subclass_of(class, keyed_unarch_class) {
        ns_keyed_unarchiver::decode_current_dict(env, coder)
    } else if env.objc.class_is_subclass_of(class, nib_archive_class) {
        _nib_archive_decoder::decode_current_dict(env, coder)
    } else {
        unimplemented!()
    };

    let mut host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(dict));
    assert!(host_obj.map.is_empty());
    for (key, val) in tuples {
        host_obj.insert(env, key, val, /* copy_key: */ true);
    }
    *env.objc.borrow_mut(dict) = host_obj;
    dict
}

fn encode_with_coder_inner(env: &mut Environment, dict: id, coder: id) {
    let host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(dict));
    let mut encoded_keys = vec![];
    let mut encoded_vals = vec![];
    for (k, _) in host_obj.map.values().flatten() {
        let kk = encode_object(env, coder, *k);
        encoded_keys.push(plist::Value::Uid(kk));
    }
    for (_, v) in host_obj.map.values().flatten() {
        let vv = encode_object(env, coder, *v);
        encoded_vals.push(plist::Value::Uid(vv));
    }
    *env.objc.borrow_mut(dict) = host_obj;

    let scope = get_value_to_encode_for_current_key(env, coder);
    scope.insert("NS.keys".to_string(), plist::Value::Array(encoded_keys));
    scope.insert("NS.objects".to_string(), plist::Value::Array(encoded_vals));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyed_subscript_reads_the_same_value_as_object_for_key() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let key = get_static_str(env, "key");
        let value = get_static_str(env, "value");
        let dictionary: id =
            msg_class![env; NSDictionary dictionaryWithObject:value forKey:key];

        let subscripted: id = msg![env; dictionary objectForKeyedSubscript:key];

        assert_eq!(subscripted, value);
    }

    #[test]
    fn objects_for_keys_preserves_order_and_uses_missing_marker() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let present = get_static_str(env, "present");
        let missing = get_static_str(env, "missing");
        let value = get_static_str(env, "value");
        let marker = get_static_str(env, "marker");
        let dictionary: id =
            msg_class![env; NSDictionary dictionaryWithObject:value forKey:present];
        let keys: id = msg_class![env; NSMutableArray arrayWithObject:present];
        let _: () = msg![env; keys addObject:missing];

        let values: id = msg![env; dictionary objectsForKeys:keys notFoundMarker:marker];

        assert_eq!(msg![env; values objectAtIndex:0u32], value);
        assert_eq!(msg![env; values objectAtIndex:1u32], marker);
    }

    #[test]
    fn mutable_singleton_factory_and_keyed_archive_preserve_mutability() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // Keyed archiving forwards Objective-C arguments through the guest ABI,
        // so the synthetic thread needs stack space for those nested calls.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let first_key = get_static_str(env, "first");
        let second_key = get_static_str(env, "second");
        let third_key = get_static_str(env, "third");
        let first_value = get_static_str(env, "one");
        let second_value = get_static_str(env, "two");
        let third_value = get_static_str(env, "three");

        let mutable: id =
            msg_class![env; NSMutableDictionary dictionaryWithObject:first_value forKey:first_key];
        let mutable_class: Class = msg_class![env; NSMutableDictionary class];
        let is_mutable: bool = msg![env; mutable isKindOfClass:mutable_class];
        assert!(is_mutable);
        () = msg![env; mutable setObject:second_value forKey:second_key];
        let count: NSUInteger = msg![env; mutable count];
        assert_eq!(count, 2);

        let data: id = msg_class![env; NSKeyedArchiver archivedDataWithRootObject:mutable];
        let decoded: id = msg_class![env; NSKeyedUnarchiver unarchiveObjectWithData:data];
        let decoded_is_mutable: bool = msg![env; decoded isKindOfClass:mutable_class];
        assert!(decoded_is_mutable);
        () = msg![env; decoded setObject:third_value forKey:third_key];
        let count: NSUInteger = msg![env; decoded count];
        assert_eq!(count, 3);

        let immutable: id =
            msg_class![env; NSDictionary dictionaryWithObject:first_value forKey:first_key];
        let immutable_is_mutable: bool = msg![env; immutable isKindOfClass:mutable_class];
        assert!(!immutable_is_mutable);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn keys_sorted_by_value_using_selector_returns_keys_in_value_order() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let low_key = get_static_str(env, "low");
        let middle_key = get_static_str(env, "middle");
        let high_key = get_static_str(env, "high");
        let low: id = msg_class![env; NSNumber numberWithInt:1i32];
        let middle: id = msg_class![env; NSNumber numberWithInt:5i32];
        let high: id = msg_class![env; NSNumber numberWithInt:9i32];
        let dictionary = dict_from_keys_and_objects(
            env,
            &[(middle_key, middle), (high_key, high), (low_key, low)],
        );
        let compare = env
            .objc
            .register_host_selector("compare:".to_string(), &mut env.mem);

        let sorted: id = msg![env; dictionary keysSortedByValueUsingSelector:compare];
        let count: NSUInteger = msg![env; sorted count];
        let first: id = msg![env; sorted objectAtIndex:0u32];
        let second: id = msg![env; sorted objectAtIndex:1u32];
        let third: id = msg![env; sorted objectAtIndex:2u32];
        assert_eq!(count, 3);
        assert_eq!(first, low_key);
        assert_eq!(second, middle_key);
        assert_eq!(third, high_key);

        release(env, dictionary);
        release(env, pool);
    }

    #[test]
    fn mutable_dictionary_removes_each_requested_key() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let one = get_static_str(env, "one");
        let two = get_static_str(env, "two");
        let value = get_static_str(env, "value");
        let dictionary: id =
            msg_class![env; NSMutableDictionary dictionaryWithObject:value forKey:one];
        () = msg![env; dictionary setObject:value forKey:two];
        let keys: id = msg_class![env; NSArray arrayWithObject:one];
        () = msg![env; dictionary removeObjectsForKeys:keys];
        let count: NSUInteger = msg![env; dictionary count];
        assert_eq!(count, 1);
        let removed: id = msg![env; dictionary objectForKey:one];
        let remaining: id = msg![env; dictionary objectForKey:two];
        assert_eq!(removed, nil);
        assert_eq!(remaining, value);
    }

    #[test]
    fn strings_file_description_quotes_and_escapes_entries() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let key = from_rust_string(&mut env, "a\"key".to_string());
        let value = from_rust_string(&mut env, "line 1\nline 2\\tail".to_string());
        let dictionary = dict_from_keys_and_objects(&mut env, &[(key, value)]);
        let description: id = msg![&mut env; dictionary descriptionInStringsFileFormat];
        assert_eq!(
            to_rust_string(&mut env, description),
            "\"a\\\"key\" = \"line 1\\nline 2\\\\tail\";\n"
        );
        release(&mut env, dictionary);
        release(&mut env, key);
        release(&mut env, value);
    }

    #[test]
    fn strings_file_description_serializes_number_values_without_quotes() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let key = get_static_str(env, "lives");
        let value: id = msg_class![env; NSNumber numberWithInt:3i32];
        let dictionary = dict_from_keys_and_objects(env, &[(key, value)]);
        let description: id = msg![env; dictionary descriptionInStringsFileFormat];
        assert_eq!(to_rust_string(env, description), "\"lives\" = 3;\n");
        release(env, dictionary);
    }

    #[test]
    fn property_list_dictionary_url_writer_rejects_nil_and_non_file_urls() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let key = get_static_str(env, "mode");
        let value = get_static_str(env, "arcade");
        let dictionary = dict_from_keys_and_objects(env, &[(key, value)]);
        let url_string = from_rust_string(env, "https://example.invalid/save.plist".to_owned());
        let url: id = msg_class![env; NSURL URLWithString:url_string];

        let nil_success: bool = msg![env; dictionary writeToURL:nil atomically:true];
        let network_success: bool = msg![env; dictionary writeToURL:url atomically:true];

        assert!(!nil_success);
        assert!(!network_success);
        release(env, dictionary);
        release(env, url_string);
    }

    #[test]
    fn dev_243_mutable_dictionary_subclass_allocation_preserves_requested_class_identity() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let subclass = env
            .objc
            .get_known_class("_touchHLE_NSMutableDictionary", &mut env.mem);
        let object = alloc_mutable_dictionary_class_cluster_member(
            env,
            subclass,
            MutVoidPtr::null(),
        );
        let actual_class: Class = msg![env; object class];
        assert_eq!(actual_class, subclass);
        assert!(env
            .objc
            .get_host_object(object)
            .unwrap()
            .as_any()
            .is::<TrivialHostObject>());
        env.objc.dealloc_object(object, &mut env.mem);
    }
}
