/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal Grand Central Dispatch queue objects.

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::environment::ThreadBlock;
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, MutVoidPtr, SafeRead};
use crate::Environment;

/// The fixed header of a 32-bit Apple block literal. Captured variables, if
/// any, follow this header and are accessed by the block's invoke function.
#[repr(C)]
pub(crate) struct BlockLiteral {
    _isa: MutVoidPtr,
    _flags: i32,
    _reserved: i32,
    invoke: GuestFunction,
    _descriptor: MutVoidPtr,
}
unsafe impl SafeRead for BlockLiteral {}

fn new_queue_object(env: &mut Environment) -> MutVoidPtr {
    // Queue objects are opaque to callers. Scheduling is intentionally added
    // separately when a game demonstrates which dispatch operations it needs.
    env.mem.alloc_and_write(0x7175_6575_u32).cast()
}

fn dispatch_queue_create(
    env: &mut Environment,
    label: ConstPtr<u8>,
    _attr: ConstVoidPtr,
) -> MutVoidPtr {
    if !label.is_null() {
        log_dbg!(
            "dispatch_queue_create('{}')",
            env.mem.cstr_at_utf8(label).unwrap_or("<invalid UTF-8>")
        );
    }
    new_queue_object(env)
}

fn dispatch_group_create(env: &mut Environment) -> MutVoidPtr {
    // Dispatch objects are opaque. Group operations will be implemented when
    // an app demonstrates which scheduling behaviour it needs.
    new_queue_object(env)
}

pub(crate) fn invoke_block_synchronously(env: &mut Environment, block: MutPtr<BlockLiteral>) {
    assert!(!block.is_null());
    let invoke = env.mem.read(block).invoke;
    assert_ne!(invoke.addr_without_thumb_bit(), 0);

    () = invoke.call_from_host(env, (block.cast_void(),));
}

fn dispatch_async(env: &mut Environment, _queue: MutVoidPtr, block: MutPtr<BlockLiteral>) {
    // A full GCD implementation needs to copy stack blocks and schedule them
    // on a queue. Until that machinery exists, invoking before this function
    // returns is the safest lifetime-correct fallback: captured stack data is
    // still valid, and callers still get the work performed.
    log_once!("TODO: dispatch_async executes its block synchronously");
    invoke_block_synchronously(env, block);
}

fn dispatch_group_async(
    env: &mut Environment,
    _group: MutVoidPtr,
    _queue: MutVoidPtr,
    block: MutPtr<BlockLiteral>,
) {
    // Running the block before returning also gives the minimal group
    // semantics needed by callers: its work is complete by the time a later
    // group wait or notification can observe it. This deliberately mirrors
    // dispatch_async's lifetime-safe fallback until real queue scheduling is
    // implemented.
    log_once!("TODO: dispatch_group_async executes its block synchronously");
    invoke_block_synchronously(env, block);
}

fn dispatch_once(env: &mut Environment, predicate: MutPtr<i32>, block: MutPtr<BlockLiteral>) {
    // dispatch_once_t is a signed long on 32-bit iOS. Its representation after
    // first use is private, but zero is the public initializer. Keep an
    // explicit in-progress state so a second guest thread cannot observe the
    // predicate as complete before initialization finishes.
    const IN_PROGRESS: i32 = 1;
    const COMPLETE: i32 = -1;

    assert!(!predicate.is_null());
    loop {
        match env.mem.read(predicate) {
            0 => {
                env.mem.write(predicate, IN_PROGRESS);
                invoke_block_synchronously(env, block);
                env.mem.write(predicate, COMPLETE);
                return;
            }
            IN_PROGRESS => {
                // Guest threads are cooperative. Yield until the thread that
                // owns initialization marks the predicate complete. Recursive
                // use of the same predicate deadlocks, matching dispatch_once.
                env.yield_thread(ThreadBlock::NotBlocked);
            }
            COMPLETE => return,
            state => panic!("Invalid dispatch_once predicate state {state:#x}"),
        }
    }
}

pub const CONSTANTS: ConstantExports = &[(
    "__dispatch_main_q",
    HostConstant::Custom(|env| new_queue_object(env).cast_const()),
)];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dispatch_queue_create(_, _)),
    export_c_func!(dispatch_group_create()),
    export_c_func!(dispatch_async(_, _)),
    export_c_func!(dispatch_group_async(_, _, _)),
    export_c_func!(dispatch_once(_, _)),
];
