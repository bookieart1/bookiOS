/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFHTTPMessage`

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::cf_data::CFDataRef;
use crate::frameworks::core_foundation::cf_string::CFStringRef;
use crate::frameworks::core_foundation::cf_url::CFURLRef;
use crate::frameworks::core_foundation::{CFOptionFlags, CFTypeRef};
use crate::frameworks::foundation::ns_string;
use crate::mem::{MutVoidPtr, Ptr};
use crate::objc::msg;
use crate::Environment;

// Note: on iOS SDK side this type is defined as a pointer to an opaque struct
type CFHTTPMessageRef = CFTypeRef;
type CFReadStreamRef = CFTypeRef;

fn CFHTTPMessageCreateRequest(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    request_method: CFStringRef,
    url: CFURLRef,
    http_version: CFStringRef,
) -> CFHTTPMessageRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let url_desc = msg![env; url description];
    log!(
        "TODO: CFHTTPMessageCreateRequest({}, '{}', {}) -> NULL",
        ns_string::to_rust_string(env, request_method),
        ns_string::to_rust_string(env, url_desc),
        ns_string::to_rust_string(env, http_version),
    );
    Ptr::null()
}

fn CFHTTPMessageSetHeaderFieldValue(
    env: &mut Environment,
    message: CFHTTPMessageRef,
    header_field: CFStringRef,
    value: CFStringRef,
) {
    if !message.is_null() {
        todo!(
            "CFHTTPMessageSetHeaderFieldValue({:?}, {}, {})",
            message,
            ns_string::to_rust_string(env, header_field),
            ns_string::to_rust_string(env, value)
        );
    }
}

fn CFHTTPMessageSetBody(_env: &mut Environment, message: CFHTTPMessageRef, body_data: CFDataRef) {
    if !message.is_null() {
        todo!("CFHTTPMessageSetBody({:?}, {:?})", message, body_data);
    }
}

fn CFReadStreamCreateForHTTPRequest(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    message: CFHTTPMessageRef,
) -> CFReadStreamRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    if !message.is_null() {
        todo!(
            "CFReadStreamCreateForHTTPRequest({:?}, {:?})",
            allocator,
            message,
        );
    }
    Ptr::null()
}

fn CFReadStreamSetProperty(
    env: &mut Environment,
    stream: CFReadStreamRef,
    property_name: CFStringRef,
    property_value: CFTypeRef,
) -> bool {
    if crate::frameworks::core_foundation::cf_socket_stream::is_socket(env, stream) {
        return false; // Offline transport has no native socket or HTTP properties.
    }
    if !stream.is_null() {
        todo!(
            "CFReadStreamSetProperty({:?}, {}, {:?})",
            stream,
            ns_string::to_rust_string(env, property_name),
            property_value
        );
    }
    false
}
fn CFReadStreamSetClient(
    env: &mut Environment, stream: CFReadStreamRef, events: CFOptionFlags,
    callback: GuestFunction, context: MutVoidPtr,
) -> bool {
    crate::frameworks::core_foundation::cf_socket_stream::set_client(env, stream, events, callback, context)
}

fn CFReadStreamCopyProperty(
    env: &mut Environment,
    stream: CFReadStreamRef,
    property_name: CFStringRef,
) -> CFTypeRef {
    if crate::frameworks::core_foundation::cf_socket_stream::is_socket(env, stream) {
        return Ptr::null(); // Offline transport has no native socket or HTTP properties.
    }
    if !stream.is_null() {
        todo!(
            "CFReadStreamCopyProperty({:?}, {})",
            stream,
            ns_string::to_rust_string(env, property_name),
        );
    }
    Ptr::null()
}

const kCFHTTPVersion1_0: &str = "kCFHTTPVersion1_0";
const kCFHTTPVersion1_1: &str = "kCFHTTPVersion1_1";

// TODO: move to correct location
const kCFStreamPropertySSLSettings: &str = "kCFStreamPropertySSLSettings";
const kCFStreamPropertyHTTPResponseHeader: &str = "kCFStreamPropertyHTTPResponseHeader";

pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFNetworkProxiesHTTPEnable",
        HostConstant::NSString("HTTPEnable"),
    ),
    (
        "_kCFNetworkProxiesHTTPProxy",
        HostConstant::NSString("HTTPProxy"),
    ),
    (
        "_kCFNetworkProxiesHTTPPort",
        HostConstant::NSString("HTTPPort"),
    ),
    (
        "_kCFHTTPVersion1_0",
        HostConstant::NSString(kCFHTTPVersion1_0),
    ),
    (
        "_kCFHTTPVersion1_1",
        HostConstant::NSString(kCFHTTPVersion1_1),
    ),
    (
        "_kCFStreamPropertySSLSettings",
        HostConstant::NSString(kCFStreamPropertySSLSettings),
    ),
    (
        "_kCFStreamPropertyHTTPResponseHeader",
        HostConstant::NSString(kCFStreamPropertyHTTPResponseHeader),
    ),
    // TODO: move to correct location
    (
        "_kCFStreamSocketSecurityLevelNegotiatedSSL",
        HostConstant::NSString("kCFStreamSocketSecurityLevelNegotiatedSSL"),
    ),
    (
        "_kCFStreamSSLValidatesCertificateChain",
        HostConstant::NSString("kCFStreamSSLValidatesCertificateChain"),
    ),
    (
        "_kCFStreamSSLPeerName",
        HostConstant::NSString("kCFStreamSSLPeerName"),
    ),
    (
        "_kCFStreamSSLLevel",
        HostConstant::NSString("kCFStreamSSLLevel"),
    ),
    (
        "_kCFStreamSSLAllowsExpiredRoots",
        HostConstant::NSString("kCFStreamSSLAllowsExpiredRoots"),
    ),
    (
        "_kCFStreamSSLAllowsExpiredCertificates",
        HostConstant::NSString("_kCFStreamSSLAllowsExpiredCertificates"),
    ),
    (
        "_kCFStreamSSLAllowsAnyRoot",
        HostConstant::NSString("kCFStreamSSLAllowsAnyRoot"),
    ),
];

#[cfg(test)]
mod proxy_constant_tests {
    use super::*;

    #[test]
    fn http_proxy_dictionary_keys_are_exported() {
        let names: Vec<_> = CONSTANTS.iter().map(|(name, _)| *name).collect();
        for required in [
            "_kCFNetworkProxiesHTTPEnable",
            "_kCFNetworkProxiesHTTPProxy",
            "_kCFNetworkProxiesHTTPPort",
        ] {
            assert!(names.contains(&required));
        }
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFHTTPMessageCreateRequest(_, _, _, _)),
    export_c_func!(CFHTTPMessageSetHeaderFieldValue(_, _, _)),
    export_c_func!(CFHTTPMessageSetBody(_, _)),
    export_c_func!(CFReadStreamCreateForHTTPRequest(_, _)),
    // TODO: move to correct location
    export_c_func!(CFReadStreamSetProperty(_, _, _)),
    export_c_func!(CFReadStreamSetClient(_, _, _, _)),
    export_c_func!(CFReadStreamCopyProperty(_, _)),
];
