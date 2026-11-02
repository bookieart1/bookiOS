/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFArray` and `CFMutableArray`.
//!
//! These are toll-free bridged to `NSArray` and `NSMutableArray` in Apple's
//! implementation. Here they are the same types.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::CFIndex;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{ns_array, NSUInteger};
use crate::mem::ConstVoidPtr;
use crate::objc::{id, msg, msg_class};
use crate::Environment;

#[allow(dead_code)]
pub type CFArrayRef = super::CFTypeRef;
pub type CFMutableArrayRef = super::CFTypeRef;

fn CFArrayCreateMutable(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    capacity: CFIndex,
    callbacks: ConstVoidPtr, // TODO, should be `const CFArrayCallBacks*`
) -> CFMutableArrayRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert!(capacity >= 0, "CFArrayCreateMutable capacity must not be negative");
    assert!(callbacks.is_null()); // TODO: support retaining etc

    let array: id = msg_class![env; _touchHLE_NSMutableArray_non_retaining new];
    ns_array::set_fixed_capacity(env, array, capacity.try_into().unwrap());
    array
}

fn CFArrayGetCount(env: &mut Environment, array: CFArrayRef) -> CFIndex {
    let count: NSUInteger = msg![env; array count];
    count.try_into().unwrap()
}

fn CFArrayGetValueAtIndex(env: &mut Environment, array: CFArrayRef, idx: CFIndex) -> ConstVoidPtr {
    let idx: NSUInteger = idx.try_into().unwrap();
    let value: id = msg![env; array objectAtIndex:idx];
    value.cast().cast_const()
}

fn CFArrayAppendValue(env: &mut Environment, array: CFMutableArrayRef, value: ConstVoidPtr) {
    let value: id = value.cast().cast_mut();
    msg![env; array addObject:value]
}

fn CFArraySetValueAtIndex(
    env: &mut Environment,
    array: CFMutableArrayRef,
    idx: CFIndex,
    value: ConstVoidPtr,
) {
    let idx: NSUInteger = idx.try_into().unwrap();
    let value: id = value.cast().cast_mut();
    msg![env; array replaceObjectAtIndex:idx withObject:value]
}

fn CFArrayRemoveValueAtIndex(env: &mut Environment, array: CFMutableArrayRef, idx: CFIndex) {
    let idx: NSUInteger = idx.try_into().unwrap();
    msg![env; array removeObjectAtIndex:idx]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFArrayCreateMutable(_, _, _)),
    export_c_func!(CFArrayGetCount(_)),
    export_c_func!(CFArrayGetValueAtIndex(_, _)),
    export_c_func!(CFArrayAppendValue(_, _)),
    export_c_func!(CFArraySetValueAtIndex(_, _, _)),
    export_c_func!(CFArrayRemoveValueAtIndex(_, _)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_capacity_is_a_fixed_bound_and_reopens_after_removal() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let array = CFArrayCreateMutable(
            &mut env,
            kCFAllocatorDefault,
            2,
            ConstVoidPtr::null(),
        );
        let first = ConstVoidPtr::from_bits(0x10);
        let second = ConstVoidPtr::from_bits(0x20);
        let third = ConstVoidPtr::from_bits(0x30);

        CFArrayAppendValue(&mut env, array, first);
        CFArrayAppendValue(&mut env, array, second);
        assert_eq!(CFArrayGetCount(&mut env, array), 2);
        CFArrayRemoveValueAtIndex(&mut env, array, 0);
        CFArrayAppendValue(&mut env, array, third);
        assert_eq!(CFArrayGetCount(&mut env, array), 2);
        assert_eq!(CFArrayGetValueAtIndex(&mut env, array, 1), third);

        crate::objc::release(&mut env, array);
    }

    #[test]
    #[should_panic(expected = "fixed-capacity CFMutableArray cannot exceed capacity 2")]
    fn positive_capacity_rejects_growth_past_the_bound() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let array = CFArrayCreateMutable(
            &mut env,
            kCFAllocatorDefault,
            2,
            ConstVoidPtr::null(),
        );
        CFArrayAppendValue(&mut env, array, ConstVoidPtr::from_bits(0x10));
        CFArrayAppendValue(&mut env, array, ConstVoidPtr::from_bits(0x20));
        CFArrayAppendValue(&mut env, array, ConstVoidPtr::from_bits(0x30));
    }

    #[test]
    #[should_panic(expected = "CFArrayCreateMutable capacity must not be negative")]
    fn negative_capacity_is_rejected() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let _ = CFArrayCreateMutable(
            &mut env,
            kCFAllocatorDefault,
            -1,
            ConstVoidPtr::null(),
        );
    }
}
