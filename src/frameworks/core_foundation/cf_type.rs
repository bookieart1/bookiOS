/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFType` (type-generic functions etc).

use super::{CFHashCode, CFIndex};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::NSUInteger;
use crate::objc::{Class, ObjC};
use crate::{msg, objc};
use crate::{msg_class, Environment};

pub type CFTypeRef = objc::id;
pub type CFTypeID = u32;

/// Stable, opaque identity for the toll-free bridged CFString/NSString family.
/// The numeric value is private to this process; only equality is meaningful.
pub(crate) const CF_STRING_TYPE_ID: CFTypeID = 0x5354_5201;

/// Core Foundation type identifiers are opaque and process-local. Toll-free
/// bridged class clusters must share one identity even when their concrete
/// Objective-C subclasses differ (for example constant, mutable and ordinary
/// strings). Families without a public GetTypeID implementation still use the
/// stable dynamic class as their process-local identity.
pub fn CFGetTypeID(env: &mut Environment, object: CFTypeRef) -> CFTypeID {
    assert!(!object.is_null());
    let object_class = ObjC::read_isa(object, &env.mem);
    let string_class = env.objc.get_known_class("NSString", &mut env.mem);
    if env.objc.class_is_subclass_of(object_class, string_class) {
        CF_STRING_TYPE_ID
    } else {
        object_class.to_bits()
    }
}

pub fn CFRetain(env: &mut Environment, object: CFTypeRef) -> CFTypeRef {
    assert!(!object.is_null()); // not allowed, unlike for normal objc objects
    objc::retain(env, object)
}

pub fn CFRelease(env: &mut Environment, object: CFTypeRef) {
    if object.is_null() {
        return;
    }
    // Some apps try to release the same object twice when closing. The first
    // release clears the memory, so we can detect this and skip the already
    // freed object.
    if ObjC::read_isa(object, &env.mem).is_null() {
        log!(
            "Warning: CFRelease called on already-freed object {:?}",
            object
        );
        return;
    }
    objc::release(env, object);
}

pub fn CFGetRetainCount(env: &mut Environment, object: CFTypeRef) -> CFIndex {
    let count: NSUInteger = msg![env; object retainCount];
    count as CFIndex
}

pub fn CFEqual(env: &mut Environment, object1: CFTypeRef, object2: CFTypeRef) -> bool {
    if object1 == object2 {
        return true;
    }
    // TODO: other classes
    let str_class: Class = msg_class![env; NSString class];
    let object1_class: Class = msg![env; object1 class];
    assert!(msg![env; object1_class isKindOfClass:str_class]);
    let object2_class: Class = msg![env; object2 class];
    assert!(msg![env; object2_class isKindOfClass:str_class]);
    // TODO: use isEqual: once it is fixed
    msg![env; object1 isEqualToString:object2]
}

pub fn CFHash(env: &mut Environment, object: CFTypeRef) -> CFHashCode {
    msg![env; object hash]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFGetTypeID(_)),
    export_c_func!(CFRetain(_)),
    export_c_func!(CFRelease(_)),
    export_c_func!(CFGetRetainCount(_)),
    export_c_func!(CFEqual(_, _)),
    export_c_func!(CFHash(_)),
];
