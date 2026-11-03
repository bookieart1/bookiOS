/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `libkern/OSAtomic.h`
//!
//! Atomic operations.
//!
//! Right now touchHLE is a single host thread application.
//! Thus, the execution of host functions couldn't be interrupted
//! by other threads. So we consider host functions to be atomic!

use crate::dyld::FunctionExports;
use crate::environment::ThreadBlock;
use crate::export_c_func;
use crate::mem::{MutPtr, MutVoidPtr};
use crate::Environment;

fn OSAtomicAdd32(env: &mut Environment, amount: i32, value_ptr: MutPtr<i32>) -> i32 {
    OSAtomicAdd32Barrier(env, amount, value_ptr)
}

fn OSAtomicAdd32Barrier(env: &mut Environment, the_amount: i32, the_value: MutPtr<i32>) -> i32 {
    let curr = env.mem.read(the_value);
    let new = curr + the_amount;
    env.mem.write(the_value, new);
    new
}

fn OSAtomicOr32(env: &mut Environment, mask: u32, value_ptr: MutPtr<u32>) -> u32 {
    OSAtomicOr32Barrier(env, mask, value_ptr)
}

fn OSAtomicOr32Barrier(env: &mut Environment, mask: u32, value_ptr: MutPtr<u32>) -> u32 {
    // Guest threads are cooperatively scheduled, so the whole host call is
    // atomic. The barrier variant has the same observable behavior here.
    let new = env.mem.read(value_ptr) | mask;
    env.mem.write(value_ptr, new);
    new
}

fn OSAtomicCompareAndSwap32(
    env: &mut Environment,
    old_value: i32,
    new_value: i32,
    the_value: MutPtr<i32>,
) -> bool {
    OSAtomicCompareAndSwap32Barrier(env, old_value, new_value, the_value)
}

fn OSAtomicCompareAndSwapIntBarrier(
    env: &mut Environment,
    old_value: i32,
    new_value: i32,
    the_value: MutPtr<i32>,
) -> bool {
    OSAtomicCompareAndSwap32Barrier(env, old_value, new_value, the_value)
}

fn OSAtomicCompareAndSwap32Barrier(
    env: &mut Environment,
    old_value: i32,
    new_value: i32,
    the_value: MutPtr<i32>,
) -> bool {
    if old_value == env.mem.read(the_value) {
        env.mem.write(the_value, new_value);
        true
    } else {
        false
    }
}

fn OSAtomicCompareAndSwapPtr(
    env: &mut Environment,
    old_value: MutVoidPtr,
    new_value: MutVoidPtr,
    the_value: MutPtr<MutVoidPtr>,
) -> bool {
    OSAtomicCompareAndSwapPtrBarrier(env, old_value, new_value, the_value)
}

fn OSAtomicCompareAndSwapPtrBarrier(
    env: &mut Environment,
    old_value: MutVoidPtr,
    new_value: MutVoidPtr,
    the_value: MutPtr<MutVoidPtr>,
) -> bool {
    if old_value == env.mem.read(the_value) {
        env.mem.write(the_value, new_value);
        true
    } else {
        false
    }
}

fn OSMemoryBarrier(_env: &mut Environment) {
    // no-op
}

fn OSSpinLockTry(env: &mut Environment, lock: MutPtr<i32>) -> bool {
    if env.mem.read(lock) != 0 {
        return false;
    }
    // Guest threads are cooperatively scheduled, so this read/write sequence
    // cannot be interrupted unless a host implementation explicitly yields.
    env.mem.write(lock, 1);
    true
}

fn OSSpinLockLock(env: &mut Environment, lock: MutPtr<i32>) {
    while !OSSpinLockTry(env, lock) {
        // Let the lock owner run instead of busy-spinning the single host
        // thread forever.
        env.yield_thread(ThreadBlock::NotBlocked);
    }
}

fn OSSpinLockUnlock(env: &mut Environment, lock: MutPtr<i32>) {
    env.mem.write(lock, 0);
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(OSAtomicAdd32(_, _)),
    export_c_func!(OSAtomicAdd32Barrier(_, _)),
    export_c_func!(OSAtomicOr32(_, _)),
    export_c_func!(OSAtomicOr32Barrier(_, _)),
    export_c_func!(OSAtomicCompareAndSwap32(_, _, _)),
    export_c_func!(OSAtomicCompareAndSwapIntBarrier(_, _, _)),
    export_c_func!(OSAtomicCompareAndSwap32Barrier(_, _, _)),
    export_c_func!(OSAtomicCompareAndSwapPtr(_, _, _)),
    export_c_func!(OSAtomicCompareAndSwapPtrBarrier(_, _, _)),
    export_c_func!(OSMemoryBarrier()),
    export_c_func!(OSSpinLockTry(_)),
    export_c_func!(OSSpinLockLock(_)),
    export_c_func!(OSSpinLockUnlock(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_or_returns_and_stores_the_new_value() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let value = env.mem.alloc_and_write(0b0101u32);
        assert_eq!(OSAtomicOr32Barrier(&mut env, 0b1010, value), 0b1111);
        assert_eq!(env.mem.read(value), 0b1111);
    }
}
