/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Synchronous Core Foundation notification centers.

use super::cf_type::CFEqual;
use super::{CFRelease, CFRetain, CFTypeRef};
use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::MutVoidPtr;
use crate::Environment;

pub type CFNotificationCenterRef = MutVoidPtr;
type CFNotificationSuspensionBehavior = u32;

#[derive(Clone, Copy)]
struct Observer {
    center: CFNotificationCenterRef,
    observer: MutVoidPtr,
    callback: GuestFunction,
    name: CFTypeRef,
    object: CFTypeRef,
}

#[derive(Default)]
pub struct State {
    local: CFNotificationCenterRef,
    darwin: CFNotificationCenterRef,
    observers: Vec<Observer>,
}

fn state(env: &mut Environment) -> &mut State {
    &mut env.framework_state.core_foundation.cf_notification_center
}

fn center(env: &mut Environment, darwin: bool) -> CFNotificationCenterRef {
    let existing = if darwin {
        state(env).darwin
    } else {
        state(env).local
    };
    if !existing.is_null() {
        return existing;
    }
    let created = env
        .mem
        .alloc_and_write(if darwin {
            0x4441_5257u32
        } else {
            0x4c4f_434cu32
        })
        .cast();
    if darwin {
        state(env).darwin = created;
    } else {
        state(env).local = created;
    }
    created
}

fn CFNotificationCenterGetLocalCenter(env: &mut Environment) -> CFNotificationCenterRef {
    center(env, false)
}

fn CFNotificationCenterGetDarwinNotifyCenter(env: &mut Environment) -> CFNotificationCenterRef {
    center(env, true)
}

fn CFNotificationCenterAddObserver(
    env: &mut Environment,
    center: CFNotificationCenterRef,
    observer: MutVoidPtr,
    callback: GuestFunction,
    name: CFTypeRef,
    object: CFTypeRef,
    _suspension_behavior: CFNotificationSuspensionBehavior,
) {
    assert!(!center.is_null());
    assert_ne!(callback.addr_without_thumb_bit(), 0);
    if !name.is_null() {
        CFRetain(env, name);
    }
    if !object.is_null() {
        CFRetain(env, object);
    }
    state(env).observers.push(Observer {
        center,
        observer,
        callback,
        name,
        object,
    });
}

fn observer_matches(env: &mut Environment, registered: CFTypeRef, posted: CFTypeRef) -> bool {
    registered.is_null()
        || (!posted.is_null() && (registered == posted || CFEqual(env, registered, posted)))
}

fn CFNotificationCenterPostNotification(
    env: &mut Environment,
    center: CFNotificationCenterRef,
    name: CFTypeRef,
    object: CFTypeRef,
    user_info: CFTypeRef,
    _deliver_immediately: bool,
) {
    // Snapshot registrations: callbacks may safely add or remove observers.
    let observers = state(env).observers.clone();
    for registration in observers {
        if registration.center == center
            && observer_matches(env, registration.name, name)
            && observer_matches(env, registration.object, object)
        {
            let _: () = registration.callback.call_from_host(
                env,
                (center, registration.observer, name, object, user_info),
            );
        }
    }
}

fn CFNotificationCenterPostNotificationWithOptions(
    env: &mut Environment,
    center: CFNotificationCenterRef,
    name: CFTypeRef,
    object: CFTypeRef,
    user_info: CFTypeRef,
    _options: u32,
) {
    CFNotificationCenterPostNotification(env, center, name, object, user_info, true);
}

fn CFNotificationCenterRemoveObserver(
    env: &mut Environment,
    center: CFNotificationCenterRef,
    observer: MutVoidPtr,
    name: CFTypeRef,
    object: CFTypeRef,
) {
    let old = std::mem::take(&mut state(env).observers);
    let mut kept = Vec::with_capacity(old.len());
    for registration in old {
        let name_matches = name.is_null() || registration.name == name;
        let object_matches = object.is_null() || registration.object == object;
        if registration.center == center
            && registration.observer == observer
            && name_matches
            && object_matches
        {
            if !registration.name.is_null() {
                CFRelease(env, registration.name);
            }
            if !registration.object.is_null() {
                CFRelease(env, registration.object);
            }
        } else {
            kept.push(registration);
        }
    }
    state(env).observers = kept;
}

fn CFNotificationCenterRemoveEveryObserver(
    env: &mut Environment,
    center: CFNotificationCenterRef,
    observer: MutVoidPtr,
) {
    CFNotificationCenterRemoveObserver(env, center, observer, CFTypeRef::null(), CFTypeRef::null());
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFNotificationCenterGetLocalCenter()),
    export_c_func!(CFNotificationCenterGetDarwinNotifyCenter()),
    export_c_func!(CFNotificationCenterAddObserver(_, _, _, _, _, _)),
    export_c_func!(CFNotificationCenterPostNotification(_, _, _, _, _)),
    export_c_func!(CFNotificationCenterPostNotificationWithOptions(
        _,
        _,
        _,
        _,
        _
    )),
    export_c_func!(CFNotificationCenterRemoveObserver(_, _, _, _)),
    export_c_func!(CFNotificationCenterRemoveEveryObserver(_, _)),
];
