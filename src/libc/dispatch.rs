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
use std::collections::HashMap;

const QUEUE_MAGIC: u32 = 0x7175_6575;
const QUEUE_RETAIN_COUNT_OFFSET: u32 = 1;
const QUEUE_GLOBAL_OFFSET: u32 = 2;

#[derive(Default)]
pub struct State {
    main_queue: MutVoidPtr,
    global_queues: HashMap<i32, MutVoidPtr>,
    copied_blocks: HashMap<u32, OwnedBlock>,
}

struct OwnedBlock {
    copied: CopiedBlock,
    retain_count: u32,
}

/// The fixed header of a 32-bit Apple block literal. Captured variables, if
/// any, follow this header and are accessed by the block's invoke function.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BlockLiteral {
    _isa: MutVoidPtr,
    _flags: i32,
    _reserved: i32,
    invoke: GuestFunction,
    _descriptor: MutVoidPtr,
}
unsafe impl SafeRead for BlockLiteral {}

/// A guest block copied out of its caller's stack frame. Blocks used only
/// synchronously do not need this, but APIs such as NSBlockOperation own the
/// block after their factory method returns.
pub(crate) struct CopiedBlock {
    block: MutPtr<BlockLiteral>,
    dispose_helper: Option<GuestFunction>,
}

impl CopiedBlock {
    pub(crate) fn block(&self) -> MutPtr<BlockLiteral> {
        self.block
    }
}

/// Copy a 32-bit Apple block literal, including its captured payload, and run
/// the compiler-emitted copy helper when the block has one. This is the subset
/// of Block_copy required by framework objects that genuinely own a block; it
/// deliberately does not expose a guest _Block_copy symbol yet.
pub(crate) fn copy_block_for_later(
    env: &mut Environment,
    block: MutPtr<BlockLiteral>,
) -> CopiedBlock {
    const BLOCK_HAS_COPY_DISPOSE: i32 = 1 << 25;

    assert!(!block.is_null());
    let literal = env.mem.read(block);
    assert!(!literal._descriptor.is_null());
    let descriptor = literal._descriptor.cast::<u32>();
    // The first two descriptor words are reserved and the total literal size.
    let size = env.mem.read(descriptor + 1);
    assert!(
        size >= std::mem::size_of::<BlockLiteral>() as u32,
        "Invalid guest block size {size}"
    );
    let bytes = env.mem.bytes_at(block.cast_const().cast(), size).to_vec();
    let copied = env.mem.alloc(size).cast::<BlockLiteral>();
    env.mem
        .bytes_at_mut(copied.cast::<u8>(), size)
        .copy_from_slice(&bytes);

    let dispose_helper = if literal._flags & BLOCK_HAS_COPY_DISPOSE != 0 {
        let copy_helper =
            GuestFunction::from_addr_with_thumb_bit(env.mem.read(descriptor + 2));
        let dispose_helper =
            GuestFunction::from_addr_with_thumb_bit(env.mem.read(descriptor + 3));
        assert_ne!(copy_helper.addr_without_thumb_bit(), 0);
        assert_ne!(dispose_helper.addr_without_thumb_bit(), 0);
        () = copy_helper.call_from_host(env, (copied.cast_void(), block.cast_void()));
        Some(dispose_helper)
    } else {
        None
    };

    CopiedBlock {
        block: copied,
        dispose_helper,
    }
}

pub(crate) fn dispose_copied_block(env: &mut Environment, copied: CopiedBlock) {
    if let Some(dispose_helper) = copied.dispose_helper {
        () = dispose_helper.call_from_host(env, (copied.block.cast_void(),));
    }
    env.mem.free(copied.block.cast_void());
}

fn block_kind(env: &Environment, block: MutPtr<BlockLiteral>) -> Option<usize> {
    let isa = env.mem.read(block)._isa;
    (0..3).find(|&index| env.objc.block_marker(index).to_bits() == isa.to_bits())
}

fn copy_objc_block(env: &mut Environment, block: MutPtr<BlockLiteral>) -> MutPtr<BlockLiteral> {
    match block_kind(env, block) {
        Some(0) => {
            let copied = copy_block_for_later(env, block);
            let result = copied.block();
            let mut literal = env.mem.read(result);
            literal._isa = env.objc.block_marker(2).cast();
            env.mem.write(result, literal);
            env.libc_state.dispatch.copied_blocks.insert(
                result.to_bits(),
                OwnedBlock {
                    copied,
                    retain_count: 1,
                },
            );
            result
        }
        Some(1) => block,
        Some(2) => {
            let owned = env
                .libc_state
                .dispatch
                .copied_blocks
                .get_mut(&block.to_bits())
                .expect("malloc Block is not owned by the Block runtime");
            owned.retain_count = owned.retain_count.checked_add(1).unwrap();
            block
        }
        _ => panic!("Object is not a recognized legacy Block"),
    }
}

fn release_objc_block(env: &mut Environment, block: MutPtr<BlockLiteral>) {
    if block_kind(env, block) != Some(2) {
        return;
    }
    let bits = block.to_bits();
    let should_dispose = {
        let owned = env
            .libc_state
            .dispatch
            .copied_blocks
            .get_mut(&bits)
            .expect("malloc Block is not owned by the Block runtime");
        assert_ne!(owned.retain_count, 0);
        owned.retain_count -= 1;
        owned.retain_count == 0
    };
    if should_dispose {
        let owned = env.libc_state.dispatch.copied_blocks.remove(&bits).unwrap();
        dispose_copied_block(env, owned.copied);
    }
}

/// Handle the NSObject-compatible surface implemented by legacy Blocks.
/// Returns whether the selector was handled and whether it returned an object.
pub(crate) fn try_objc_block_message(
    env: &mut Environment,
    receiver: MutPtr<BlockLiteral>,
    selector: &str,
) -> Option<bool> {
    block_kind(env, receiver)?;
    match selector {
        "copy" | "copyWithZone:" => {
            let copied = copy_objc_block(env, receiver);
            env.cpu.regs_mut()[0] = copied.to_bits();
            env.cpu.regs_mut()[1] = 0;
            Some(true)
        }
        "retain" => {
            if block_kind(env, receiver) == Some(2) {
                let _ = copy_objc_block(env, receiver);
            }
            env.cpu.regs_mut()[0] = receiver.to_bits();
            env.cpu.regs_mut()[1] = 0;
            Some(true)
        }
        "release" => {
            release_objc_block(env, receiver);
            env.cpu.regs_mut()[0..2].fill(0);
            Some(false)
        }
        "autorelease" => {
            // Blocks copied by framework owners are released by those owners.
            // Legacy stack/global Blocks do not gain lifetime from autorelease.
            env.cpu.regs_mut()[0] = receiver.to_bits();
            env.cpu.regs_mut()[1] = 0;
            Some(true)
        }
        _ => None,
    }
}

fn new_queue_object(env: &mut Environment, global: bool) -> MutVoidPtr {
    // Queue objects are opaque to callers. Scheduling is intentionally added
    // separately when a game demonstrates which dispatch operations it needs.
    let queue = env.mem.alloc(12).cast::<u32>();
    env.mem.write(queue, QUEUE_MAGIC);
    env.mem.write(queue + QUEUE_RETAIN_COUNT_OFFSET, 1);
    env.mem.write(queue + QUEUE_GLOBAL_OFFSET, u32::from(global));
    queue.cast()
}

fn get_main_queue(env: &mut Environment) -> MutVoidPtr {
    let existing = env.libc_state.dispatch.main_queue;
    if !existing.is_null() {
        return existing;
    }
    let queue = new_queue_object(env, true);
    env.libc_state.dispatch.main_queue = queue;
    queue
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
    new_queue_object(env, false)
}

fn dispatch_group_create(env: &mut Environment) -> MutVoidPtr {
    // Dispatch objects are opaque. Group operations will be implemented when
    // an app demonstrates which scheduling behaviour it needs.
    new_queue_object(env, false)
}

fn dispatch_get_global_queue(
    env: &mut Environment,
    identifier: i32,
    flags: u32,
) -> MutVoidPtr {
    // iOS 4 exposed the four documented global priorities. The flags argument
    // was reserved and required to be zero.
    const DISPATCH_QUEUE_PRIORITY_HIGH: i32 = 2;
    const DISPATCH_QUEUE_PRIORITY_DEFAULT: i32 = 0;
    const DISPATCH_QUEUE_PRIORITY_LOW: i32 = -2;
    const DISPATCH_QUEUE_PRIORITY_BACKGROUND: i32 = i16::MIN as i32;

    if flags != 0
        || !matches!(
            identifier,
            DISPATCH_QUEUE_PRIORITY_HIGH
                | DISPATCH_QUEUE_PRIORITY_DEFAULT
                | DISPATCH_QUEUE_PRIORITY_LOW
                | DISPATCH_QUEUE_PRIORITY_BACKGROUND
        )
    {
        return MutVoidPtr::null();
    }
    if let Some(&queue) = env.libc_state.dispatch.global_queues.get(&identifier) {
        return queue;
    }
    let queue = new_queue_object(env, true);
    env.libc_state
        .dispatch
        .global_queues
        .insert(identifier, queue);
    queue
}

fn dispatch_retain(env: &mut Environment, object: MutVoidPtr) {
    if object.is_null() {
        return;
    }
    let queue = object.cast::<u32>();
    assert_eq!(env.mem.read(queue), QUEUE_MAGIC);
    if env.mem.read(queue + QUEUE_GLOBAL_OFFSET) != 0 {
        return;
    }
    let count = env.mem.read(queue + QUEUE_RETAIN_COUNT_OFFSET);
    env.mem
        .write(queue + QUEUE_RETAIN_COUNT_OFFSET, count.checked_add(1).unwrap());
}

fn dispatch_release(env: &mut Environment, object: MutVoidPtr) {
    if object.is_null() {
        return;
    }
    let queue = object.cast::<u32>();
    assert_eq!(env.mem.read(queue), QUEUE_MAGIC);
    if env.mem.read(queue + QUEUE_GLOBAL_OFFSET) != 0 {
        return;
    }
    let count = env.mem.read(queue + QUEUE_RETAIN_COUNT_OFFSET);
    assert_ne!(count, 0);
    if count == 1 {
        env.mem.free(object);
    } else {
        env.mem.write(queue + QUEUE_RETAIN_COUNT_OFFSET, count - 1);
    }
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
    HostConstant::Custom(|env| get_main_queue(env).cast_const()),
)];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dispatch_queue_create(_, _)),
    export_c_func!(dispatch_group_create()),
    export_c_func!(dispatch_get_global_queue(_, _)),
    export_c_func!(dispatch_retain(_)),
    export_c_func!(dispatch_release(_)),
    export_c_func!(dispatch_async(_, _)),
    export_c_func!(dispatch_group_async(_, _, _)),
    export_c_func!(dispatch_once(_, _)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_255_dispatch_global_queue_lifetime_contract() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let default = dispatch_get_global_queue(&mut env, 0, 0);
        assert!(!default.is_null());
        assert_eq!(dispatch_get_global_queue(&mut env, 0, 0), default);
        dispatch_retain(&mut env, default);
        dispatch_release(&mut env, default);
        assert_eq!(dispatch_get_global_queue(&mut env, 123, 0), MutVoidPtr::null());
        assert_eq!(dispatch_get_global_queue(&mut env, 0, 1), MutVoidPtr::null());
    }

    #[test]
    fn development_255_dispatch_created_queue_retain_release_contract() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let queue = new_queue_object(&mut env, false);
        dispatch_retain(&mut env, queue);
        let words = queue.cast::<u32>();
        assert_eq!(env.mem.read(words + QUEUE_RETAIN_COUNT_OFFSET), 2);
        dispatch_release(&mut env, queue);
        assert_eq!(env.mem.read(words + QUEUE_RETAIN_COUNT_OFFSET), 1);
        dispatch_release(&mut env, queue);
    }

    #[test]
    fn development_255_legacy_objc_stack_block_copy_contract() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let stack_marker = env.mem.alloc_and_write(0u8).cast();
        let global_marker = env.mem.alloc_and_write(0u8).cast();
        let malloc_marker = env.mem.alloc_and_write(0u8).cast();
        env.objc.set_block_marker(0, stack_marker);
        env.objc.set_block_marker(1, global_marker);
        env.objc.set_block_marker(2, malloc_marker);

        let descriptor = env.mem.alloc(8).cast::<u32>();
        env.mem.write(descriptor, 0);
        env.mem.write(descriptor + 1, std::mem::size_of::<BlockLiteral>() as u32);
        let block = env.mem.alloc_and_write(BlockLiteral {
            _isa: stack_marker.cast(),
            _flags: 0,
            _reserved: 0,
            invoke: GuestFunction::from_addr_with_thumb_bit(0),
            _descriptor: descriptor.cast(),
        });

        assert_eq!(try_objc_block_message(&mut env, block, "copy"), Some(true));
        let copied = MutPtr::<BlockLiteral>::from_bits(env.cpu.regs()[0]);
        assert_ne!(copied, block);
        assert_eq!(env.mem.read(copied)._isa, malloc_marker.cast());
        assert_eq!(env.libc_state.dispatch.copied_blocks.len(), 1);
        assert_eq!(try_objc_block_message(&mut env, copied, "retain"), Some(true));
        assert_eq!(
            env.libc_state
                .dispatch
                .copied_blocks
                .get(&copied.to_bits())
                .unwrap()
                .retain_count,
            2
        );
        assert_eq!(try_objc_block_message(&mut env, copied, "release"), Some(false));
        assert_eq!(try_objc_block_message(&mut env, copied, "release"), Some(false));
        assert!(env.libc_state.dispatch.copied_blocks.is_empty());
    }
}
