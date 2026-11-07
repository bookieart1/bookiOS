/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSArray` class cluster, including `NSMutableArray`.

use super::ns_enumerator::{fast_enumeration_helper, NSFastEnumerationState};
use super::ns_property_list_serialization::{
    deserialize_plist_from_file, NSPropertyListBinaryFormat_v1_0,
};
use super::{
    _nib_archive_decoder, ns_keyed_unarchiver, ns_string, ns_url, NSComparisonResult, NSNotFound,
    NSRange, NSUInteger,
};
use crate::abi::{CallFromHost, DotDotDot, GuestFunction};
use crate::frameworks::foundation::ns_keyed_archiver::{
    encode_object, get_value_to_encode_for_current_key,
};
use crate::fs::GuestPath;
use crate::libc::stdlib::qsort::qsort_generic;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, nil, objc_classes, release, retain, Class,
    ClassExports, HostObject, NSZonePtr, SEL,
};
use crate::Environment;

struct ObjectEnumeratorHostObject {
    /// the enumerated collection, NSArray *
    array: id,
    /// an iterator
    iterator: std::vec::IntoIter<id>,
}
impl HostObject for ObjectEnumeratorHostObject {}

/// Belongs to _touchHLE_NSArray
#[derive(Debug, Default)]
pub(super) struct ArrayHostObject {
    pub(super) array: Vec<id>,
    /// Core Foundation mutable collections use a positive capacity as a
    /// fixed upper bound. Foundation's `initWithCapacity:` is only a storage
    /// hint, so ordinary NSMutableArray instances leave this unset.
    fixed_capacity: Option<usize>,
}
impl HostObject for ArrayHostObject {}

/// The fixed header of a 32-bit Apple block literal. Captured variables, if
/// any, follow this header and are accessed by the block's invoke function.
#[repr(C)]
struct BlockLiteral {
    _isa: MutVoidPtr,
    _flags: i32,
    _reserved: i32,
    invoke: GuestFunction,
    _descriptor: MutVoidPtr,
}
unsafe impl SafeRead for BlockLiteral {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSArray is an abstract class. A subclass must provide:
// - (NSUInteger)count;
// - (id)objectAtIndex:(NSUInteger)index;
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSArray.
@implementation NSArray: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSArray might be subclassed by something which needs allocWithZone:
    // to have the normal behaviour. Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSArray", &mut env.mem));
    msg_class![env; _touchHLE_NSArray allocWithZone:zone]
}

+ (id)array {
    let array: id = msg![env; this new];
    autorelease(env, array)
}

+ (id)arrayWithArray:(id)other { // NSArray*
    let array: id = msg![env; this alloc];
    let array: id = msg![env; array initWithArray:other];
    autorelease(env, array)
}

// These probably comes from some category related to plists.
+ (id)arrayWithContentsOfFile:(id)path { // NSString*
    let array: id = msg![env; this alloc];
    let array: id = msg![env; array initWithContentsOfFile:path];
    autorelease(env, array)
}
+ (id)arrayWithContentsOfURL:(id)url { // NSURL*
    let array: id = msg![env; this alloc];
    let array: id = msg![env; array initWithContentsOfURL:url];
    autorelease(env, array)
}

+ (id)arrayWithObject:(id)object {
    assert_ne!(object, nil, "arrayWithObject: requires a non-nil object");
    retain(env, object);
    let objects = vec![object];
    let array = from_vec(env, objects);
    autorelease(env, array)
}
+ (id)arrayWithObjects:(id)first_obj, ...args {
    let objects = if first_obj == nil {
        vec![]
    } else {
        retain(env, first_obj);
        let mut objects = vec![first_obj];
        let mut varargs = args.start();
        loop {
            let next_arg: id = varargs.next(env);
            if next_arg.is_null() {
                break;
            }
            retain(env, next_arg);
            objects.push(next_arg);
        }
        objects
    };
    let array = from_vec(env, objects);
    autorelease(env, array)
}
+ (id)arrayWithObjects:(ConstPtr<id>)objects_ptr count:(NSUInteger)count {
    let array: id = msg![env; this alloc];
    let array: id = msg![env; array initWithObjects:objects_ptr count:count];
    autorelease(env, array)
}

- (id)filteredArrayUsingPredicate:(id)predicate { // NSPredicate*
    let count: NSUInteger = msg![env; this count];
    let mut objects = Vec::new();
    for index in 0..count {
        let object: id = msg![env; this objectAtIndex:index];
        let include: bool = msg![env; predicate evaluateWithObject:object];
        if include {
            retain(env, object);
            objects.push(object);
        }
    }
    let filtered = from_vec(env, objects);
    autorelease(env, filtered)
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
        /* array_expected: */ true,
    )
}
- (id)initWithContentsOfURL:(id)url { // NSURL*
    release(env, this);
    if url == nil {
        return nil;
    }
    let path = ns_url::to_rust_path(env, url);
    deserialize_plist_from_file(env, &path, /* array_expected: */ true)
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
        "[(NSArray *){:?} writeToFile:{:?} atomically:{}] -> {}",
        this,
        ns_string::to_rust_string(env, path),
        atomically,
        res
    );
    res
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (NSUInteger)indexOfObject:(id)object {
    let count: NSUInteger = msg![env; this count];
    for i in 0..count {
        let curr_object: id = msg![env; this objectAtIndex:i];
        let equal: bool = msg![env; object isEqual:curr_object];
        if equal {
            return i;
        }
    }
    NSNotFound as NSUInteger
}
- (bool)containsObject:(id)object {
    let idx: NSUInteger = msg![env; this indexOfObject:object];
    idx != NSNotFound as NSUInteger
}

- (id)arrayByAddingObjectsFromArray:(id)other { // NSArray *
    let own_count: NSUInteger = msg![env; this count];
    let other_count: NSUInteger = msg![env; other count];
    let mut objects = Vec::with_capacity((own_count + other_count) as usize);
    for index in 0..own_count {
        let object: id = msg![env; this objectAtIndex:index];
        objects.push(retain(env, object));
    }
    for index in 0..other_count {
        let object: id = msg![env; other objectAtIndex:index];
        objects.push(retain(env, object));
    }
    let combined = from_vec(env, objects);
    autorelease(env, combined)
}

- (id)firstObject {
    let size: NSUInteger = msg![env; this count];
    if size == 0 {
        return nil;
    }
    msg![env; this objectAtIndex:0u32]
}

- (id)lastObject {
    let size: NSUInteger = msg![env; this count];
    if size == 0 {
        return nil;
    }
    msg![env; this objectAtIndex:(size - 1)]
}

- (id)componentsJoinedByString:(id)str { // NSString *
    let res: id = msg_class![env; NSMutableString new];
    let count: NSUInteger = msg![env; this count];
    if count == 0 {
        autorelease(env, res);
        return res;
    }
    for i in 0..count {
        let curr_object: id = msg![env; this objectAtIndex:i];
        let curr_desc: id = msg![env; curr_object description];
        () = msg![env; res appendString:curr_desc];
        if i != count-1 {
            () = msg![env; res appendString:str];
        }
    }
    let res_imm = msg![env; res copy];
    release(env, res);
    autorelease(env, res_imm)
}

- (id)sortedArrayUsingFunction:(GuestFunction)comparator
                       context:(MutVoidPtr)context {
    let array = msg![env; this mutableCopy];
    () = msg![env; array sortUsingFunction:comparator context:context];
    let array_imm = msg![env; array copy];
    release(env, array);
    autorelease(env, array_imm)
}

- (id)sortedArrayUsingComparator:(MutPtr<BlockLiteral>)comparator {
    let array: id = msg![env; this mutableCopy];
    () = msg![env; array sortUsingComparator:comparator];
    let array_imm: id = msg![env; array copy];
    release(env, array);
    autorelease(env, array_imm)
}

- (())makeObjectsPerformSelector:(SEL)sel {
    let count: NSUInteger = msg![env; this count];
    env.diagnostic_array_event("perform-begin", this.to_bits(), 0, 0, count, count);
    for idx in 0..count {
        let obj: id = msg![env; this objectAtIndex:idx];
        let before: NSUInteger = msg![env; this count];
        env.diagnostic_array_event(
            "perform-callback",
            this.to_bits(),
            idx,
            obj.to_bits(),
            before,
            before,
        );
        let _: id = msg![env; obj performSelector:sel];
        let after: NSUInteger = msg![env; this count];
        env.diagnostic_array_event(
            "perform-return",
            this.to_bits(),
            idx,
            obj.to_bits(),
            before,
            after,
        );
    }
    let after: NSUInteger = msg![env; this count];
    env.diagnostic_array_event("perform-end", this.to_bits(), count, 0, count, after);
}

- (())makeObjectsPerformSelector:(SEL)sel withObject:(id)object {
    let count: NSUInteger = msg![env; this count];
    env.diagnostic_array_event(
        "perform-object-begin",
        this.to_bits(),
        0,
        object.to_bits(),
        count,
        count,
    );
    for idx in 0..count {
        let obj: id = msg![env; this objectAtIndex:idx];
        let before: NSUInteger = msg![env; this count];
        env.diagnostic_array_event(
            "perform-object-callback",
            this.to_bits(),
            idx,
            obj.to_bits(),
            before,
            before,
        );
        let _: id = msg![env; obj performSelector:sel withObject:object];
        let after: NSUInteger = msg![env; this count];
        env.diagnostic_array_event(
            "perform-object-return",
            this.to_bits(),
            idx,
            obj.to_bits(),
            before,
            after,
        );
    }
    let after: NSUInteger = msg![env; this count];
    env.diagnostic_array_event(
        "perform-object-end",
        this.to_bits(),
        count,
        object.to_bits(),
        count,
        after,
    );
}

- (NSUInteger)hash {
    // TODO: define better hash
    msg![env; this count]
}
- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSArray class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
    msg![env; this isEqualToArray:other]
}
- (bool)isEqualToArray:(id)other { // NSArray *
    if other == nil {
        return false;
    }
    let count: NSUInteger = msg![env; this count];
    let other_count: NSUInteger = msg![env; other count];
    if count != other_count {
        return false;
    }
    for i in 0..count {
        let curr_object: id = msg![env; this objectAtIndex:i];
        let curr_other_object: id = msg![env; other objectAtIndex:i];
        let equal: bool = msg![env; curr_object isEqual:curr_other_object];
        if !equal {
            return false;
        }
    }
    true
}

@end

// NSMutableArray is an abstract class. A subclass must provide everything
// NSArray provides, plus:
// - (void)insertObject:(id)object atIndex:(NSUInteger)index;
// - (void)removeObjectAtIndex:(NSUInteger)index;
// - (void)addObject:(id)object;
// - (void)removeLastObject
// - (void)replaceObjectAtIndex:(NSUInteger)index withObject:(id)object;
// Note that it inherits from NSArray, so we must ensure we override any default
// methods that would be inappropriate for mutability.
@implementation NSMutableArray: NSArray

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSArray might be subclassed by something which needs allocWithZone:
    // to have the normal behaviour. Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSMutableArray", &mut env.mem));
    msg_class![env; _touchHLE_NSMutableArray allocWithZone:zone]
}

+ (id)arrayWithCapacity:(NSUInteger)capacity {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCapacity:capacity];
    autorelease(env, new)
}

+ (id)arrayWithObject:(id)object {
    assert_ne!(object, nil, "arrayWithObject: requires a non-nil object");
    let array: id = msg![env; this arrayWithCapacity:1u32];
    () = msg![env; array addObject:object];
    array
}

+ (id)arrayWithArray:(id)array {
    let new: id = msg![env; this alloc];
    () = msg![env; new addObjectsFromArray:array];
    autorelease(env, new)
}

+ (id)arrayWithObjects:(id)firstObj, ...args {
    retain(env, firstObj);
    let mut objects = vec![firstObj];
    let mut varargs = args.start();
    loop {
        let next_arg: id = varargs.next(env);
        if next_arg.is_null() {
            break;
        }
        retain(env, next_arg);
        objects.push(next_arg);
    }
    let array = mutable_from_vec(env, objects);
    autorelease(env, array)
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
        /* array_expected: */ true,
    );
    if tmp == nil {
        return nil;
    }
    // We should respect mutability of the top most container!
    let res = msg_class![env; NSMutableArray alloc];
    let res = msg![env; res initWithArray:tmp];
    release(env, tmp);
    res
}
- (id)initWithContentsOfURL:(id)url { // NSURL*
    release(env, this);
    if url == nil {
        return nil;
    }
    let path = ns_url::to_rust_path(env, url);
    let tmp = deserialize_plist_from_file(env, &path, /* array_expected: */ true);
    if tmp == nil {
        return nil;
    }
    // We should respect mutability of the top most container!
    let res = msg_class![env; NSMutableArray alloc];
    let res = msg![env; res initWithArray:tmp];
    release(env, tmp);
    res
}

- (())addObjectsFromArray:(id)other { // NSArray*
    let enumerator: id = msg![env; other objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            break;
        }
        () = msg![env; this addObject:next];
    }
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    let other: id = msg_class![env; NSArray alloc];
    let other: id = msg![env; other initWithArray:this];
    other
}

@end

// Our private subclass that is the single implementation of NSArray for the
// time being.
@implementation _touchHLE_NSArray: NSArray

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(ArrayHostObject {
        array: Vec::new(),
        fixed_capacity: None,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    init_with_coder_inner(env, this, coder)
}
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

- (id)initWithArray:(id)array { // NSArray*
    let mut objects = Vec::new();
    let enumerator: id = msg![env; array objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            break;
        }
        objects.push(next);
        retain(env, next);
    }
    env.objc.borrow_mut::<ArrayHostObject>(this).array = objects;
    this
}

- (id)initWithObjects:(id)first_obj, ...args {
    init_with_objects_inner(env, this, first_obj, args);
    this
}

- (id)initWithObjects:(ConstPtr<id>)objects_ptr count:(NSUInteger)count {
    let mut objects = Vec::new();
    for i in 0..count {
        let obj: id = env.mem.read(objects_ptr + i);
        retain(env, obj);
        objects.push(obj);
    }
    env.objc.borrow_mut::<ArrayHostObject>(this).array = objects;
    this
}

- (())dealloc {
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let array = std::mem::take(&mut host_object.array);

    for object in array {
        release(env, object);
    }

    env.objc.dealloc_object(this, &mut env.mem)
}

// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    mutable_copy_inner(env, this)
}

- (id)objectEnumerator { // NSEnumerator*
    object_enumerator_inner(env, this)
}
- (id)reverseObjectEnumerator { // NSEnumerator*
    reverse_object_enumerator_inner(env, this)
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    let count: NSUInteger = msg![env; this count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; this objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

// TODO: more init methods, etc

- (NSUInteger)count {
    env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap()
}
- (id)objectAtIndex:(NSUInteger)index {
    // TODO: throw real exception rather than panic if out-of-bounds?
    let count: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    if index >= count {
        env.diagnostic_array_event(
            "invalid-access",
            this.to_bits(),
            index,
            0,
            count,
            count,
        );
    }
    env.objc.borrow::<ArrayHostObject>(this).array[index as usize]
}

- (id)description {
    build_description(env, this)
}

- (id)subarrayWithRange:(NSRange)range {
    let mut tmp = Vec::new();
    tmp.extend_from_slice(
        &env.objc.borrow::<ArrayHostObject>(this).array[range.location as usize..(range.location + range.length) as usize]
    );
    for &obj in &tmp {
        retain(env, obj);
    }
    let res = from_vec(env, tmp);
    autorelease(env, res)
}

- (id)sortedArrayUsingSelector:(SEL)comparator {
    let new = msg![env; this mutableCopy];
    () = msg![env; new sortUsingSelector:comparator];
    autorelease(env, new)
}

- (id)sortedArrayUsingDescriptors:(id)descriptors { // NSArray<NSSortDescriptor*>*
    let new = msg![env; this mutableCopy];
    () = msg![env; new sortUsingDescriptors:descriptors];
    autorelease(env, new)
}

@end

// Special variant for use by CFArray with NULL callbacks: objects aren't
// necessarily Objective-C objects and won't be retained/released.
@implementation _touchHLE_NSArray_non_retaining: _touchHLE_NSArray

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation _touchHLE_NSArray_ObjectEnumerator: NSEnumerator

- (id)nextObject {
    let host_obj = env.objc.borrow_mut::<ObjectEnumeratorHostObject>(this);
    host_obj.iterator.next().unwrap_or(nil)
}

- (())dealloc {
    let host_obj = env.objc.borrow::<ObjectEnumeratorHostObject>(this);
    release(env, host_obj.array);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

// Our private subclass that is the single implementation of NSMutableArray for
// the time being.
@implementation _touchHLE_NSMutableArray: NSMutableArray

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(ArrayHostObject {
        array: Vec::new(),
        fixed_capacity: None,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCapacity:(NSUInteger)capacity {
    env.objc.borrow_mut::<ArrayHostObject>(this).array.reserve(capacity as usize);
    this
}

- (id)initWithObjects:(id)first_obj, ...args {
    init_with_objects_inner(env, this, first_obj, args);
    this
}

- (id)initWithObjects:(ConstPtr<id>)objects_ptr count:(NSUInteger)count {
    let mut objects = Vec::new();
    for i in 0..count {
        let object: id = env.mem.read(objects_ptr + i);
        retain(env, object);
        objects.push(object);
    }
    env.objc.borrow_mut::<ArrayHostObject>(this).array = objects;
    this
}

- (id)initWithArray:(id)array { // NSArray*
    let mut objects = Vec::new();
    let enumerator: id = msg![env; array objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            break;
        }
        objects.push(next);
        retain(env, next);
    }
    env.objc.borrow_mut::<ArrayHostObject>(this).array = objects;
    this
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    init_with_coder_inner(env, this, coder)
}
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    let arr: id = msg_class![env; NSArray alloc];
    let array = env.objc.borrow::<ArrayHostObject>(this).array.clone();
    for &object in &array {
        retain(env, object);
    }
    env.objc.borrow_mut::<ArrayHostObject>(arr).array = array;
    arr
}

// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    mutable_copy_inner(env, this)
}

- (())dealloc {
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let array = std::mem::take(&mut host_object.array);

    for object in array {
        release(env, object);
    }

    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)objectEnumerator { // NSEnumerator*
    object_enumerator_inner(env, this)
}
- (id)reverseObjectEnumerator { // NSEnumerator*
    reverse_object_enumerator_inner(env, this)
}

- (())sortUsingFunction:(GuestFunction)comparator
                context:(MutVoidPtr)context {
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let mut array = std::mem::take(&mut host_object.array);
    let len = array.len().try_into().unwrap();
    let mut user_data = (env, &mut array);
    qsort_generic(
        &mut user_data,
        len,
        &mut |(env, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            comparator.call_from_host(env, (array[l], array[r], context))
        },
        &mut |(_, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            array.swap(l, r);
        },
    );
    let (env, _) = user_data;
    env.objc.borrow_mut::<ArrayHostObject>(this).array = array;
}

- (())sortUsingComparator:(MutPtr<BlockLiteral>)comparator {
    assert!(!comparator.is_null());
    let invoke = env.mem.read(comparator).invoke;
    assert_ne!(invoke.addr_without_thumb_bit(), 0);

    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let mut array = std::mem::take(&mut host_object.array);
    let len = array.len().try_into().unwrap();
    let mut user_data = (env, &mut array);
    qsort_generic(
        &mut user_data,
        len,
        &mut |(env, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            invoke.call_from_host(env, (comparator.cast_void(), array[l], array[r]))
        },
        &mut |(_, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            array.swap(l, r);
        },
    );

    let (env, _) = user_data;
    env.objc.borrow_mut::<ArrayHostObject>(this).array = array;
}

- (())sortUsingSelector:(SEL)comparator {
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let mut array = std::mem::take(&mut host_object.array);
    let len = array.len().try_into().unwrap();
    let mut user_data = (env, &mut array);
    qsort_generic(
        &mut user_data,
        len,
        &mut |(env, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            let res: NSComparisonResult = msg_send(env, (array[l], comparator, array[r]));
            res
        },
        &mut |(_, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            array.swap(l, r);
        },
    );

    let (env, _) = user_data;
    env.objc.borrow_mut::<ArrayHostObject>(this).array = array;
}

- (())sortUsingDescriptors:(id)descriptors { // NSArray<NSSortDescriptor*>*
    let descriptors = env.objc.borrow::<ArrayHostObject>(descriptors).array.clone();
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let mut array = std::mem::take(&mut host_object.array);
    let len = array.len().try_into().unwrap();
    let mut user_data = (env, &mut array);
    qsort_generic(
        &mut user_data,
        len,
        &mut |(env, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            for &descriptor in &descriptors {
                let res: NSComparisonResult =
                    msg![env; descriptor compareObject:(array[l]) toObject:(array[r])];
                if res != 0 {
                    return res;
                }
            }
            0
        },
        &mut |(_, array), l, r| {
            let (l, r): (usize, usize) = (l.try_into().unwrap(), r.try_into().unwrap());
            array.swap(l, r);
        },
    );

    let (env, _) = user_data;
    env.objc.borrow_mut::<ArrayHostObject>(this).array = array;
}

// NSFastEnumeration implementation
- (NSUInteger)countByEnumeratingWithState:(MutPtr<NSFastEnumerationState>)state
                                  objects:(MutPtr<id>)stackbuf
                                    count:(NSUInteger)len {
    // TODO: check that array wasn't mutated!
    let count: NSUInteger = msg![env; this count];
    fast_enumeration_helper(env, this, |env, idx| {
        if idx < count {
            msg![env; this objectAtIndex:idx]
        } else {
            nil
        }
    }, state, stackbuf, len)
}

- (NSUInteger)count {
    env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap()
}
- (id)objectAtIndex:(NSUInteger)index {
    // TODO: throw real exception rather than panic if out-of-bounds?
    let count: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    if index >= count {
        env.diagnostic_array_event(
            "invalid-access",
            this.to_bits(),
            index,
            0,
            count,
            count,
        );
    }
    env.objc.borrow::<ArrayHostObject>(this).array[index as usize]
}

- (id)description {
    build_description(env, this)
}

// TODO: more mutation methods

- (())insertObject:(id)object
           atIndex:(NSUInteger)index {
    assert_can_grow(env, this);
    let before: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    retain(env, object);
    env.objc.borrow_mut::<ArrayHostObject>(this).array.insert(index as usize, object);
    env.diagnostic_array_event("insert", this.to_bits(), index, object.to_bits(), before, before + 1);
}

- (())addObject:(id)object {
    assert_can_grow(env, this);
    let before: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    retain(env, object);
    env.objc.borrow_mut::<ArrayHostObject>(this).array.push(object);
    env.diagnostic_array_event("add", this.to_bits(), before, object.to_bits(), before, before + 1);
}

- (())removeObject:(id)object {
    let mut to_remove = Vec::new();
    let count: NSUInteger = msg![env; this count];
    for i in 0..count {
        let curr_object: id = msg![env; this objectAtIndex:i];
        let equal: bool = msg![env; object isEqual:curr_object];
        if equal {
            to_remove.push(i);
        }
    }
    // TODO: runtime here is O(n^2), it could be O(n) instead
    // Preserve the original indexes while removing every equal object.
    for i in to_remove.into_iter().rev() {
        () = msg![env; this removeObjectAtIndex:i];
    }
}

- (())removeObjectIdenticalTo:(id)object {
    let mut to_remove = Vec::new();
    let count: NSUInteger = msg![env; this count];
    for i in 0..count {
        let current: id = msg![env; this objectAtIndex:i];
        if current == object {
            to_remove.push(i);
        }
    }
    for i in to_remove.into_iter().rev() {
        () = msg![env; this removeObjectAtIndex:i];
    }
}

- (())removeObjectAtIndex:(NSUInteger)index {
    let before: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    let object = env.objc.borrow_mut::<ArrayHostObject>(this).array.remove(index as usize);
    env.diagnostic_array_event("remove", this.to_bits(), index, object.to_bits(), before, before - 1);
    release(env, object)
}

- (())replaceObjectAtIndex:(NSUInteger)index withObject:(id)obj {
    let count: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    retain(env, obj);
    let object = std::mem::replace(&mut env.objc.borrow_mut::<ArrayHostObject>(this).array[index as usize], obj);
    env.diagnostic_array_event("replace", this.to_bits(), index, obj.to_bits(), count, count);
    release(env, object);
}

- (())removeLastObject {
    // Apple's docs say this raises on an empty array, but it does nothing.
    let before: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    if let Some(object) = env.objc.borrow_mut::<ArrayHostObject>(this).array.pop() {
        env.diagnostic_array_event("remove-last", this.to_bits(), before - 1, object.to_bits(), before, before - 1);
        release(env, object)
    }
}

- (())removeAllObjects {
    let before: NSUInteger = env.objc.borrow::<ArrayHostObject>(this).array.len().try_into().unwrap();
    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let array = std::mem::take(&mut host_object.array);
    for object in array {
        release(env, object);
    }

    env.objc.borrow_mut::<ArrayHostObject>(this).array = Vec::new();
    env.diagnostic_array_event("remove-all", this.to_bits(), 0, 0, before, 0)
}

- (())removeObjectsInArray:(id)other { // NSArray*
    let other_count: NSUInteger = msg![env; other count];
    if other_count == 0 {
        return;
    }

    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(this);
    let array = std::mem::take(&mut host_object.array);
    let before: NSUInteger = array.len().try_into().unwrap();

    // Since removing objects individually has to iterate the whole array
    // for each object (with [array removeObject:obj]), so this just saves some
    // runtime costs.
    let other_array_set: id = msg_class![env; NSSet alloc];
    let other_array_set: id = msg![env; other_array_set initWithArray:other];
    let mut new_array = Vec::with_capacity(array.len());
    for object in array {
        let contains: bool = msg![env; other_array_set containsObject:object];
        if contains {
            release(env, object);
        } else {
            new_array.push(object);
        }
    }

    let after: NSUInteger = new_array.len().try_into().unwrap();
    env.objc.borrow_mut::<ArrayHostObject>(this).array = new_array;
    env.diagnostic_array_event("remove-array", this.to_bits(), 0, other.to_bits(), before, after);
    release(env, other_array_set);
}

- (())removeObjectsAtIndexes:(id)indexes { // NSIndexSet*
    assert!(indexes != nil, "removeObjectsAtIndexes: requires a non-nil index set");

    // Validate the complete set before mutating so an invalid high index does
    // not leave the receiver partially changed. Removing in descending order
    // then preserves the meaning of every remaining original index.
    let last: NSUInteger = msg![env; indexes lastIndex];
    if last == NSNotFound as NSUInteger {
        return;
    }
    let count: NSUInteger = msg![env; this count];
    assert!(last < count, "removeObjectsAtIndexes: index out of range");

    let mut index = last;
    loop {
        () = msg![env; this removeObjectAtIndex:index];
        let previous: NSUInteger = msg![env; indexes indexLessThanIndex:index];
        if previous == NSNotFound as NSUInteger {
            break;
        }
        index = previous;
    }
}

@end

// Special variant for use by CFArray with NULL callbacks: objects aren't
// necessarily Objective-C objects and won't be retained/released.
@implementation _touchHLE_NSMutableArray_non_retaining: _touchHLE_NSMutableArray

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

- (())addObject:(id)object {
    assert_can_grow(env, this);
    env.objc.borrow_mut::<ArrayHostObject>(this).array.push(object);
}

- (())removeObjectAtIndex:(NSUInteger)index {
    env.objc.borrow_mut::<ArrayHostObject>(this).array.remove(index as usize);
}

- (())replaceObjectAtIndex:(NSUInteger)index withObject:(id)obj {
    env.objc.borrow_mut::<ArrayHostObject>(this).array[index as usize] = obj;
}

- (())removeLastObject {
    env.objc.borrow_mut::<ArrayHostObject>(this).array.pop().unwrap();
}

@end

};

fn assert_can_grow(env: &mut Environment, array: id) {
    let host = env.objc.borrow::<ArrayHostObject>(array);
    if let Some(capacity) = host.fixed_capacity {
        assert!(
            host.array.len() < capacity,
            "fixed-capacity CFMutableArray cannot exceed capacity {capacity}"
        );
    }
}

/// Apply Core Foundation's fixed-capacity contract to an empty mutable array.
/// A zero capacity means variable-size and therefore leaves the limit unset.
pub(crate) fn set_fixed_capacity(env: &mut Environment, array: id, capacity: usize) {
    let host = env.objc.borrow_mut::<ArrayHostObject>(array);
    assert!(host.array.is_empty());
    if capacity == 0 {
        host.fixed_capacity = None;
    } else {
        host.array.reserve(capacity);
        host.fixed_capacity = Some(capacity);
    }
}

/// Shortcut for host code, roughly equivalent to
/// `[[NSArray alloc] initWithObjects:count]` but without copying.
/// The elements should already be "retained by" the `Vec`.
pub fn from_vec(env: &mut Environment, objects: Vec<id>) -> id {
    let array: id = msg_class![env; NSArray alloc];
    env.objc.borrow_mut::<ArrayHostObject>(array).array = objects;
    array
}

/// Shortcut for host code, roughly equivalent to
/// `[[NSMutableArray alloc] initWithObjects:count]` but without copying.
/// The elements should already be "retained by" the `Vec`.
pub fn mutable_from_vec(env: &mut Environment, objects: Vec<id>) -> id {
    let array: id = msg_class![env; NSMutableArray alloc];
    env.objc.borrow_mut::<ArrayHostObject>(array).array = objects;
    array
}

/// A helper to build a description NSString
/// for a NSArray or a NSMutableArray.
fn build_description(env: &mut Environment, arr: id) -> id {
    // According to docs, this description should be formatted as property list.
    // But by the same docs, it's meant to be used for debugging purposes only.
    let desc: id = msg_class![env; NSMutableString new];
    let prefix: id = ns_string::from_rust_string(env, "(\n".to_string());
    () = msg![env; desc appendString:prefix];
    release(env, prefix);
    let values: Vec<id> = env.objc.borrow_mut::<ArrayHostObject>(arr).array.clone();
    for value in values {
        let value_desc: id = msg![env; value description];
        // TODO: respect nesting and padding
        let format = format!("\t{},\n", ns_string::to_rust_string(env, value_desc));
        let format = ns_string::from_rust_string(env, format);
        () = msg![env; desc appendString:format];
        release(env, format);
    }
    let suffix: id = ns_string::from_rust_string(env, ")".to_string());
    () = msg![env; desc appendString:suffix];
    release(env, suffix);
    let desc_imm = msg![env; desc copy];
    release(env, desc);
    autorelease(env, desc_imm)
}

/// A shared objectEnumerator helper method.
fn object_enumerator_inner(env: &mut Environment, arr: id) -> id {
    let array_host_object: &mut ArrayHostObject = env.objc.borrow_mut(arr);
    let vec = array_host_object.array.to_vec();
    object_enumerator_inner_helper(env, arr, vec)
}

/// A shared reverseObjectEnumerator helper method.
fn reverse_object_enumerator_inner(env: &mut Environment, arr: id) -> id {
    let array_host_object: &mut ArrayHostObject = env.objc.borrow_mut(arr);
    // TODO: avoid copying?
    let vec = array_host_object
        .array
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<_>>();
    object_enumerator_inner_helper(env, arr, vec)
}

fn object_enumerator_inner_helper(env: &mut Environment, arr: id, vec: Vec<id>) -> id {
    let host_object = Box::new(ObjectEnumeratorHostObject {
        array: arr,
        iterator: vec.into_iter(),
    });
    retain(env, arr);
    let class = env
        .objc
        .get_known_class("_touchHLE_NSArray_ObjectEnumerator", &mut env.mem);
    let enumerator = env.objc.alloc_object(class, host_object, &mut env.mem);
    autorelease(env, enumerator)
}

fn mutable_copy_inner(env: &mut Environment, arr: id) -> id {
    let mut_arr: id = msg_class![env; NSMutableArray alloc];
    let array = env.objc.borrow::<ArrayHostObject>(arr).array.clone();
    for &object in &array {
        retain(env, object);
    }
    env.objc.borrow_mut::<ArrayHostObject>(mut_arr).array = array;
    mut_arr
}

fn init_with_coder_inner(env: &mut Environment, arr: id, coder: id) -> id {
    let class: Class = msg![env; coder class];
    let keyed_unarch_class: Class = msg_class![env; NSKeyedUnarchiver class];
    let nib_archive_class: Class = msg_class![env; _touchHLE_NIBArchiveDecoder class];
    // It seems that every NSArray item in an NSKeyedArchiver plist looks like:
    // {
    //   "$class" => (uid of NSArray class goes here),
    //   "NS.objects" => [
    //     // objects here
    //   ]
    // }
    // Presumably we need to call a `decodeFooBarForKey:` method on the NSCoder
    // here, passing in an NSString for "NS.objects". There is no method for
    // arrays though (maybe it's `decodeObjectForKey:`), and in any case
    // allocating an NSString here would be inconvenient, so let's just take a
    // shortcut.
    let objects = if env.objc.class_is_subclass_of(class, keyed_unarch_class) {
        ns_keyed_unarchiver::decode_current_array(env, coder)
    } else if env.objc.class_is_subclass_of(class, nib_archive_class) {
        _nib_archive_decoder::decode_current_array(env, coder)
    } else {
        unimplemented!()
    };

    let host_object: &mut ArrayHostObject = env.objc.borrow_mut(arr);
    assert!(host_object.array.is_empty());
    host_object.array = objects; // objects are already retained
    arr
}

fn encode_with_coder_inner(env: &mut Environment, arr: id, coder: id) {
    let host_obj: ArrayHostObject = std::mem::take(env.objc.borrow_mut(arr));
    let mut encoded_vals = vec![];
    for v in &host_obj.array {
        // TODO: support other type of coders, not only NSKeyedArchiver
        let vv = encode_object(env, coder, *v);
        encoded_vals.push(plist::Value::Uid(vv));
    }
    *env.objc.borrow_mut(arr) = host_obj;

    let scope = get_value_to_encode_for_current_key(env, coder);
    scope.insert("NS.objects".to_string(), plist::Value::Array(encoded_vals));
}

/// Helper for initWithObjects.
fn init_with_objects_inner(env: &mut Environment, arr: id, first_obj: id, args: DotDotDot) {
    retain(env, first_obj);
    let mut objects = vec![first_obj];
    let mut varargs = args.start();
    loop {
        let next_arg: id = varargs.next(env);
        if next_arg.is_null() {
            break;
        }
        retain(env, next_arg);
        objects.push(next_arg);
    }
    env.objc.borrow_mut::<ArrayHostObject>(arr).array = objects;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_factories_preserve_mutability_and_element_ownership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let first: id = msg_class![env; NSObject new];
        let second: id = msg_class![env; NSObject new];
        let mutable: id = msg_class![env; NSMutableArray arrayWithObject:first];
        let immutable: id = msg_class![env; NSArray arrayWithObject:first];
        let mutable_class: Class = msg_class![env; NSMutableArray class];
        let is_mutable: bool = msg![env; mutable isKindOfClass:mutable_class];
        let is_immutable_mutable: bool = msg![env; immutable isKindOfClass:mutable_class];
        assert!(is_mutable);
        assert!(!is_immutable_mutable);
        // The GDataXML children path appends a second node to a singleton.
        () = msg![env; mutable addObject:second];
        let count: NSUInteger = msg![env; mutable count];
        assert_eq!(count, 2);
        let count: NSUInteger = msg![env; immutable count];
        assert_eq!(count, 1);
        let actual: id = msg![env; mutable objectAtIndex:0u32];
        assert_eq!(actual, first);
        let actual: id = msg![env; mutable objectAtIndex:1u32];
        assert_eq!(actual, second);
        assert_eq!(env.objc.get_refcount(first).get(), 3);
        assert_eq!(env.objc.get_refcount(second).get(), 2);
        retain(env, mutable);
        release(env, pool);
        assert_eq!(env.objc.get_refcount(first).get(), 2);
        () = msg![env; mutable removeAllObjects];
        assert_eq!(env.objc.get_refcount(first).get(), 1);
        assert_eq!(env.objc.get_refcount(second).get(), 1);
        release(env, mutable);
        release(env, first);
        release(env, second);
    }

    #[test]
    fn mutable_array_accepts_pointer_count_initializer() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let first: id = msg_class![env; NSObject new];
        let second: id = msg_class![env; NSObject new];
        let objects: crate::mem::MutPtr<id> = env.mem.alloc(8).cast();
        env.mem.write(objects, first);
        env.mem.write(objects + 1, second);
        let object_list = objects.cast_const();
        let array: id = msg_class![env; NSMutableArray alloc];
        let array: id = msg![env; array initWithObjects:object_list count:2u32];
        let count: NSUInteger = msg![env; array count];
        assert_eq!(count, 2);
        let actual: id = msg![env; array objectAtIndex:1u32];
        assert_eq!(actual, second);
        release(env, array);
        release(env, second);
        release(env, first);
        env.mem.free(objects.cast());
    }

    #[test]
    fn dev_243_array_by_adding_objects_preserves_order_ownership_and_inputs() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let first: id = msg_class![env; NSObject new];
        let second: id = msg_class![env; NSObject new];
        let left: id = msg_class![env; NSArray arrayWithObject:first];
        let right: id = msg_class![env; NSArray arrayWithObject:second];

        let combined: id = msg![env; left arrayByAddingObjectsFromArray:right];
        let combined_count: NSUInteger = msg![env; combined count];
        let left_count: NSUInteger = msg![env; left count];
        let right_count: NSUInteger = msg![env; right count];
        let combined_first: id = msg![env; combined objectAtIndex:0u32];
        let combined_second: id = msg![env; combined objectAtIndex:1u32];
        assert_eq!(combined_count, 2);
        assert_eq!(left_count, 1);
        assert_eq!(right_count, 1);
        assert_eq!(combined_first, first);
        assert_eq!(combined_second, second);
        assert_eq!(env.objc.get_refcount(first).get(), 3);
        assert_eq!(env.objc.get_refcount(second).get(), 3);

        release(env, pool);
        assert_eq!(env.objc.get_refcount(first).get(), 1);
        assert_eq!(env.objc.get_refcount(second).get(), 1);
        release(env, first);
        release(env, second);
    }

    #[test]
    fn remove_objects_at_indexes_uses_original_indexes_and_ownership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let objects: Vec<id> = (0..6).map(|_| msg_class![env; NSObject new]).collect();
        let owned_objects = objects.iter().map(|&object| retain(env, object)).collect();
        let array = mutable_from_vec(env, owned_objects);
        let indexes: id = msg_class![env; NSMutableIndexSet indexSet];
        () = msg![env; indexes addIndex:1u32];
        () = msg![env; indexes addIndex:3u32];
        () = msg![env; indexes addIndex:4u32];

        () = msg![env; array removeObjectsAtIndexes:indexes];
        let count: NSUInteger = msg![env; array count];
        assert_eq!(count, 3);
        let first: id = msg![env; array objectAtIndex:0u32];
        let second: id = msg![env; array objectAtIndex:1u32];
        let third: id = msg![env; array objectAtIndex:2u32];
        assert_eq!(first, objects[0]);
        assert_eq!(second, objects[2]);
        assert_eq!(third, objects[5]);
        for (index, &object) in objects.iter().enumerate() {
            let expected = if [1, 3, 4].contains(&index) { 1 } else { 2 };
            assert_eq!(env.objc.get_refcount(object).get(), expected);
        }

        release(env, array);
        for object in objects {
            release(env, object);
        }
        release(env, pool);
    }

    #[test]
    fn remove_object_removes_all_equal_entries_without_index_drift() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let object: id = msg_class![env; NSObject new];
        let owned = vec![
            retain(env, object),
            retain(env, object),
            retain(env, object),
        ];
        let array = mutable_from_vec(env, owned);

        () = msg![env; array removeObject:object];

        let count: NSUInteger = msg![env; array count];
        assert_eq!(count, 0);
        assert_eq!(env.objc.get_refcount(object).get(), 1);
        release(env, array);
        release(env, object);
    }

    #[test]
    fn dev_244_remove_object_identical_uses_identity_not_equality() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let first: id = msg_class![env; NSNumber numberWithInt:1000i32];
        let second: id = msg_class![env; NSNumber numberWithInt:1000i32];
        assert_ne!(first, second);
        let equal: bool = msg![env; first isEqual:second];
        assert!(equal);
        let owned = vec![retain(env, first), retain(env, second), retain(env, first)];
        let array = mutable_from_vec(env, owned);

        () = msg![env; array removeObjectIdenticalTo:first];

        let count: NSUInteger = msg![env; array count];
        let remaining: id = msg![env; array objectAtIndex:0u32];
        assert_eq!(count, 1);
        assert_eq!(remaining, second);
        release(env, array);
        release(env, pool);
    }

    #[test]
    #[should_panic(expected = "arrayWithObject: requires a non-nil object")]
    fn immutable_singleton_rejects_nil() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let _: id = msg_class![env; NSArray arrayWithObject:nil];
    }

    #[test]
    #[should_panic(expected = "arrayWithObject: requires a non-nil object")]
    fn mutable_singleton_rejects_nil() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let _: id = msg_class![env; NSMutableArray arrayWithObject:nil];
    }
}
