/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Thread-specific data keys.

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::errno::{EAGAIN, EINVAL};
use crate::mem::{ConstVoidPtr, MutPtr, MutVoidPtr, Ptr};
use crate::{Environment, ThreadId};
use std::collections::HashMap;

const PTHREAD_KEYS_MAX: i32 = 512;

#[derive(Default)]
pub struct State {
    /// The `pthread_key_t` value, with 1 subtracted, is the index into this
    /// vector. The tuple contains the map of thread-specific data pointers plus
    /// the destructor pointer.
    keys: Vec<Option<(HashMap<ThreadId, MutVoidPtr>, GuestFunction)>>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.libc_state.pthread.key
    }
}

type pthread_key_t = u32;

fn pthread_key_create(
    env: &mut Environment,
    key_ptr: MutPtr<pthread_key_t>,
    destructor: GuestFunction, // void (*destructor)(void *), may be NULL
) -> i32 {
    for (idx, slot) in State::get(env).keys.iter_mut().enumerate() {
        if slot.is_none() {
            let key: pthread_key_t = (idx + 1).try_into().unwrap();
            *slot = Some((HashMap::new(), destructor));
            env.mem.write(key_ptr, key);
            return 0; // success, reused old slot
        }
    }
    if State::get(env).keys.len() < PTHREAD_KEYS_MAX as usize {
        let idx = State::get(env).keys.len();
        let key: pthread_key_t = (idx + 1).try_into().unwrap(); // can unwrap, PTHREAD_KEYS_MAX is small enough
        State::get(env)
            .keys
            .push(Some((HashMap::new(), destructor)));
        env.mem.write(key_ptr, key);
        return 0; // success, created new slot
    }
    EAGAIN // failure, no slots left
}

fn pthread_getspecific(env: &mut Environment, key: pthread_key_t) -> MutVoidPtr {
    // Use of invalid key is undefined, panicking is fine.
    let idx: usize = key.checked_sub(1).unwrap().try_into().unwrap();
    let current_thread = env.current_thread;
    match State::get(env).keys.get(idx) {
        Some(Some((map, _))) => map.get(&current_thread).copied().unwrap_or(Ptr::null()),
        _ => Ptr::null(), // failure, NULL for a deleted key, UB for an invalid key
    }
}

fn pthread_setspecific(env: &mut Environment, key: pthread_key_t, value: ConstVoidPtr) -> i32 {
    let Some(idx) = key.checked_sub(1).map(|idx| idx as usize) else {
        return EINVAL;
    };
    let current_thread = env.current_thread;
    match State::get(env).keys.get_mut(idx) {
        Some(Some((map, _))) => {
            map.insert(current_thread, value.cast_mut());
            0 // success
        }
        _ => EINVAL, // failure, unused key or out of bounds
    }
}

fn pthread_key_delete(env: &mut Environment, key: pthread_key_t) -> i32 {
    let Some(idx) = key.checked_sub(1).map(|idx| idx as usize) else {
        return EINVAL;
    };
    match State::get(env).keys.get_mut(idx) {
        Some(slot @ Some(_)) => {
            *slot = None;
            0 // success
        }
        _ => EINVAL, // failure, unused key or out of bounds
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(pthread_key_create(_, _)),
    export_c_func!(pthread_getspecific(_)),
    export_c_func!(pthread_setspecific(_, _)),
    export_c_func!(pthread_key_delete(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_243_invalid_zero_and_out_of_range_keys_return_einval_without_allocating() {
        let mut env = Environment::new_fake_for_test();
        assert_eq!(pthread_setspecific(&mut env, 0, Ptr::null()), EINVAL);
        assert_eq!(pthread_key_delete(&mut env, 0), EINVAL);
        assert_eq!(pthread_setspecific(&mut env, 999, Ptr::null()), EINVAL);
        assert_eq!(pthread_key_delete(&mut env, 999), EINVAL);

        let key_ptr = env.mem.alloc(4).cast();
        assert_eq!(
            pthread_key_create(&mut env, key_ptr, GuestFunction::from_addr_with_thumb_bit(0)),
            0
        );
        let key: pthread_key_t = env.mem.read(key_ptr);
        assert_eq!(key, 1);
        assert_eq!(pthread_setspecific(&mut env, key, Ptr::null()), 0);
        assert_eq!(pthread_key_delete(&mut env, key), 0);
        env.mem.free(key_ptr.cast());
    }
}
