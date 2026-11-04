/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFRunLoop`.
//!
//! This is not even toll-free bridged to `NSRunLoop` in Apple's implementation,
//! but here it is the same type.

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::time::CFTimeInterval;
use crate::frameworks::foundation::ns_run_loop::{
    add_source, contains_source, contains_timer, remove_source, remove_timer, run_run_loop,
    run_run_loop_single_iteration, stop_run_loop,
};
use crate::frameworks::foundation::ns_string;
use crate::frameworks::core_foundation::CFIndex;
use crate::mem::{MutPtr, MutVoidPtr, SafeRead};
use crate::objc::{
    id, msg, msg_class, msg_super, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};
use crate::Environment;

pub type CFRunLoopRef = super::CFTypeRef;
pub type CFRunLoopMode = super::cf_string::CFStringRef;
pub type CFRunLoopSourceRef = super::CFTypeRef;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CFRunLoopSourceContext {
    version: CFIndex,
    info: MutVoidPtr,
    retain: GuestFunction,
    release: GuestFunction,
    _copy_description: GuestFunction,
    _equal: GuestFunction,
    _hash: GuestFunction,
    schedule: GuestFunction,
    cancel: GuestFunction,
    perform: GuestFunction,
}
unsafe impl SafeRead for CFRunLoopSourceContext {}

struct CFRunLoopSourceHostObject {
    order: CFIndex,
    info: MutVoidPtr,
    release_info: GuestFunction,
    schedule: GuestFunction,
    cancel: GuestFunction,
    perform: GuestFunction,
    memberships: Vec<(CFRunLoopRef, CFRunLoopMode)>,
    signaled: bool,
    valid: bool,
}
impl HostObject for CFRunLoopSourceHostObject {}
impl Default for CFRunLoopSourceHostObject {
    fn default() -> Self {
        Self {
            order: 0,
            info: MutVoidPtr::null(),
            release_info: GuestFunction::null_ptr(),
            schedule: GuestFunction::null_ptr(),
            cancel: GuestFunction::null_ptr(),
            perform: GuestFunction::null_ptr(),
            memberships: Vec::new(),
            signaled: false,
            valid: false,
        }
    }
}

fn supported_mode(env: &mut Environment, mode: CFRunLoopMode) -> bool {
    let default_mode = ns_string::get_static_str(env, kCFRunLoopDefaultMode);
    let common_modes = ns_string::get_static_str(env, kCFRunLoopCommonModes);
    msg![env; mode isEqualToString:default_mode] || msg![env; mode isEqualToString:common_modes]
}

fn source_has_membership(
    env: &mut Environment,
    source: CFRunLoopSourceRef,
    run_loop: CFRunLoopRef,
    mode: CFRunLoopMode,
) -> bool {
    let memberships = env
        .objc
        .borrow::<CFRunLoopSourceHostObject>(source)
        .memberships
        .clone();
    memberships
        .into_iter()
        .any(|(candidate_loop, candidate_mode)| {
            candidate_loop == run_loop && msg![env; candidate_mode isEqualToString:mode]
        })
}

fn CFRunLoopSourceCreate(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    order: CFIndex,
    context: MutPtr<CFRunLoopSourceContext>,
) -> CFRunLoopSourceRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    assert!(!context.is_null());
    let context = env.mem.read(context);
    let version = context.version;
    assert_eq!(version, 0);

    let mut info = context.info;
    let retain = context.retain;
    if !retain.to_ptr().is_null() {
        info = retain.call_from_host(env, (info,));
    }
    let class = env
        .objc
        .get_known_class("_touchHLE_CFRunLoopSource", &mut env.mem);
    env.objc.alloc_object(
        class,
        Box::new(CFRunLoopSourceHostObject {
            order,
            info,
            release_info: context.release,
            schedule: context.schedule,
            cancel: context.cancel,
            perform: context.perform,
            memberships: Vec::new(),
            signaled: false,
            valid: true,
        }),
        &mut env.mem,
    )
}

pub(crate) fn source_order(env: &Environment, source: CFRunLoopSourceRef) -> CFIndex {
    env.objc.borrow::<CFRunLoopSourceHostObject>(source).order
}

fn CFRunLoopSourceGetOrder(env: &mut Environment, source: CFRunLoopSourceRef) -> CFIndex {
    source_order(env, source)
}

fn CFRunLoopAddSource(
    env: &mut Environment,
    run_loop: CFRunLoopRef,
    source: CFRunLoopSourceRef,
    mode: CFRunLoopMode,
) {
    if !supported_mode(env, mode) {
        log_dbg!("Treating custom CFRunLoop mode {:?} as a schedulable mode", mode);
    }
    if !env.objc.borrow::<CFRunLoopSourceHostObject>(source).valid
        || source_has_membership(env, source, run_loop, mode)
    {
        return;
    }
    retain(env, mode);
    env.objc
        .borrow_mut::<CFRunLoopSourceHostObject>(source)
        .memberships
        .push((run_loop, mode));
    add_source(env, run_loop, source);
    let (schedule, info) = {
        let source = env.objc.borrow::<CFRunLoopSourceHostObject>(source);
        (source.schedule, source.info)
    };
    if !schedule.to_ptr().is_null() {
        let _: () = schedule.call_from_host(env, (info, run_loop, mode));
    }
}

fn CFRunLoopContainsSource(
    env: &mut Environment,
    run_loop: CFRunLoopRef,
    source: CFRunLoopSourceRef,
    mode: CFRunLoopMode,
) -> bool {
    contains_source(env, run_loop, source)
        && source_has_membership(env, source, run_loop, mode)
}

fn CFRunLoopRemoveSource(
    env: &mut Environment,
    run_loop: CFRunLoopRef,
    source: CFRunLoopSourceRef,
    mode: CFRunLoopMode,
) {
    let memberships = env
        .objc
        .borrow::<CFRunLoopSourceHostObject>(source)
        .memberships
        .clone();
    let remove_index = memberships
        .into_iter()
        .enumerate()
        .find(|(_, (candidate_loop, candidate_mode))| {
            let candidate_mode = *candidate_mode;
            *candidate_loop == run_loop && msg![env; candidate_mode isEqualToString:mode]
        })
        .map(|(index, _)| index);
    let removed_mode = remove_index.map(|index| {
        env.objc
            .borrow_mut::<CFRunLoopSourceHostObject>(source)
            .memberships
            .swap_remove(index)
            .1
    });
    let Some(removed_mode) = removed_mode else {
        return;
    };
    let (cancel, info, still_on_run_loop) = {
        let host = env.objc.borrow::<CFRunLoopSourceHostObject>(source);
        (
            host.cancel,
            host.info,
            host.memberships.iter().any(|&(candidate, _)| candidate == run_loop),
        )
    };
    if !cancel.to_ptr().is_null() {
        let _: () = cancel.call_from_host(env, (info, run_loop, removed_mode));
    }
    release(env, removed_mode);
    if !still_on_run_loop {
        remove_source(env, run_loop, source);
    }
}

fn CFRunLoopSourceSignal(env: &mut Environment, source: CFRunLoopSourceRef) {
    let host = env.objc.borrow_mut::<CFRunLoopSourceHostObject>(source);
    if host.valid {
        host.signaled = true;
    }
}

fn CFRunLoopSourceIsValid(env: &mut Environment, source: CFRunLoopSourceRef) -> bool {
    env.objc.borrow::<CFRunLoopSourceHostObject>(source).valid
}

fn CFRunLoopSourceInvalidate(env: &mut Environment, source: CFRunLoopSourceRef) {
    retain(env, source);
    let memberships = {
        let host = env.objc.borrow_mut::<CFRunLoopSourceHostObject>(source);
        if !host.valid {
            release(env, source);
            return;
        }
        host.valid = false;
        host.signaled = false;
        std::mem::take(&mut host.memberships)
    };
    let (cancel, info) = {
        let host = env.objc.borrow::<CFRunLoopSourceHostObject>(source);
        (host.cancel, host.info)
    };
    for (run_loop, mode) in memberships {
        if !cancel.to_ptr().is_null() {
            let _: () = cancel.call_from_host(env, (info, run_loop, mode));
        }
        release(env, mode);
        if !env
            .objc
            .borrow::<CFRunLoopSourceHostObject>(source)
            .memberships
            .iter()
            .any(|&(candidate, _)| candidate == run_loop)
        {
            remove_source(env, run_loop, source);
        }
    }
    release(env, source);
}

pub(crate) fn handle_source(env: &mut Environment, source: CFRunLoopSourceRef) {
    let callback = {
        let host = env.objc.borrow_mut::<CFRunLoopSourceHostObject>(source);
        if !host.valid || !std::mem::take(&mut host.signaled) || host.perform.to_ptr().is_null() {
            None
        } else {
            Some((host.perform, host.info))
        }
    };
    if let Some((perform, info)) = callback {
        let _: () = perform.call_from_host(env, (info,));
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation _touchHLE_CFRunLoopSource: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<CFRunLoopSourceHostObject>::default(), &mut env.mem)
}

- (())dealloc {
    let host = std::mem::take(env.objc.borrow_mut::<CFRunLoopSourceHostObject>(this));
    assert!(host.memberships.is_empty());
    if !host.release_info.to_ptr().is_null() {
        let _: () = host.release_info.call_from_host(env, (host.info,));
    }
    msg_super![env; this dealloc]
}

@end

};

fn CFRunLoopGetCurrent(env: &mut Environment) -> CFRunLoopRef {
    msg_class![env; NSRunLoop currentRunLoop]
}

pub fn CFRunLoopGetMain(env: &mut Environment) -> CFRunLoopRef {
    msg_class![env; NSRunLoop mainRunLoop]
}

fn CFRunLoopRun(env: &mut Environment) {
    let current_run_loop = CFRunLoopGetCurrent(env);
    run_run_loop(
        env,
        current_run_loop,
        /* single_iteration: */ false,
        None,
    );
}

fn CFRunLoopStop(env: &mut Environment, run_loop: CFRunLoopRef) {
    stop_run_loop(env, run_loop);
}

fn CFRunLoopContainsTimer(
    env: &mut Environment,
    run_loop: CFRunLoopRef,
    timer: super::CFTypeRef,
    mode: CFRunLoopMode,
) -> bool {
    let default_mode = ns_string::get_static_str(env, kCFRunLoopDefaultMode);
    let common_modes = ns_string::get_static_str(env, kCFRunLoopCommonModes);
    if msg![env; mode isEqualToString:default_mode]
        || msg![env; mode isEqualToString:common_modes]
    {
        contains_timer(env, run_loop, timer)
    } else {
        // The scheduler currently models only the default/common modes. A
        // timer cannot be a member of another, unmodelled mode.
        false
    }
}

fn CFRunLoopRemoveTimer(
    env: &mut Environment,
    run_loop: CFRunLoopRef,
    timer: super::CFTypeRef,
    mode: CFRunLoopMode,
) {
    let default_mode = ns_string::get_static_str(env, kCFRunLoopDefaultMode);
    let common_modes = ns_string::get_static_str(env, kCFRunLoopCommonModes);
    if (msg![env; mode isEqualToString:default_mode]
        || msg![env; mode isEqualToString:common_modes])
        && contains_timer(env, run_loop, timer)
    {
        // The scheduler currently collapses the default and common modes into
        // one membership list. Removing a timer that is not registered is a
        // no-op, matching Core Foundation's observable contract.
        remove_timer(env, run_loop, timer);
    }
}

fn CFRunLoopRunInMode(
    env: &mut Environment,
    mode: CFRunLoopMode,
    seconds: CFTimeInterval,
    _return_something: bool,
) -> i32 {
    let default_mode = ns_string::get_static_str(env, kCFRunLoopDefaultMode);
    let common_modes = ns_string::get_static_str(env, kCFRunLoopCommonModes);
    // TODO: handle other modes
    assert!(
        msg![env; mode isEqualToString:default_mode]
            || msg![env; mode isEqualToString:common_modes]
    );
    let current_run_loop = CFRunLoopGetCurrent(env);
    if seconds == 0.0 {
        run_run_loop_single_iteration(env, current_run_loop);
    } else {
        let limit_date: id = msg_class![env; NSDate dateWithTimeIntervalSinceNow:seconds];
        () = msg![env; current_run_loop runUntilDate:limit_date];
    }
    1 // kCFRunLoopRunFinished
}

pub const kCFRunLoopCommonModes: &str = "kCFRunLoopCommonModes";
pub const kCFRunLoopDefaultMode: &str = "kCFRunLoopDefaultMode";

pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFRunLoopCommonModes",
        HostConstant::NSString(kCFRunLoopCommonModes),
    ),
    (
        "_kCFRunLoopDefaultMode",
        HostConstant::NSString(kCFRunLoopDefaultMode),
    ),
];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFRunLoopGetCurrent()),
    export_c_func!(CFRunLoopGetMain()),
    export_c_func!(CFRunLoopRun()),
    export_c_func!(CFRunLoopStop(_)),
    export_c_func!(CFRunLoopContainsTimer(_, _, _)),
    export_c_func!(CFRunLoopRemoveTimer(_, _, _)),
    export_c_func!(CFRunLoopRunInMode(_, _, _)),
    export_c_func!(CFRunLoopSourceCreate(_, _, _)),
    export_c_func!(CFRunLoopAddSource(_, _, _)),
    export_c_func!(CFRunLoopContainsSource(_, _, _)),
    export_c_func!(CFRunLoopRemoveSource(_, _, _)),
    export_c_func!(CFRunLoopSourceSignal(_)),
    export_c_func!(CFRunLoopSourceIsValid(_)),
    export_c_func!(CFRunLoopSourceInvalidate(_)),
    export_c_func!(CFRunLoopSourceGetOrder(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_zero_membership_signal_and_invalidation_are_bounded() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let stack_size = 0x1000;
        let stack = env.mem.alloc(stack_size);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + stack_size;
        let context = env.mem.alloc_and_write(CFRunLoopSourceContext {
            version: 0,
            info: MutVoidPtr::null(),
            retain: GuestFunction::null_ptr(),
            release: GuestFunction::null_ptr(),
            _copy_description: GuestFunction::null_ptr(),
            _equal: GuestFunction::null_ptr(),
            _hash: GuestFunction::null_ptr(),
            schedule: GuestFunction::null_ptr(),
            cancel: GuestFunction::null_ptr(),
            perform: GuestFunction::null_ptr(),
        });
        let source = CFRunLoopSourceCreate(&mut env, kCFAllocatorDefault, 0, context);
        let run_loop = CFRunLoopGetCurrent(&mut env);
        let mode = ns_string::get_static_str(&mut env, kCFRunLoopDefaultMode);

        assert!(CFRunLoopSourceIsValid(&mut env, source));
        CFRunLoopAddSource(&mut env, run_loop, source, mode);
        assert!(CFRunLoopContainsSource(&mut env, run_loop, source, mode));
        CFRunLoopSourceSignal(&mut env, source);
        handle_source(&mut env, source);
        assert!(!env.objc.borrow::<CFRunLoopSourceHostObject>(source).signaled);

        CFRunLoopSourceInvalidate(&mut env, source);
        assert!(!CFRunLoopSourceIsValid(&mut env, source));
        assert!(!CFRunLoopContainsSource(&mut env, run_loop, source, mode));
        release(&mut env, source);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn source_preserves_historical_order_value() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let stack_size = 0x1000;
        let stack = env.mem.alloc(stack_size);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + stack_size;
        let context = env.mem.alloc_and_write(CFRunLoopSourceContext {
            version: 0,
            info: MutVoidPtr::null(),
            retain: GuestFunction::null_ptr(),
            release: GuestFunction::null_ptr(),
            _copy_description: GuestFunction::null_ptr(),
            _equal: GuestFunction::null_ptr(),
            _hash: GuestFunction::null_ptr(),
            schedule: GuestFunction::null_ptr(),
            cancel: GuestFunction::null_ptr(),
            perform: GuestFunction::null_ptr(),
        });
        let source = CFRunLoopSourceCreate(&mut env, kCFAllocatorDefault, -7, context);
        assert_eq!(CFRunLoopSourceGetOrder(&mut env, source), -7);
        CFRunLoopSourceInvalidate(&mut env, source);
        release(&mut env, source);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
