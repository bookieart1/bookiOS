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
    deliver_offline_callback(env, this)
}
@end

};

struct SCNetworkReachabilityHostObject {
    address: Option<SocketAddrV4>,
    callback: Option<(GuestFunction, MutVoidPtr)>,
    delivered_offline_callback: bool,
    offline_callback_pending: bool,
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
            delivered_offline_callback: false,
            offline_callback_pending: false,
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
            delivered_offline_callback: false,
            offline_callback_pending: false,
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
    let callback_changed = match (host_object.callback, callback) {
        (None, None) => false,
        (Some((old, old_info)), Some((new, new_info))) => {
            old.addr_with_thumb_bit() != new.addr_with_thumb_bit() || old_info != new_info
        }
        _ => true,
    };
    host_object.callback = callback;
    if callback_changed {
        host_object.delivered_offline_callback = false;
    }
    let delivered = host_object.delivered_offline_callback;
    let pending = host_object.offline_callback_pending;
    let info = host_object.callback.map_or(0, |(_, info)| info.to_bits());
    env.diagnostic_campaign_event(
        "reachability",
        "set-callback target,callout,context,info,changed,delivered,pending",
        [
            target.to_bits(),
            callout.addr_with_thumb_bit(),
            context.to_bits(),
            info,
            u32::from(callback_changed),
            u32::from(delivered),
            u32::from(pending),
            0,
        ],
    );
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
    // Older reachability wrappers expect an initial callback after scheduling,
    // but Core Foundation never invokes it reentrantly from this scheduling
    // call. Queue one delivery on the requested run loop. Keeping an identical
    // callback registration's delivered state also prevents legacy wrappers
    // from creating an endless next-turn notification loop.
    if !env.options.network_access {
        let should_schedule = {
            let host_object = env
                .objc
                .borrow_mut::<SCNetworkReachabilityHostObject>(target);
            if host_object.delivered_offline_callback
                || host_object.offline_callback_pending
                || host_object.callback.is_none()
            {
                false
            } else {
                host_object.offline_callback_pending = true;
                true
            }
        };
        if should_schedule {
            let selector = offline_callback_selector(env);
            add_perform_request(env, run_loop, target, selector, nil, Some(0.0), false);
        }
        env.diagnostic_campaign_event(
            "reachability",
            "schedule-offline target,run-loop,mode,queued",
            [
                target.to_bits(),
                run_loop.to_bits(),
                _run_loop_mode.to_bits(),
                u32::from(should_schedule),
                0,
                0,
                0,
                0,
            ],
        );
    }
    true
}

fn offline_callback_selector(env: &mut Environment) -> SEL {
    env.objc.register_host_selector(
        "_touchHLE_deliverOfflineCallback:".to_owned(),
        &mut env.mem,
    )
}

fn deliver_offline_callback(env: &mut Environment, target: SCNetworkReachabilityRef) {
    let callback = {
        let host_object = env
            .objc
            .borrow_mut::<SCNetworkReachabilityHostObject>(target);
        host_object.offline_callback_pending = false;
        if host_object.delivered_offline_callback {
            None
        } else {
            host_object.delivered_offline_callback = true;
            host_object.callback
        }
    };
    if let Some((callback, info)) = callback {
        env.diagnostic_campaign_event(
            "reachability",
            "deliver-offline target,callout,info",
            [
                target.to_bits(),
                callback.addr_with_thumb_bit(),
                info.to_bits(),
                0,
                0,
                0,
                0,
                0,
            ],
        );
        () = callback.call_from_host(env, (target, 0_u32, info));
    }
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
        .offline_callback_pending = false;
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
                delivered_offline_callback: false,
                offline_callback_pending: false,
            }),
            &mut env.mem,
        )
    }

    #[test]
    fn identical_callback_registration_keeps_the_delivered_generation() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.options.diagnostic_campaign = crate::options::DiagnosticCampaign::Off;
        env.bundle.set_fake_identity_for_test();
        let name = env.mem.alloc_and_write_cstr(b"example.invalid");
        let target = SCNetworkReachabilityCreateWithName(
            &mut env,
            kCFAllocatorDefault,
            name.cast_const(),
        );
        let callback = GuestFunction::from_addr_with_thumb_bit(0x1001);
        assert!(SCNetworkReachabilitySetCallback(
            &mut env,
            target,
            callback,
            Ptr::null(),
        ));
        env.objc
            .borrow_mut::<SCNetworkReachabilityHostObject>(target)
            .delivered_offline_callback = true;

        assert!(SCNetworkReachabilitySetCallback(
            &mut env,
            target,
            callback,
            Ptr::null(),
        ));
        assert!(
            env.objc
                .borrow::<SCNetworkReachabilityHostObject>(target)
                .delivered_offline_callback
        );

        assert!(SCNetworkReachabilitySetCallback(
            &mut env,
            target,
            GuestFunction::from_addr_with_thumb_bit(0x2001),
            Ptr::null(),
        ));
        assert!(
            !env.objc
                .borrow::<SCNetworkReachabilityHostObject>(target)
                .delivered_offline_callback
        );
    }

    #[test]
    fn offline_registration_schedule_delivery_and_reschedule_truth_table() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.diagnostic_campaign = crate::options::DiagnosticCampaign::Off;
        env.options.network_access = false;
        let target = new_test_target(env);
        let run_loop = msg_class![env; NSRunLoop currentRunLoop];
        let callback = GuestFunction::from_addr_with_thumb_bit(0x1001);

        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            callback,
            MutVoidPtr::null(),
        ));
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        let selector = offline_callback_selector(env);
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 1);
        let state = env.objc.borrow::<SCNetworkReachabilityHostObject>(target);
        assert!(state.offline_callback_pending);
        assert!(!state.delivered_offline_callback);

        // An identical schedule while pending queues nothing else.
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 1);

        // Model the queued turn without entering a guest callback address.
        env.objc
            .borrow_mut::<SCNetworkReachabilityHostObject>(target)
            .offline_callback_pending = false;
        env.objc
            .borrow_mut::<SCNetworkReachabilityHostObject>(target)
            .delivered_offline_callback = true;
        cancel_perform_requests(env, run_loop, target, selector, nil);
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 0);

        // An identical registration preserves that generation. A changed
        // callback creates a new generation and permits exactly one queue.
        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            callback,
            MutVoidPtr::null(),
        ));
        assert!(
            env.objc
                .borrow::<SCNetworkReachabilityHostObject>(target)
                .delivered_offline_callback
        );
        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            GuestFunction::from_addr_with_thumb_bit(0x2001),
            MutVoidPtr::null(),
        ));
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 1);

        assert!(SCNetworkReachabilityUnscheduleFromRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 0);
        assert!(!env
            .objc
            .borrow::<SCNetworkReachabilityHostObject>(target)
            .offline_callback_pending);
        release(env, target);
    }

    #[test]
    fn no_callback_online_and_cancellation_paths_do_not_queue() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.options.diagnostic_campaign = crate::options::DiagnosticCampaign::Off;
        let target = new_test_target(env);
        let run_loop = msg_class![env; NSRunLoop currentRunLoop];
        let selector = offline_callback_selector(env);

        env.options.network_access = false;
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 0);

        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            GuestFunction::from_addr_with_thumb_bit(0x1001),
            MutVoidPtr::null(),
        ));
        env.options.network_access = true;
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 0);

        env.options.network_access = false;
        assert!(SCNetworkReachabilityScheduleWithRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 1);
        assert!(SCNetworkReachabilitySetCallback(
            env,
            target,
            GuestFunction::from_addr_with_thumb_bit(0),
            MutVoidPtr::null(),
        ));
        assert!(SCNetworkReachabilityUnscheduleFromRunLoop(
            env, target, run_loop, nil,
        ));
        assert_eq!(pending_perform_request_count(env, run_loop, target, selector), 0);
        release(env, target);
    }
}
