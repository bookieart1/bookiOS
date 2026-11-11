/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! A bounded, offline implementation of the historical `CFHost` API.

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::{CFIndex, CFTypeRef};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::libc::errno::ENETUNREACH;
use crate::mem::{MutPtr, MutVoidPtr, SafeRead};
use crate::objc::{id, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject};
use crate::Environment;

type CFHostRef = CFTypeRef;
type CFHostInfoType = i32;
type CFReadStreamRef = id;
type CFWriteStreamRef = id;
type Boolean = u8;

const K_CF_STREAM_ERROR_DOMAIN_POSIX: CFIndex = 1;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
struct CFStreamError {
    domain: CFIndex,
    error: i32,
}
unsafe impl SafeRead for CFStreamError {}

struct CFHostHostObject {
    name: id, // CFStringRef
}
impl HostObject for CFHostHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CFHostRef is an opaque Core Foundation type whose Create functions return
// an object owned by the caller and released with CFRelease().
@implementation _touchHLE_CFHost: NSObject

- (())dealloc {
    let name = env.objc.borrow::<CFHostHostObject>(this).name;
    release(env, name);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

fn alloc_host(env: &mut Environment, name: id) -> CFHostRef {
    retain(env, name);
    let class = env.objc.get_known_class("_touchHLE_CFHost", &mut env.mem);
    env.objc
        .alloc_object(class, Box::new(CFHostHostObject { name }), &mut env.mem)
}

fn CFHostCreateWithName(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    hostname: id, // CFStringRef
) -> CFHostRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    if hostname == nil {
        return nil;
    }

    let host = alloc_host(env, hostname);
    log_dbg!(
        "CFHostCreateWithName(\"{}\") -> {:?}",
        to_rust_string(env, hostname),
        host
    );
    host
}

fn CFHostCreateCopy(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    host: CFHostRef,
) -> CFHostRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    if host == nil {
        return nil;
    }

    let name = env.objc.borrow::<CFHostHostObject>(host).name;
    alloc_host(env, name)
}

fn CFHostSetClient(
    _env: &mut Environment,
    _host: CFHostRef,
    callback: GuestFunction,
    _context: MutVoidPtr,
) -> bool {
    if callback.to_ptr().is_null() {
        // Disassociating a client is fully supported because no callback state
        // is retained by this offline implementation.
        true
    } else {
        log_once!(
            "CFHost asynchronous clients are unavailable; host resolution will report offline"
        );
        false
    }
}

fn CFHostScheduleWithRunLoop(
    _env: &mut Environment,
    _host: CFHostRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) {
    // No resolution can be scheduled after CFHostSetClient reports failure.
}

fn CFHostUnscheduleFromRunLoop(
    _env: &mut Environment,
    _host: CFHostRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) {
}

fn CFHostStartInfoResolution(
    env: &mut Environment,
    host: CFHostRef,
    info: CFHostInfoType,
    error: MutPtr<CFStreamError>,
) -> bool {
    let name = env.objc.borrow::<CFHostHostObject>(host).name;
    log!(
        "CFHostStartInfoResolution(\"{}\", {}) is unavailable while offline",
        to_rust_string(env, name),
        info
    );
    if !error.is_null() {
        env.mem.write(
            error,
            CFStreamError {
                domain: K_CF_STREAM_ERROR_DOMAIN_POSIX,
                error: ENETUNREACH,
            },
        );
    }
    false
}

fn unresolved_result(env: &mut Environment, resolved: MutPtr<Boolean>) -> CFTypeRef {
    if !resolved.is_null() {
        env.mem.write(resolved, 0);
    }
    nil
}

fn CFHostGetAddressing(
    env: &mut Environment,
    _host: CFHostRef,
    resolved: MutPtr<Boolean>,
) -> CFTypeRef {
    unresolved_result(env, resolved)
}

fn CFHostGetNames(env: &mut Environment, _host: CFHostRef, resolved: MutPtr<Boolean>) -> CFTypeRef {
    unresolved_result(env, resolved)
}

fn CFHostGetReachability(
    env: &mut Environment,
    _host: CFHostRef,
    resolved: MutPtr<Boolean>,
) -> CFTypeRef {
    unresolved_result(env, resolved)
}

fn CFHostCancelInfoResolution(_env: &mut Environment, _host: CFHostRef, _info: CFHostInfoType) {}

/// Share the offline socket-stream lifecycle with the string-host variant.
fn CFStreamCreatePairWithSocketToCFHost(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    host: CFHostRef,
    port: i32,
    read_stream: MutPtr<CFReadStreamRef>,
    write_stream: MutPtr<CFWriteStreamRef>,
) {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());

    let name = env.objc.borrow::<CFHostHostObject>(host).name;
    crate::frameworks::core_foundation::cf_socket_stream::create_pair(env, name, port.try_into().unwrap(), read_stream, write_stream);
}

fn CFNetworkCopySystemProxySettings(env: &mut Environment) -> CFTypeRef {
    // The compatibility lane is intentionally offline and has no configured
    // guest proxy. Core Foundation's Copy rule still requires a retained
    // dictionary rather than NULL.
    msg_class![env; NSDictionary new]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFHostCreateWithName(_, _)),
    export_c_func!(CFHostCreateCopy(_, _)),
    export_c_func!(CFHostSetClient(_, _, _)),
    export_c_func!(CFHostScheduleWithRunLoop(_, _, _)),
    export_c_func!(CFHostUnscheduleFromRunLoop(_, _, _)),
    export_c_func!(CFHostStartInfoResolution(_, _, _)),
    export_c_func!(CFHostGetAddressing(_, _)),
    export_c_func!(CFHostGetNames(_, _)),
    export_c_func!(CFHostGetReachability(_, _)),
    export_c_func!(CFHostCancelInfoResolution(_, _)),
    export_c_func!(CFStreamCreatePairWithSocketToCFHost(_, _, _, _, _)),
    export_c_func!(CFNetworkCopySystemProxySettings()),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;

    #[test]
    fn system_proxy_settings_function_is_exported() {
        assert!(FUNCTIONS
            .iter()
            .any(|(name, _)| *name == "_CFNetworkCopySystemProxySettings"));
    }

    #[test]
    fn named_host_reports_bounded_offline_failure() {
        let mut env = Environment::new_fake_for_test();
        let name = ns_string::from_rust_string(&mut env, "analytics.example".to_string());
        let host = CFHostCreateWithName(&mut env, kCFAllocatorDefault, name);

        assert_ne!(host, nil);
        assert_eq!(env.objc.borrow::<CFHostHostObject>(host).name, name);

        let error = env.mem.alloc_and_write(CFStreamError {
            domain: 0,
            error: 0,
        });
        assert!(!CFHostStartInfoResolution(&mut env, host, 0, error));
        assert_eq!(
            env.mem.read(error),
            CFStreamError {
                domain: K_CF_STREAM_ERROR_DOMAIN_POSIX,
                error: ENETUNREACH,
            }
        );

        let resolved = env.mem.alloc_and_write(1u8);
        assert_eq!(CFHostGetAddressing(&mut env, host, resolved), nil);
        assert_eq!(env.mem.read(resolved), 0);
    }

    #[test]
    fn socket_stream_pair_creates_owned_objects_before_offline_open() {
        let mut env = Environment::new_fake_for_test();
        let name = ns_string::from_rust_string(&mut env, "analytics.example".to_string());
        let host = CFHostCreateWithName(&mut env, kCFAllocatorDefault, name);
        let read_stream = env.mem.alloc_and_write(host);
        let write_stream = env.mem.alloc_and_write(host);

        CFStreamCreatePairWithSocketToCFHost(
            &mut env,
            kCFAllocatorDefault,
            host,
            80,
            read_stream,
            write_stream,
        );

        let read = env.mem.read(read_stream);
        let write = env.mem.read(write_stream);
        assert_ne!(read, nil);
        assert_ne!(write, nil);
        assert_ne!(read, write);
        assert!(crate::frameworks::core_foundation::cf_socket_stream::is_socket(&env, read));
        crate::objc::release(&mut env, read);
        crate::objc::release(&mut env, write);

        // Either output pointer may be omitted independently.
        CFStreamCreatePairWithSocketToCFHost(
            &mut env,
            kCFAllocatorDefault,
            host,
            80,
            MutPtr::null(),
            MutPtr::null(),
        );
    }
}
