/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::errno::{set_errno, EINVAL, ENOMEM, EPERM};
use crate::mem::{ConstPtr, ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::{Environment, ThreadId};
use std::collections::HashMap;

const SS_ONSTACK: i32 = 0x0001;
const SS_DISABLE: i32 = 0x0004;
const MINSIGSTKSZ: GuestUSize = 32 * 1024;

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct stack_t {
    ss_sp: MutVoidPtr,
    ss_size: GuestUSize,
    ss_flags: i32,
}
unsafe impl SafeRead for stack_t {}

impl Default for stack_t {
    fn default() -> Self {
        Self {
            ss_sp: Ptr::null(),
            ss_size: 0,
            ss_flags: SS_DISABLE,
        }
    }
}

#[derive(Default)]
pub struct State {
    alternate_stacks: HashMap<ThreadId, stack_t>,
}

fn sigaction(env: &mut Environment, signum: i32, act: ConstVoidPtr, old_act: MutVoidPtr) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log!("TODO: sigaction({:?}, {:?}, {:?})", signum, act, old_act);
    0
}

fn signal(env: &mut Environment, signum: i32, handler: MutVoidPtr) -> MutVoidPtr {
    // TODO: handle errno properly
    set_errno(env, 0);

    log!("TODO: signal({:?}, {:?})", signum, handler);
    Ptr::null()
}

fn sigprocmask(env: &mut Environment, how: i32, set: ConstVoidPtr, old_set: MutVoidPtr) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log!("TODO: sigprocmask({}, {:?}, {:?})", how, set, old_set);
    0
}

fn sigaltstack(env: &mut Environment, ss: ConstPtr<stack_t>, old_ss: MutPtr<stack_t>) -> i32 {
    let current = env
        .libc_state
        .signal
        .alternate_stacks
        .get(&env.current_thread)
        .copied()
        .unwrap_or_default();

    if !old_ss.is_null() {
        env.mem.write(old_ss, current);
    }
    if ss.is_null() {
        return 0;
    }
    if current.ss_flags & SS_ONSTACK != 0 {
        set_errno(env, EPERM);
        return -1;
    }

    let requested = env.mem.read(ss);
    if requested.ss_flags & !SS_DISABLE != 0 {
        set_errno(env, EINVAL);
        return -1;
    }

    let next = if requested.ss_flags & SS_DISABLE != 0 {
        stack_t::default()
    } else {
        if requested.ss_size < MINSIGSTKSZ {
            set_errno(env, ENOMEM);
            return -1;
        }
        stack_t {
            ss_flags: 0,
            ..requested
        }
    };
    env.libc_state
        .signal
        .alternate_stacks
        .insert(env.current_thread, next);
    0
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sigaction(_, _, _)),
    export_c_func!(signal(_, _)),
    export_c_func!(sigprocmask(_, _, _)),
    export_c_func!(sigaltstack(_, _)),
];
