/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFAllocator`.
//!
//! Only a small subset of allocators is currently supported.

use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::mem::{ConstPtr, Ptr, SafeRead};
use crate::Environment;

// CFAllocator is an opaque struct.
// It doesn't need to be a CFTypeRef compatible (seems so?),
// so it's just a struct without Obj-C counterpart.
pub struct CFAllocatorHostObject(CFAllocatorType);
unsafe impl SafeRead for CFAllocatorHostObject {}

impl CFAllocatorHostObject {
    pub fn is_system_default(&self) -> bool {
        self.0 == CFAllocatorType::SystemDefault
    }
    pub fn is_null(&self) -> bool {
        self.0 == CFAllocatorType::Null
    }
}

// TODO: support other types
#[repr(i32)]
#[derive(PartialEq)]
enum CFAllocatorType {
    /// For us, same as Default one, but not a NULL ptr.
    SystemDefault = 1,
    Null = 2,
}

pub type CFAllocatorRef = ConstPtr<CFAllocatorHostObject>;

pub const kCFAllocatorDefault: CFAllocatorRef = Ptr::null();

#[derive(Default)]
pub struct State {
    system_default: Option<CFAllocatorRef>,
}

fn system_default_allocator(env: &mut Environment) -> CFAllocatorRef {
    if let Some(allocator) = env
        .framework_state
        .core_foundation
        .cf_allocator
        .system_default
    {
        return allocator;
    }
    let allocator = env
        .mem
        .alloc_and_write(CFAllocatorHostObject(CFAllocatorType::SystemDefault))
        .cast_const();
    env.framework_state
        .core_foundation
        .cf_allocator
        .system_default = Some(allocator);
    allocator
}

fn CFAllocatorGetDefault(env: &mut Environment) -> CFAllocatorRef {
    system_default_allocator(env)
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(CFAllocatorGetDefault())];

pub const CONSTANTS: ConstantExports = &[
    ("_kCFAllocatorDefault", HostConstant::NullPtr),
    (
        "_kCFAllocatorSystemDefault",
        HostConstant::Custom(|env| {
            let allocator_ptr = system_default_allocator(env);
            env.mem.alloc_and_write(allocator_ptr).cast().cast_const()
        }),
    ),
    (
        "_kCFAllocatorNull",
        HostConstant::Custom(|env| {
            let allocator_ptr = env
                .mem
                .alloc_and_write(CFAllocatorHostObject(CFAllocatorType::Null));
            env.mem.alloc_and_write(allocator_ptr).cast().cast_const()
        }),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_245_default_allocator_is_stable_and_is_the_system_default() {
        let mut env = Environment::new_fake_for_test();
        let first = CFAllocatorGetDefault(&mut env);
        let second = CFAllocatorGetDefault(&mut env);
        assert!(!first.is_null());
        assert_eq!(first, second);
        assert!(env.mem.read(first).is_system_default());
    }
}
