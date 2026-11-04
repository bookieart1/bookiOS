/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSSet` class cluster, including `NSMutableSet` and `NSCountedSet`.

use super::ns_array;
use super::ns_dictionary::DictionaryHostObject;
use super::ns_enumerator::{fast_enumeration_helper, NSFastEnumerationState};
use super::NSUInteger;
use crate::abi::DotDotDot;
use crate::environment::Environment;
use crate::mem::{ConstPtr, MutPtr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, retain, ClassExports, HostObject, NSZonePtr,
};

/// Belongs to _touchHLE_NSSet
#[derive(Debug, Default)]
struct SetHostObject {
    dict: DictionaryHostObject,
}
impl HostObject for SetHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSSet is an abstract class. A subclass must provide:
// - (NSUInteger)count;
// - (id)member:(id)object;
// - (NSEnumerator*)objectEnumerator;
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSSet.
@implementation NSSet: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSSet might be subclassed by something which needs allocWithZone:
    // to have the normal behaviour. Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSSet", &mut env.mem));
    msg_class![env; _touchHLE_NSSet allocWithZone:zone]
}

+ (id)set {
    let set: id = msg![env; this new];
    autorelease(env, set)
}

+ (id)setWithArray:(id)array { // NSArray *
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithArray:array];
    autorelease(env, new)
}

+ (id)setWithSet:(id)set {
    let objects: id = msg![env; set allObjects];
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithArray:objects];
    autorelease(env, new)
}

+ (id)setWithObject:(id)object {
    assert!(object != nil);
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithObject:object];
    autorelease(env, new)
}

+ (id)setWithObjects:(ConstPtr<id>)objects count:(NSUInteger)count {
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<SetHostObject>(new).dict = set_from_counted_objects(env, objects, count);
    autorelease(env, new)
}

+ (id)setWithObjects:(id)first_obj, ...args {
    assert!(this == env.objc.get_known_class("NSSet", &mut env.mem));
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<SetHostObject>(new).dict = set_from_objects(env, first_obj, args);
    autorelease(env, new)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

// NSMutableCopying always produces independent, mutable membership.
- (id)mutableCopyWithZone:(NSZonePtr)zone {
    let objects: id = msg![env; this allObjects];
    let copy: id = msg_class![env; NSMutableSet allocWithZone:zone];
    msg![env; copy initWithArray:objects]
}

- (bool)containsObject:(id)object {
    let enumerator: id = msg![env; this objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            return false;
        }
        if msg![env; next isEqual:object] {
            return true;
        }
    }
}

@end

// NSMutableSet is an abstract class. A subclass must provide everything
// NSSet provides, plus:
// - (void)addObject:(id)object;
// - (void)removeObject:(id)object;
// Note that it inherits from NSSet, so we must ensure we override any default
// methods that would be inappropriate for mutability.
@implementation NSMutableSet: NSSet

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSSet might be subclassed by something which needs allocWithZone:
    // to have the normal behaviour. Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSMutableSet", &mut env.mem));
    msg_class![env; _touchHLE_NSMutableSet allocWithZone:zone]
}

+ (id)setWithObjects:(id)first_obj, ...args {
    assert!(this == env.objc.get_known_class("NSMutableSet", &mut env.mem));
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<SetHostObject>(new).dict = set_from_objects(env, first_obj, args);
    autorelease(env, new)
}

+ (id)setWithObjects:(ConstPtr<id>)objects count:(NSUInteger)count {
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<SetHostObject>(new).dict = set_from_counted_objects(env, objects, count);
    autorelease(env, new)
}

+ (id)setWithCapacity:(NSUInteger)capacity {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCapacity:capacity];
    autorelease(env, new)
}

+ (id)setWithSet:(id)set {
    let objects: id = msg![env; set allObjects];
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithArray:objects];
    autorelease(env, new)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)zone {
    let objects: id = msg![env; this allObjects];
    let copy: id = msg_class![env; NSSet allocWithZone:zone];
    msg![env; copy initWithArray:objects]
}

@end

// NSCountedSet stores one NSNumber count per distinct member. Its inherited
// `count` is the number of distinct members; `countForObject:` exposes the
// multiplicity.
@implementation NSCountedSet: NSMutableSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(SetHostObject {
        dict: Default::default(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCapacity:(NSUInteger)_capacity {
    this
}

- (id)initWithArray:(id)array {
    let count: NSUInteger = msg![env; array count];
    for idx in 0..count {
        let object: id = msg![env; array objectAtIndex:idx];
        () = msg![env; this addObject:object];
    }
    this
}

- (())dealloc {
    std::mem::take(&mut env.objc.borrow_mut::<SetHostObject>(this).dict).release(env);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (NSUInteger)count {
    env.objc.borrow::<SetHostObject>(this).dict.count
}

- (NSUInteger)countForObject:(id)object {
    let host = std::mem::take(env.objc.borrow_mut::<SetHostObject>(this));
    let number = host.dict.lookup(env, object);
    let count = if number == nil {
        0
    } else {
        msg![env; number unsignedIntegerValue]
    };
    *env.objc.borrow_mut(this) = host;
    count
}

- (id)member:(id)object {
    if object == nil {
        return nil;
    }
    let host = std::mem::take(env.objc.borrow_mut::<SetHostObject>(this));
    let member = host
        .dict
        .iter_keys()
        .find(|&candidate| candidate == object || msg![env; candidate isEqual:object])
        .unwrap_or(nil);
    *env.objc.borrow_mut(this) = host;
    member
}

- (())addObject:(id)object {
    assert!(object != nil);
    let mut host = std::mem::take(env.objc.borrow_mut::<SetHostObject>(this));
    let old_number = host.dict.lookup(env, object);
    let old_count: NSUInteger = if old_number == nil {
        0
    } else {
        msg![env; old_number unsignedIntegerValue]
    };
    if old_number != nil {
        host.dict.remove(env, object);
    }
    let new_number: id = msg_class![env; NSNumber numberWithUnsignedInteger:(old_count + 1)];
    host.dict.insert(env, object, new_number, /* copy_key: */ false);
    *env.objc.borrow_mut(this) = host;
}

- (())removeObject:(id)object {
    let mut host = std::mem::take(env.objc.borrow_mut::<SetHostObject>(this));
    let old_number = host.dict.lookup(env, object);
    if old_number != nil {
        let old_count: NSUInteger = msg![env; old_number unsignedIntegerValue];
        if old_count <= 1 {
            host.dict.remove(env, object);
        } else {
            host.dict.remove(env, object);
            let new_number: id = msg_class![env; NSNumber numberWithUnsignedInteger:(old_count - 1)];
            host.dict.insert(env, object, new_number, /* copy_key: */ false);
        }
    }
    *env.objc.borrow_mut(this) = host;
}

- (())removeAllObjects {
    let mut old = std::mem::take(env.objc.borrow_mut::<SetHostObject>(this));
    old.dict.release(env);
}

- (id)anyObject {
    env.objc
        .borrow::<SetHostObject>(this)
        .dict
        .iter_keys()
        .next()
        .unwrap_or(nil)
}

- (id)allObjects {
    all_objects(env, this)
}

- (id)objectEnumerator {
    let array: id = msg![env; this allObjects];
    msg![env; array objectEnumerator]
}

- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    let objects: id = msg![env; this allObjects];
    let count: NSUInteger = msg![env; objects count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; objects objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

@end

// Our private subclass that is the single implementation of NSSet for the
// time being.
@implementation _touchHLE_NSSet: NSSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(SetHostObject {
        dict: Default::default(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithObject:(id)object {
    let null: id = msg_class![env; NSNull null];

    let mut dict = <DictionaryHostObject as Default>::default();
    dict.insert(env, object, null, /* copy_key: */ false);

    env.objc.borrow_mut::<SetHostObject>(this).dict = dict;

    this
}

- (id)initWithObjects:(id)first_obj, ...args {
    env.objc.borrow_mut::<SetHostObject>(this).dict = set_from_objects(env, first_obj, args);
    this
}

- (id)initWithArray:(id)array {
    env.objc.borrow_mut::<SetHostObject>(this).dict = set_from_array(env, array);
    this
}

- (())dealloc {
    std::mem::take(&mut env.objc.borrow_mut::<SetHostObject>(this).dict).release(env);
    env.objc.dealloc_object(this, &mut env.mem)
}

// TODO: more init methods, etc

// TODO: accessors
- (NSUInteger)count {
    env.objc.borrow_mut::<SetHostObject>(this).dict.count
}

- (id)anyObject {
    let object_or_none = env.objc.borrow_mut::<SetHostObject>(this).dict.iter_keys().next();
    match object_or_none {
        Some(object) => object,
        None => nil
    }
}

- (id)member:(id)object {
    find_member(env, this, object)
}

- (id)allObjects {
    all_objects(env, this)
}

- (id)objectEnumerator { // NSEnumerator*
    let array: id = msg![env; this allObjects];
    msg![env; array objectEnumerator]
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    // We assume that order in which objects are reported is consistent
    // between calls!
    let objects: id = msg![env; this allObjects];
    let count: NSUInteger = msg![env; objects count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; objects objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

@end

// Our private subclass that is the single implementation of NSMutableSet for
// the time being.
@implementation _touchHLE_NSMutableSet: NSMutableSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(SetHostObject {
        dict: Default::default(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithObject:(id)object {
    let null: id = msg_class![env; NSNull null];

    let mut dict = <DictionaryHostObject as Default>::default();
    dict.insert(env, object, null, /* copy_key: */ false);

    env.objc.borrow_mut::<SetHostObject>(this).dict = dict;

    this
}

- (id)initWithObjects:(id)first_obj, ...args {
    env.objc.borrow_mut::<SetHostObject>(this).dict = set_from_objects(env, first_obj, args);
    this
}

- (id)initWithArray:(id)array {
    env.objc.borrow_mut::<SetHostObject>(this).dict = set_from_array(env, array);
    this
}

- (id)initWithCapacity:(NSUInteger)_capacity {
    // TODO: capacity
    msg![env; this init]
}

- (())dealloc {
    std::mem::take(&mut env.objc.borrow_mut::<SetHostObject>(this).dict).release(env);
    env.objc.dealloc_object(this, &mut env.mem)
}

// TODO: init methods etc

- (NSUInteger)count {
    env.objc.borrow_mut::<SetHostObject>(this).dict.count
}

- (id)anyObject {
    let object_or_none = env.objc.borrow_mut::<SetHostObject>(this).dict.iter_keys().next();
    match object_or_none {
        Some(object) => object,
        None => nil
    }
}

- (id)member:(id)object {
    find_member(env, this, object)
}

- (id)allObjects {
    all_objects(env, this)
}

- (id)objectEnumerator { // NSEnumerator*
    let array: id = msg![env; this allObjects];
    msg![env; array objectEnumerator]
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    // TODO: check that set wasn't mutated!
    // We assume that order in which objects are reported is consistent
    // between calls!
    let objects: id = msg![env; this allObjects];
    let count: NSUInteger = msg![env; objects count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; objects objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

// TODO: more mutation methods

- (())addObject:(id)object {
    let null: id = msg_class![env; NSNull null];
    let mut host_obj: SetHostObject = std::mem::take(env.objc.borrow_mut(this));
    host_obj.dict.insert(env, object, null, /* copy_key: */ false);
    *env.objc.borrow_mut(this) = host_obj;
}

- (())addObjectsFromArray:(id)array { // NSArray *
    let count: NSUInteger = msg![env; array count];
    for i in 0..count {
        let object: id = msg![env; array objectAtIndex:i];
        () = msg![env; this addObject:object];
    }
}

- (())removeObject:(id)object {
    let mut host_obj: SetHostObject = std::mem::take(env.objc.borrow_mut(this));
    host_obj.dict.remove(env, object);
    *env.objc.borrow_mut(this) = host_obj;
}

- (())removeAllObjects {
    let mut old_host_obj = std::mem::replace(
        env.objc.borrow_mut(this),
        SetHostObject {
            dict: Default::default(),
        },
    );
    old_host_obj.dict.release(env);
}

- (())unionSet:(id)other { // NSSet *
    let enumerator: id = msg![env; other objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            break;
        }
        () = msg![env; this addObject:next];
    }
}

@end

};

/// An autoreleased snapshot owns its elements independently of the set.
/// This is also used by enumeration and collection copying.
fn all_objects(env: &mut Environment, set: id) -> id {
    let objects: Vec<id> = env
        .objc
        .borrow::<SetHostObject>(set)
        .dict
        .iter_keys()
        .collect();
    for &object in &objects {
        retain(env, object);
    }
    let array = ns_array::from_vec(env, objects);
    autorelease(env, array)
}

/// Return the actual object stored in the set that compares equal to `object`.
/// `containsObject:` alone is insufficient here: callers rely on the returned
/// member preserving the identity of the object already owned by the set.
fn find_member(env: &mut Environment, set: id, object: id) -> id {
    if object == nil {
        return nil;
    }
    let host = std::mem::take(env.objc.borrow_mut::<SetHostObject>(set));
    let member = host
        .dict
        .iter_keys()
        .find(|&candidate| candidate == object || msg![env; candidate isEqual:object])
        .unwrap_or(nil);
    *env.objc.borrow_mut(set) = host;
    member
}

/// Helper method shared between `initWithObjects:` of `_touchHLE_NSSet` and
/// `_touchHLE_NSMutableSet`
fn set_from_objects(env: &mut Environment, first_obj: id, args: DotDotDot) -> DictionaryHostObject {
    let null: id = msg_class![env; NSNull null];

    let mut dict = <DictionaryHostObject as Default>::default();
    dict.insert(env, first_obj, null, /* copy_key: */ false);
    let mut varargs = args.start();
    loop {
        let next_arg: id = varargs.next(env);
        if next_arg == nil {
            break;
        }
        dict.insert(env, next_arg, null, /* copy_key: */ false);
    }
    dict
}

/// Helper method shared between `initWithArray:` of `_touchHLE_NSSet` and
/// `_touchHLE_NSMutableSet`
fn set_from_array(env: &mut Environment, array: id) -> DictionaryHostObject {
    let null: id = msg_class![env; NSNull null];

    let mut dict = <DictionaryHostObject as Default>::default();
    let count: NSUInteger = msg![env; array count];
    for i in 0..count {
        let next: id = msg![env; array objectAtIndex:i];
        dict.insert(env, next, null, /* copy_key: */ false);
    }
    dict
}

fn set_from_counted_objects(
    env: &mut Environment,
    objects: ConstPtr<id>,
    count: NSUInteger,
) -> DictionaryHostObject {
    assert!(count == 0 || !objects.is_null());
    let null: id = msg_class![env; NSNull null];
    let mut dict = DictionaryHostObject::default();
    for index in 0..count {
        let object = env.mem.read(objects + index);
        assert_ne!(object, nil, "NSSet cannot contain nil");
        dict.insert(env, object, null, /* copy_key: */ false);
    }
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;
    use crate::mem::guest_size_of;
    use crate::objc::{release, Class};

    #[test]
    fn counted_factory_reads_exact_count_and_deduplicates() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let one: id = msg_class![env; NSObject new];
        let two: id = msg_class![env; NSObject new];
        let objects: MutPtr<id> = env.mem.alloc(3 * guest_size_of::<id>()).cast();
        env.mem.write(objects, one);
        env.mem.write(objects + 1, two);
        env.mem.write(objects + 2, one);
        let objects = objects.cast_const();
        let set: id = msg_class![env; NSSet setWithObjects:objects count:3u32];
        let count: NSUInteger = msg![env; set count];
        assert_eq!(count, 2);
        release(env, one);
        release(env, two);
    }

    #[test]
    fn concrete_sets_return_the_stored_equal_member() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stored = ns_string::from_rust_string(env, "mini-militia".to_owned());
        let equal = ns_string::from_rust_string(env, "mini-militia".to_owned());
        assert_ne!(stored, equal);

        let immutable: id = msg_class![env; NSSet setWithObject:stored];
        let mutable: id = msg_class![env; NSMutableSet setWithObject:stored];
        let immutable_member: id = msg![env; immutable member:equal];
        let mutable_member: id = msg![env; mutable member:equal];
        assert_eq!(immutable_member, stored);
        assert_eq!(mutable_member, stored);
        let missing: id = msg![env; mutable member:nil];
        assert_eq!(missing, nil);

        release(env, stored);
        release(env, equal);
    }

    #[test]
    fn counted_set_tracks_multiplicity_and_distinct_count() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let object = ns_string::from_rust_string(env, "rolando".to_owned());
        let set: id = msg_class![env; NSCountedSet new];
        () = msg![env; set addObject:object];
        () = msg![env; set addObject:object];
        let distinct: NSUInteger = msg![env; set count];
        let multiplicity: NSUInteger = msg![env; set countForObject:object];
        assert_eq!(distinct, 1);
        assert_eq!(multiplicity, 2);
        () = msg![env; set removeObject:object];
        let multiplicity: NSUInteger = msg![env; set countForObject:object];
        assert_eq!(multiplicity, 1);
        release(env, object);
        release(env, set);
    }

    #[test]
    fn copies_preserve_element_identity_and_independent_membership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        // NSObject has no copy implementation: elements must be retained,
        // never copied, even when the container becomes immutable.
        let element: id = msg_class![env; NSObject new];
        let source: id = msg_class![env; NSMutableSet alloc];
        let source: id = msg![env; source initWithObject:element];
        let mutable: id = msg![env; source mutableCopy];
        let immutable: id = msg![env; source copy];
        let another: id = msg![env; immutable mutableCopy];
        let retained: id = msg![env; immutable copy];
        assert_eq!(retained, immutable);
        release(env, retained);
        assert_ne!(mutable, source);
        assert_ne!(another, immutable);
        let mutable_class: Class = msg_class![env; NSMutableSet class];
        let is_mutable: bool = msg![env; mutable isKindOfClass:mutable_class];
        let is_immutable_mutable: bool = msg![env; immutable isKindOfClass:mutable_class];
        assert!(is_mutable);
        assert!(!is_immutable_mutable);
        for set in [source, mutable, immutable, another] {
            let actual: id = msg![env; set anyObject];
            assert_eq!(actual, element);
            let count: NSUInteger = msg![env; set count];
            assert_eq!(count, 1);
        }
        () = msg![env; source removeAllObjects];
        let count: NSUInteger = msg![env; mutable count];
        assert_eq!(count, 1);
        () = msg![env; mutable removeObject:element];
        let count: NSUInteger = msg![env; immutable count];
        assert_eq!(count, 1);
        let enumerator: id = msg![env; immutable objectEnumerator];
        let actual: id = msg![env; enumerator nextObject];
        assert_eq!(actual, element);
        let actual: id = msg![env; enumerator nextObject];
        assert_eq!(actual, nil);
        release(env, source);
        release(env, mutable);
        release(env, another);
        release(env, pool); // Temporary arrays/enumerators must balance.
        assert_eq!(env.objc.get_refcount(element).get(), 2);
        let actual: id = msg![env; immutable anyObject];
        assert_eq!(actual, element);
        release(env, immutable);
        assert_eq!(env.objc.get_refcount(element).get(), 1);
        release(env, element);
    }

    #[test]
    fn set_snapshots_own_elements_and_empty_copies_are_mutable() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let element: id = msg_class![env; NSObject new];
        let source: id = msg_class![env; NSSet alloc];
        let source: id = msg![env; source initWithObject:element];
        let snapshot: id = msg![env; source allObjects];
        retain(env, snapshot);
        let copy: id = msg![env; source mutableCopy];
        release(env, copy); // Opposite release order from the previous test.
        release(env, source);
        release(env, pool);
        assert_eq!(env.objc.get_refcount(element).get(), 2);
        let actual: id = msg![env; snapshot objectAtIndex:0u32];
        assert_eq!(actual, element);
        release(env, snapshot);
        assert_eq!(env.objc.get_refcount(element).get(), 1);

        let pool: id = msg_class![env; NSAutoreleasePool new];
        let empty: id = msg_class![env; NSSet new];
        let copy: id = msg![env; empty mutableCopy];
        () = msg![env; copy addObject:element];
        let count: NSUInteger = msg![env; empty count];
        assert_eq!(count, 0);
        let actual: id = msg![env; copy anyObject];
        assert_eq!(actual, element);
        release(env, copy);
        release(env, empty);
        release(env, pool);
        assert_eq!(env.objc.get_refcount(element).get(), 1);
        release(env, element);
    }

    #[test]
    fn add_objects_from_array_adds_each_unique_object() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let first: id = msg_class![env; NSObject new];
        let second: id = msg_class![env; NSObject new];
        let array: id = msg_class![env; NSMutableArray arrayWithObject:first];
        () = msg![env; array addObject:second];
        () = msg![env; array addObject:first];
        let set: id = msg_class![env; NSMutableSet new];

        () = msg![env; set addObjectsFromArray:array];

        let count: NSUInteger = msg![env; set count];
        let contains_first: bool = msg![env; set containsObject:first];
        let contains_second: bool = msg![env; set containsObject:second];
        assert_eq!(count, 2);
        assert!(contains_first);
        assert!(contains_second);
        release(env, set);
        release(env, first);
        release(env, second);
        release(env, pool);
    }
}
