/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSValue` class cluster, including `NSNumber`.

use super::ns_string::{from_rust_ordering, from_rust_string, to_rust_string};
use super::{
    _nib_archive_decoder, ns_keyed_unarchiver, NSComparisonResult, NSOrderedSame, NSRange,
    NSUInteger,
};
use crate::frameworks::core_animation::ca_transform_3d::CATransform3D;
use crate::frameworks::core_foundation::cf_number::{
    kCFNumberCFIndexType, kCFNumberCGFloatType, kCFNumberCharType, kCFNumberDoubleType,
    kCFNumberFloat32Type, kCFNumberFloat64Type, kCFNumberFloatType, kCFNumberIntType,
    kCFNumberLongLongType, kCFNumberLongType, kCFNumberNSIntegerType, kCFNumberSInt16Type,
    kCFNumberSInt32Type, kCFNumberSInt64Type, kCFNumberSInt8Type, kCFNumberShortType, CFNumberType,
};
use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_keyed_archiver::get_value_to_encode_for_current_key;
use crate::frameworks::foundation::NSInteger;
use crate::mem::{ConstPtr, ConstVoidPtr, MutVoidPtr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, Class, ClassExports,
    HostObject, NSZonePtr,
};
use crate::Environment;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    /// Permanently allocated, null-terminated Objective-C type encodings.
    objc_type_encodings: HashMap<u8, ConstPtr<u8>>,
    /// Apple's Core Foundation keeps canonical integer objects for -1...12.
    /// NSNumber and CFNumber are toll-free bridged, so Foundation factory and
    /// initializer paths must share the same process-lifetime objects.
    small_integer_cache: HashMap<i32, id>,
    /// Stable Foundation boolean singletons. Keep these separate from the
    /// integer cache so boolean type identity (`objCType`, coding) is retained.
    boolean_cache: [Option<id>; 2],
}

#[derive(Debug)]
pub(super) enum NSValueHostObject {
    CGPoint(CGPoint),
    CGSize(CGSize),
    CGRect(CGRect),
    CATransform3D(CATransform3D),
    NSRange(NSRange),
}
impl HostObject for NSValueHostObject {}

macro_rules! impl_AsValue {
    ($method_name:tt, $typ:tt) => {
        pub fn $method_name(&self) -> $typ {
            match self {
                // Cast to u8 is needed for float conversions
                NSNumberHostObject::Bool(x) => *x as u8 as _,
                NSNumberHostObject::UnsignedLongLong(x) => *x as _,
                NSNumberHostObject::UnsignedInt(x) => *x as _,
                NSNumberHostObject::Int(x) => *x as _,
                NSNumberHostObject::LongLong(x) => *x as _,
                NSNumberHostObject::Float(x) => *x as _,
                NSNumberHostObject::Double(x) => *x as _,
                NSNumberHostObject::Short(x) => *x as _,
                NSNumberHostObject::UnsignedShort(x) => *x as _,
                NSNumberHostObject::Char(x) => *x as _,
            }
        }
    };
}

#[derive(Debug)]
pub(super) enum NSNumberHostObject {
    Bool(bool),
    UnsignedLongLong(u64),
    UnsignedInt(u32),
    Int(i32), // Also covers Integer and Long since this is a 32-bit platform.
    LongLong(i64),
    Float(f32),
    Double(f64),
    Short(i16),
    UnsignedShort(u16),
    Char(i8),
}
impl HostObject for NSNumberHostObject {}

impl NSNumberHostObject {
    fn as_bool(&self) -> bool {
        match self {
            NSNumberHostObject::Bool(x) => *x,
            NSNumberHostObject::UnsignedLongLong(x) => *x != 0,
            NSNumberHostObject::UnsignedInt(x) => *x != 0,
            NSNumberHostObject::Int(x) => *x != 0,
            NSNumberHostObject::LongLong(x) => *x != 0,
            NSNumberHostObject::Float(x) => *x != 0.0,
            NSNumberHostObject::Double(x) => *x != 0.0,
            NSNumberHostObject::Short(x) => *x != 0,
            NSNumberHostObject::UnsignedShort(x) => *x != 0,
            NSNumberHostObject::Char(x) => *x != 0,
        }
    }
    fn is_float(&self) -> bool {
        matches!(
            self,
            NSNumberHostObject::Float(_) | NSNumberHostObject::Double(_)
        )
    }
    impl_AsValue!(as_int, i32);
    impl_AsValue!(as_long_long, i64);
    impl_AsValue!(as_unsigned_long_long, u64);
    impl_AsValue!(as_unsigned_int, u32);
    impl_AsValue!(as_float, f32);
    impl_AsValue!(as_double, f64);
    impl_AsValue!(as_short, i16);
    impl_AsValue!(as_unsigned_short, u16);
    impl_AsValue!(as_char, i8);
    impl_AsValue!(as_i128, i128);
}

const MIN_CACHED_INTEGER: i32 = -1;
const MAX_CACHED_INTEGER: i32 = 12;

/// Return Apple's process-lifetime cached integer object when applicable.
///
/// CFNumber canonicalizes cached integers to its signed 32-bit storage class,
/// regardless of the integer initializer that first requested the value.  Do
/// the same here.  The exact range comes from Apple's open-source CFNumber
/// implementation and is observable through object lifetime and identity.
fn cached_small_integer(env: &mut Environment, class: Class, value: i64) -> Option<id> {
    let value: i32 = value.try_into().ok()?;
    if !(MIN_CACHED_INTEGER..=MAX_CACHED_INTEGER).contains(&value) {
        return None;
    }

    let ns_number = env.objc.get_known_class("NSNumber", &mut env.mem);
    if class != ns_number {
        // Preserve subclass construction semantics rather than returning a
        // base-class singleton from an inherited initializer.
        return None;
    }

    if let Some(&number) = env
        .framework_state
        .foundation
        .ns_value
        .small_integer_cache
        .get(&value)
    {
        return Some(number);
    }

    let number = env.objc.alloc_static_object(
        ns_number,
        Box::new(NSNumberHostObject::Int(value)),
        &mut env.mem,
    );
    env.framework_state
        .foundation
        .ns_value
        .small_integer_cache
        .insert(value, number);
    Some(number)
}

fn replace_with_cached_small_integer(env: &mut Environment, this: id, value: i64) -> Option<id> {
    let class: Class = msg![env; this class];
    let cached = cached_small_integer(env, class, value)?;
    release(env, this);
    Some(cached)
}

fn replace_with_cached_boolean(env: &mut Environment, this: id, value: bool) -> Option<id> {
    let class: Class = msg![env; this class];
    let ns_number = env.objc.get_known_class("NSNumber", &mut env.mem);
    if class != ns_number {
        return None;
    }
    let index = usize::from(value);
    let cached = if let Some(number) = env
        .framework_state
        .foundation
        .ns_value
        .boolean_cache[index]
    {
        number
    } else {
        let number = env.objc.alloc_static_object(
            ns_number,
            Box::new(NSNumberHostObject::Bool(value)),
            &mut env.mem,
        );
        env.framework_state
            .foundation
            .ns_value
            .boolean_cache[index] = Some(number);
        number
    };
    release(env, this);
    Some(cached)
}

/// Return the canonical Core Foundation storage type for this NSNumber.
///
/// CFNumber does not preserve all of the type aliases supplied at creation
/// time. In particular, its public contract reports the internal storage
/// category rather than necessarily echoing the caller's original type.
pub(crate) fn cf_number_type(env: &mut Environment, number: id) -> CFNumberType {
    match env.objc.borrow::<NSNumberHostObject>(number) {
        NSNumberHostObject::Bool(_) | NSNumberHostObject::Char(_) => kCFNumberSInt8Type,
        NSNumberHostObject::Short(_) => kCFNumberSInt16Type,
        NSNumberHostObject::UnsignedShort(_) | NSNumberHostObject::Int(_) => kCFNumberSInt32Type,
        NSNumberHostObject::UnsignedInt(_)
        | NSNumberHostObject::LongLong(_)
        | NSNumberHostObject::UnsignedLongLong(_) => kCFNumberSInt64Type,
        NSNumberHostObject::Float(_) => kCFNumberFloat32Type,
        NSNumberHostObject::Double(_) => kCFNumberFloat64Type,
    }
}

fn objc_type(env: &mut Environment, number: id) -> ConstPtr<u8> {
    let encoding = match env.objc.borrow::<NSNumberHostObject>(number) {
        NSNumberHostObject::Bool(_) | NSNumberHostObject::Char(_) => b'c',
        NSNumberHostObject::UnsignedLongLong(_) => b'Q',
        NSNumberHostObject::UnsignedInt(_) => b'I',
        NSNumberHostObject::Int(_) => b'i',
        NSNumberHostObject::LongLong(_) => b'q',
        NSNumberHostObject::Float(_) => b'f',
        NSNumberHostObject::Double(_) => b'd',
        NSNumberHostObject::Short(_) => b's',
        NSNumberHostObject::UnsignedShort(_) => b'S',
    };

    if let Some(&ptr) = env
        .framework_state
        .foundation
        .ns_value
        .objc_type_encodings
        .get(&encoding)
    {
        return ptr;
    }

    // Apple's API returns a pointer that remains valid for the lifetime of the
    // NSNumber. Keep one process-lifetime C string for each type encoding.
    let ptr = env.mem.alloc_and_write_cstr(&[encoding]).cast_const();
    env.framework_state
        .foundation
        .ns_value
        .objc_type_encodings
        .insert(encoding, ptr);
    ptr
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSValue is an abstract class. None of the things it should provide are
// implemented here yet (TODO).
@implementation NSValue: NSObject

+ (id)valueWithPointer:(ConstVoidPtr)ptr {
    // TODO: implement with `value:withObjCType:` instead
    msg_class![env; NSNumber numberWithUnsignedInt:(ptr.to_bits())]
}

+ (id)valueWithCGPoint:(CGPoint)value {
    let host_object = Box::new(NSValueHostObject::CGPoint(value));
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, new)
}

+ (id)valueWithCGSize:(CGSize)value {
    let host_object = Box::new(NSValueHostObject::CGSize(value));
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, new)
}

+ (id)valueWithCGRect:(CGRect)value {
    let host_object = Box::new(NSValueHostObject::CGRect(value));
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, new)
}

+ (id)valueWithCATransform3D:(CATransform3D)value {
    let host_object = Box::new(NSValueHostObject::CATransform3D(value));
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, new)
}

+ (id)valueWithRange:(NSRange)value {
    let host_object = Box::new(NSValueHostObject::NSRange(value));
    let new = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, new)
}

- (CGPoint)CGPointValue {
    let host_object = env.objc.borrow::<NSValueHostObject>(this);
    match host_object {
        NSValueHostObject::CGPoint(cg_point) => *cg_point,
        _ => unimplemented!()
    }
}

- (CGSize)CGSizeValue {
    let host_object = env.objc.borrow::<NSValueHostObject>(this);
    match host_object {
        NSValueHostObject::CGSize(cg_size) => *cg_size,
        _ => unimplemented!()
    }
}

- (CGRect)CGRectValue {
    let host_object = env.objc.borrow::<NSValueHostObject>(this);
    match host_object {
        NSValueHostObject::CGRect(cg_rect) => *cg_rect,
        _ => unimplemented!()
    }
}

- (CATransform3D)CATransform3DValue {
    let host_object = env.objc.borrow::<NSValueHostObject>(this);
    match host_object {
        NSValueHostObject::CATransform3D(transform) => *transform,
        _ => unimplemented!()
    }
}

- (NSRange)rangeValue {
    let host_object = env.objc.borrow::<NSValueHostObject>(this);
    match host_object {
        NSValueHostObject::NSRange(range) => NSRange {
            location: range.location,
            length: range.length,
        },
        _ => unimplemented!()
    }
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (MutVoidPtr)pointerValue {
    let class: Class = msg![env; this class];
    assert!(class == env.objc.get_known_class("NSNumber", &mut env.mem));
    // According to the docs, `If the value object was not created to hold
    // a pointer-sized data item, the result is undefined.`
    let val = msg![env; this unsignedIntValue];
    MutVoidPtr::from_bits(val)
}

@end

// NSNumber is not an abstract class.
@implementation NSNumber: NSValue

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberHostObject::Bool(false));
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)numberWithBool:(bool)value {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBool:value];
    autorelease(env, new)
}

+ (id)numberWithFloat:(f32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithFloat:value];
    autorelease(env, new)
}

+ (id)numberWithDouble:(f64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithDouble:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedInt:(u32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedInt:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedLong:(u32)value {
    // `unsigned long` is 32 bits in the guest iPhone ABI.
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedLong:value];
    autorelease(env, new)
}

+ (id)numberWithInt:(i32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInt:value];
    autorelease(env, new)
}

+ (id)numberWithLong:(i32)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithLong:value];
    autorelease(env, new)
}

+ (id)numberWithInteger:(NSInteger)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInteger:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedInteger:(NSUInteger)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedInteger:value];
    autorelease(env, new)
}

+ (id)numberWithLongLong:(i64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithLongLong:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedLongLong:(u64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedLongLong:value];
    autorelease(env, new)
}

+ (id)numberWithShort:(i16)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithShort:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedShort:(u16)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedShort:value];
    autorelease(env, new)
}

+ (id)numberWithChar:(i8)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithChar:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedChar:(u8)value {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedChar:value];
    autorelease(env, new)
}

// TODO: types other than booleans and long longs

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    let class: Class = msg![env; coder class];
    let keyed_unarch_class: Class = msg_class![env; NSKeyedUnarchiver class];
    let nib_archive_class: Class = msg_class![env; _touchHLE_NIBArchiveDecoder class];
    let new_num = if env.objc.class_is_subclass_of(class, keyed_unarch_class) {
        ns_keyed_unarchiver::decode_current_number(env, coder)
    } else if env.objc.class_is_subclass_of(class, nib_archive_class) {
        _nib_archive_decoder::decode_current_number(env, coder)
    } else {
        unimplemented!();
    };
    release(env, this);
    new_num
}
- (())encodeWithCoder:(id)coder {
    let host_object = env.objc.borrow::<NSNumberHostObject>(this);
    let (key, val) = match host_object {
        NSNumberHostObject::Bool(b) => ("NS.boolval", plist::Value::Boolean(*b)),
        NSNumberHostObject::UnsignedLongLong(i) => {
            ("NS.intval", plist::Value::Integer((*i).into()))
        }
        NSNumberHostObject::UnsignedInt(i) => {
            ("NS.intval", plist::Value::Integer((*i).into()))
        }
        NSNumberHostObject::Int(i) => ("NS.intval", plist::Value::Integer((*i).into())),
        NSNumberHostObject::LongLong(i) => ("NS.intval", plist::Value::Integer((*i).into())),
        NSNumberHostObject::Float(f) => ("NS.dblval", plist::Value::Real((*f).into())),
        NSNumberHostObject::Double(d) => ("NS.dblval", plist::Value::Real(*d)),
        NSNumberHostObject::Short(i) => ("NS.intval", plist::Value::Integer((*i).into())),
        NSNumberHostObject::UnsignedShort(i) => {
            ("NS.intval", plist::Value::Integer((*i).into()))
        }
        NSNumberHostObject::Char(i) => ("NS.intval", plist::Value::Integer((*i).into())),
    };

    let scope = get_value_to_encode_for_current_key(env, coder);
    scope.insert(key.to_string(), val);
}

- (id)initWithBool:(bool)value {
    // Historical Foundation uses stable cached boolean number objects. This
    // also protects old applications that stored a convenience boolean
    // without retaining it across an autorelease-pool drain.
    if let Some(cached) = replace_with_cached_boolean(env, this, value) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Bool(value);
    this
}

- (id)initWithFloat:(f32)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Float(value);
    this
}

- (id)initWithDouble:(f64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Double(value);
    this
}

- (id)initWithLongLong:(i64)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::LongLong(value);
    this
}

- (id)initWithUnsignedInt:(u32)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedInt(value);
    this
}

- (id)initWithUnsignedLong:(u32)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedInt(value);
    this
}

- (id)initWithInt:(i32)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithLong:(i32)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithInteger:(NSInteger)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithUnsignedInteger:(NSUInteger)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedInt(value);
    this
}

- (id)initWithUnsignedLongLong:(u64)value {
    if value <= i64::MAX as u64 {
        if let Some(cached) = replace_with_cached_small_integer(env, this, value as i64) {
            return cached;
        }
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedLongLong(value);
    this
}

- (id)initWithShort:(i16)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Short(value);
    this
}

- (id)initWithUnsignedShort:(u16)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedShort(value);
    this
}

- (id)initWithChar:(i8)value {
    if let Some(cached) = replace_with_cached_small_integer(env, this, value.into()) {
        return cached;
    }
    *env.objc.borrow_mut(this) = NSNumberHostObject::Char(value);
    this
}

- (id)initWithUnsignedChar:(u8)value {
    // NSNumber is a class cluster: reuse unsigned storage without sign
    // extension.
    // As with the other integer initializers, objCType describes storage.
    msg![env; this initWithUnsignedShort:(u16::from(value))]
}

- (u8)unsignedCharValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_int() as u8
}

- (bool)boolValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_bool()
}

- (NSInteger)integerValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_int()
}

- (i32)intValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_int()
}

- (i32)longValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_int()
}

- (f32)floatValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_float()
}

- (f64)doubleValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_double()
}

- (i64)longLongValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_long_long()
}

- (u64)unsignedLongLongValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_long_long()
}

- (u32)unsignedIntValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_int()
}

- (u32)unsignedLongValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_int()
}

- (NSUInteger)unsignedIntegerValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_int()
}

- (i16)shortValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_short()
}

- (u16)unsignedShortValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_unsigned_short()
}

- (i8)charValue {
    env.objc.borrow::<NSNumberHostObject>(this).as_char()
}

- (ConstPtr<u8>)objCType {
    objc_type(env, this)
}

- (id)description {
    msg![env; this stringValue]
}

- (id)stringValue {
    msg![env; this descriptionWithLocale:nil]
}
- (id)descriptionWithLocale:(id)locale {
    assert_eq!(locale, nil); // TODO
    // TODO: do not alloc format strings each time
    let format = match env.objc.borrow(this) {
        NSNumberHostObject::Bool(_) | NSNumberHostObject::Char(_) | NSNumberHostObject::Int(_) => from_rust_string(env, "%i".to_string()),
        NSNumberHostObject::Double(_) => from_rust_string(env, "%0.16g".to_string()),
        NSNumberHostObject::Float(_) => from_rust_string(env, "%0.7g".to_string()),
        NSNumberHostObject::LongLong(_) => from_rust_string(env, "%lli".to_string()),
        NSNumberHostObject::Short(_) => from_rust_string(env, "%hi".to_string()),
        NSNumberHostObject::UnsignedInt(_) => from_rust_string(env, "%u".to_string()),
        NSNumberHostObject::UnsignedLongLong(_) => from_rust_string(env, "%llu".to_string()),
        NSNumberHostObject::UnsignedShort(_) => from_rust_string(env, "%hu".to_string()),
    };
    let ns_string_class = env.objc.get_known_class("NSString", &mut env.mem);
    let sel = env.objc.lookup_selector("stringWithFormat:").unwrap();
    // TODO: type info for host-to-host message calls with var-args
    let res = match env.objc.borrow(this) {
        NSNumberHostObject::Bool(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value as i32)),
        NSNumberHostObject::Char(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::Double(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::Float(value) => {
            // Need to promote float to double for the expected argument of %g
            crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value as f64))
        },
        NSNumberHostObject::Int(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::LongLong(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::Short(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::UnsignedInt(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::UnsignedLongLong(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
        NSNumberHostObject::UnsignedShort(value) => crate::objc::msg_send_no_type_checking(env, (ns_string_class, sel, format, *value)),
    };
    release(env, format);
    res
}

- (NSUInteger)hash {
    // The only requirement for [obj hash] is that values that compare equal
    // (via [obj isEqual] have the same hash. Hashing the underlying
    // bits works here.
    let value =
    match env.objc.borrow(this) {
        NSNumberHostObject::Bool(value) => *value as u64,
        NSNumberHostObject::UnsignedLongLong(value) => *value,
        NSNumberHostObject::UnsignedInt(value) => *value as u64,
        NSNumberHostObject::Int(value) => *value as u64,
        NSNumberHostObject::LongLong(value) => *value as u64,
        NSNumberHostObject::Float(value) => value.to_bits() as u64,
        NSNumberHostObject::Double(value) => value.to_bits(),
        NSNumberHostObject::Short(value) => *value as u64,
        NSNumberHostObject::UnsignedShort(value) => *value as u64,
        NSNumberHostObject::Char(value) => *value as u64,
    };
    super::hash_helper(&value)
}

- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSNumber class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
    msg![env; this isEqualToNumber:other]
}

- (bool)isEqualToNumber:(id)other {
    let res: NSComparisonResult = msg![env; this compare:other];
    res == NSOrderedSame
}

- (NSComparisonResult)compare:(id)other { // NSNumber *
    let num = env.objc.borrow::<NSNumberHostObject>(this);
    let other_num = env.objc.borrow::<NSNumberHostObject>(other);
    let ordering = match (num.is_float(), other_num.is_float()) {
        (false, false) => num.as_i128().cmp(&other_num.as_i128()),
        // In case of having a float, we promote to double for comparison
        _ => {
            // TODO: handle partial cmp fails
            let res = num.as_double().partial_cmp(&other_num.as_double()).unwrap();
            if res == Ordering::Equal {
                // On ties, we compare as i128 as well
                num.as_i128().cmp(&other_num.as_i128())
            } else {
                res
            }
        },
    };
    from_rust_ordering(ordering)
}

// TODO: accessors etc

@end

@implementation NSDecimalNumber: NSNumber

+ (id)decimalNumberWithString:(id)string {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithString:string];
    autorelease(env, new)
}

- (id)initWithString:(id)string {
    let value = to_rust_string(env, string)
        .trim()
        .parse::<f64>()
        .unwrap_or(f64::NAN);
    msg![env; this initWithDouble:value]
}

@end

};

pub fn is_conversion_lossless(env: &mut Environment, this: id, type_: CFNumberType) -> bool {
    let num = env.objc.borrow::<NSNumberHostObject>(this);
    let num2: id = match type_ {
        kCFNumberSInt32Type | kCFNumberIntType => {
            let val: i32 = num.as_int();
            msg_class![env; NSNumber numberWithInt:val]
        }
        kCFNumberFloat32Type | kCFNumberFloatType => {
            let val: f32 = num.as_float();
            msg_class![env; NSNumber numberWithFloat:val]
        }
        kCFNumberSInt16Type | kCFNumberShortType => {
            let val: i16 = num.as_short();
            msg_class![env; NSNumber numberWithShort:val]
        }
        kCFNumberSInt8Type | kCFNumberCharType => {
            let val: i8 = num.as_char();
            msg_class![env; NSNumber numberWithChar:val]
        }
        kCFNumberSInt64Type | kCFNumberLongLongType => {
            let val: i64 = num.as_long_long();
            msg_class![env; NSNumber numberWithLongLong:val]
        }
        kCFNumberFloat64Type | kCFNumberDoubleType => {
            let val: f64 = num.as_double();
            msg_class![env; NSNumber numberWithDouble:val]
        }
        kCFNumberLongType | kCFNumberCFIndexType | kCFNumberNSIntegerType => {
            let val: i32 = num.as_int();
            msg_class![env; NSNumber numberWithLong:val]
        }
        kCFNumberCGFloatType => {
            let val: f32 = num.as_float();
            msg_class![env; NSNumber numberWithFloat:val]
        }
        _ => unimplemented!("is_conversion_lossless for {}", type_),
    };
    msg![env; this isEqualToNumber:num2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_long_uses_the_full_32_bit_guest_range() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let maximum = u32::MAX;
        let number: id = msg_class![env; NSNumber numberWithUnsignedLong:maximum];
        let value: u32 = msg![env; number unsignedLongValue];
        let unsigned_int: u32 = msg![env; number unsignedIntValue];
        assert_eq!(value, u32::MAX);
        assert_eq!(unsigned_int, u32::MAX);
        let string: id = msg![env; number stringValue];
        assert_eq!(to_rust_string(env, string), "4294967295");
    }

    #[test]
    fn small_integer_cache_matches_core_foundation_range_and_lifetime() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;

        for value in MIN_CACHED_INTEGER..=MAX_CACHED_INTEGER {
            let first: id = msg_class![env; NSNumber numberWithInt:value];
            let second: id = msg_class![env; NSNumber numberWithLongLong:(value as i64)];
            assert_eq!(first, second);
            assert!(env.objc.has_static_lifetime(first));
            let round_trip: i32 = msg![env; first intValue];
            assert_eq!(round_trip, value);
        }

        let below: id = msg_class![env; NSNumber numberWithInt:(MIN_CACHED_INTEGER - 1)];
        let above: id = msg_class![env; NSNumber numberWithInt:(MAX_CACHED_INTEGER + 1)];
        assert!(!env.objc.has_static_lifetime(below));
        assert!(!env.objc.has_static_lifetime(above));
    }

    #[test]
    fn boolean_factories_return_stable_boolean_typed_objects() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        for (value, integer) in [(false, 0), (true, 1)] {
            let pool: id = msg_class![env; NSAutoreleasePool new];
            let boolean: id = msg_class![env; NSNumber numberWithBool:value];
            release(env, pool);
            let repeated: id = msg_class![env; NSNumber numberWithBool:value];
            let numeric: id = msg_class![env; NSNumber numberWithInt:integer];
            assert_eq!(boolean, repeated);
            assert_ne!(boolean, numeric);
            assert!(env.objc.has_static_lifetime(boolean));
            assert!(matches!(
                env.objc.borrow::<NSNumberHostObject>(boolean),
                NSNumberHostObject::Bool(stored) if *stored == value
            ));
            let round_trip: bool = msg![env; boolean boolValue];
            assert_eq!(round_trip, value);
        }
    }
}
