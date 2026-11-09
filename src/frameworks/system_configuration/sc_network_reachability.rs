/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! SCNetworkReachability

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::frameworks::foundation::ns_run_loop::{
    add_perform_request, cancel_perform_requests,
};
use crate::libc::sys::socket::sockaddr;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::objc::{id, msg, nil, objc_classes, Class, ClassExports, HostObject, SEL};
use crate::Environment;
use std::net::SocketAddrV4;

type SCNetworkReachabilityFlags = u32;
const kSCNetworkReachabilityFlagsReachable: SCNetworkReachabilityFlags = 1 << 1;
const kSCNetworkReachabilityFlagsIsDirect: SCNetworkReachabilityFlags = 1 << 17;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// SCNetworkReachabilityRef is not explicitly stated to be CFType-based type,
// but the result of "Create" or "Copy" functions here is expected to be
// released with CFRelease().
@implementation _touchHLE_SCNetworkReachability: NSObject

- (())_touchHLE_deliverOfflineCallback:(id)_unused {
    let callback = take_offline_callback_for_delivery(env, this);
    if let Some((callback, info)) = callback {
        log!("Delivering deferred offline SCNetworkReachability callback for {:?}", this);
        () = callback.call_from_host(env, (this, 0_u32, info));
    }
}

@end

};

struct SCNetworkReachabilityHostObject {
    address: Option<SocketAddrV4>,
    callback: Option<(GuestFunction, MutVoidPtr)>,
    offline_callback_scheduled: bool,
    delivered_offline_callback: bool,
}
impl HostObject for SCNetworkReachabilityHostObject {}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct SCNetworkReachabilityContext {
    version: i32,
    info: MutVoidPtr,
    _retain: GuestFunction,
    _release: GuestFunction,
    _copy_description: GuestFunction,
}
unsafe impl SafeRead for SCNetworkReachabilityContext {}

// See comment for `_touchHLE_SCNetworkReachability` class
type SCNetworkReachabilityRef = CFTypeRef;

fn SCNetworkReachabilityCreateWithName(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    name: ConstPtr<u8>,
) -> SCNetworkReachabilityRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    if env
        .bundle
        .bundle_identifier()
        .starts_with("com.chillingo.cuttherope")
        && env.mem.cstr_at_utf8(name).unwrap() == "chillingo-crystal.appspot.com"
    {
        log!("Applying game-specific hack for Cut the Rope: SCNetworkReachabilityCreateWithName(\"chillingo-crystal.appspot.com\") returns NULL");
        return Ptr::null();
    }
    let isa = env
        .objc
        .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem);
    let res = env.objc.alloc_object(
        isa,
        Box::new(SCNetworkReachabilityHostObject {
            address: None,
            callback: None,
            offline_callback_scheduled: false,
            delivered_offline_callback: false,
        }), // TODO
        &mut env.mem,
    );
    log!(
        "TODO: SCNetworkReachabilityCreateWithName({:?}, {:?} {:?}) -> {:?}",
        allocator,
        name,
        env.mem.cstr_at_utf8(name),
        res
    );
    res
}

fn SCNetworkReachabilityCreateWithAddress(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    address: ConstPtr<sockaddr>,
) -> SCNetworkReachabilityRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let isa = env
        .objc
        .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem);
    let address_val = env.mem.read(address);
    let res = env.objc.alloc_object(
        isa,
        Box::new(SCNetworkReachabilityHostObject {
            address: Some(address_val.to_sockaddr_v4()),
            callback: None,
            offline_callback_scheduled: false,
            delivered_offline_callback: false,
        }),
        &mut env.mem,
    );
    log_dbg!(
        "SCNetworkReachabilityCreateWithAddress({:?}, {:?} ({})) -> {:?}",
        allocator,
        address,
        address_val.to_sockaddr_v4(),
        res
    );
    res
}

fn SCNetworkReachabilityGetFlags(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    flags: MutPtr<SCNetworkReachabilityFlags>,
) -> bool {
    if !env.options.network_access {
        log_dbg!(
            "Network access is disabled, SCNetworkReachabilityGetFlags({:?}, {:?}) -> false",
            target,
            flags
        );
        // The query itself succeeded; an empty flag set means offline. Some
        // older reachability wrappers do not safely handle a failed query.
        if !flags.is_null() {
            env.mem.write(flags, 0);
        }
        return true;
    }

    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    let host_object = env.objc.borrow::<SCNetworkReachabilityHostObject>(target);
    if let Some(addr) = host_object.address {
        if addr.ip().is_link_local() {
            log_dbg!(
                "SCNetworkReachabilityGetFlags({:?}, {:?}) -> true",
                target,
                flags
            );
            // Those corresponds to local WiFi connection on a real iOS device
            // TODO: actually check for the connectivity
            // (but do we _really_ need it?)
            let out_flags =
                kSCNetworkReachabilityFlagsReachable | kSCNetworkReachabilityFlagsIsDirect;
            env.mem.write(flags, out_flags);
            return true;
        }
    }
    if !flags.is_null() {
        env.mem.write(flags, 0);
    }
    log!(
        "SCNetworkReachabilityGetFlags({:?}, {:?}) -> true (not reachable)",
        target,
        flags
    );
    true
}

fn SCNetworkReachabilitySetCallback(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    callout: GuestFunction, // SCNetworkReachabilityCallBack
    context: MutVoidPtr,    // SCNetworkReachabilityContext *
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    let callback = if callout.addr_without_thumb_bit() == 0 {
        None
    } else {
        let info = if context.is_null() {
            MutVoidPtr::null()
        } else {
            let context: SCNetworkReachabilityContext = env.mem.read(context.cast());
            let version = context.version;
            assert_eq!(version, 0);
            context.info
        };
        Some((callout, info))
    };
    let host_object = env
        .objc
        .borrow_mut::<SCNetworkReachabilityHostObject>(target);
    host_object.callback = callback;
    host_object.delivered_offline_callback = false;
    log!(
        "SCNetworkReachabilitySetCallback({:?}, {:?}, {:?}) -> TRUE",
        target,
        callout,
        context
    );
    true
}

fn SCNetworkReachabilityScheduleWithRunLoop(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    // Older reachability wrappers expect an initial callback after scheduling.
    // Queue one stable offline result when networking is disabled. Delivery
    // must occur from the run loop rather than re-entering the guest while
    // SCNetworkReachabilityScheduleWithRunLoop is still on its stack: older
    // wrappers may install their singleton only after their constructor
    // returns.
    if !env.options.network_access {
        let should_schedule = {
            let host_object = env
                .objc
                .borrow_mut::<SCNetworkReachabilityHostObject>(target);
            if host_object.delivered_offline_callback
                || host_object.offline_callback_scheduled
                || host_object.callback.is_none()
            {
                false
            } else {
                host_object.offline_callback_scheduled = true;
                true
            }
        };
        if should_schedule {
            let selector = offline_callback_selector(env);
            add_perform_request(
                env,
                run_loop,
                target,
                selector,
                nil,
                Some(0.0),
                false,
            );
            log!(
                "Queued deferred offline SCNetworkReachability callback for {:?} on run loop {:?}",
                target,
                run_loop
            );
        }
    }
    true
}

fn offline_callback_selector(env: &mut Environment) -> SEL {
    env.objc.register_host_selector(
        "_touchHLE_deliverOfflineCallback:".to_owned(),
        &mut env.mem,
    )
}

fn take_offline_callback_for_delivery(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
) -> Option<(GuestFunction, MutVoidPtr)> {
    let host_object = env
        .objc
        .borrow_mut::<SCNetworkReachabilityHostObject>(target);
    if host_object.delivered_offline_callback || !host_object.offline_callback_scheduled {
        return None;
    }
    host_object.offline_callback_scheduled = false;
    host_object.delivered_offline_callback = true;
    host_object.callback
}

fn SCNetworkReachabilityUnscheduleFromRunLoop(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    let selector = offline_callback_selector(env);
    cancel_perform_requests(env, run_loop, target, selector, nil);
    env.objc
        .borrow_mut::<SCNetworkReachabilityHostObject>(target)
        .offline_callback_scheduled = false;
    true
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(SCNetworkReachabilityCreateWithName(_, _)),
    export_c_func!(SCNetworkReachabilityCreateWithAddress(_, _)),
    export_c_func!(SCNetworkReachabilityGetFlags(_, _)),
    export_c_func!(SCNetworkReachabilitySetCallback(_, _, _)),
    export_c_func!(SCNetworkReachabilityScheduleWithRunLoop(_, _, _)),
    export_c_func!(SCNetworkReachabilityUnscheduleFromRunLoop(_, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_run_loop::pending_perform_request_count;
    use crate::objc::{msg_class, release};

    fn new_test_target(env: &mut Environment) -> SCNetworkReachabilityRef {
        let class = env
            .objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem);
        env.objc.alloc_object(
            class,
            Box::new(SCNetworkReachabilityHostObject {
                address: None,
                callback: None,
                offline_callback_scheduled: false,
                delivered_offline_callback: false,
            }),
            &mut env.mem,
        )
    }

    #[test]
    fn offline_callback_is_queued_not_delivered_synchronously_and_only_once() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let target = new_test_target(env);
        let run_loop = msg_class![env; NSRunLoop currentRunLoop];
        let callback = GuestFunction::from_addr_with_thumb_bit(0x1000);
        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            callback,
            MutVoidPtr::null(),
        ));

        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env,
            target,
            run_loop,
            nil,
        ));
        let selector = offline_callback_selector(env);
        assert_eq!(
            pending_perform_request_count(env, run_loop, target, selector),
            1
        );
        let state = env.objc.borrow::<SCNetworkReachabilityHostObject>(target);
        assert!(state.offline_callback_scheduled);
        assert!(!state.delivered_offline_callback);

        let queued = take_offline_callback_for_delivery(env, target);
        assert_eq!(
            queued.map(|(function, _)| function.addr_without_thumb_bit()),
            Some(callback.addr_without_thumb_bit())
        );
        assert!(take_offline_callback_for_delivery(env, target).is_none());

        assert!(SCNetworkReachabilityUnscheduleFromRunLoop(
            env,
            target,
            run_loop,
            nil,
        ));
        release(env, target);
    }

    #[test]
    fn unschedule_cancels_pending_offline_callback() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let target = new_test_target(env);
        let run_loop = msg_class![env; NSRunLoop currentRunLoop];
        let callback = GuestFunction::from_addr_with_thumb_bit(0x1000);
        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            callback,
            MutVoidPtr::null(),
        ));
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env,
            target,
            run_loop,
            nil,
        ));
        assert!(SCNetworkReachabilityUnscheduleFromRunLoop(
            env,
            target,
            run_loop,
            nil,
        ));

        let selector = offline_callback_selector(env);
        assert_eq!(
            pending_perform_request_count(env, run_loop, target, selector),
            0
        );
        assert!(take_offline_callback_for_delivery(env, target).is_none());

        release(env, target);
    }
}
