/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal historical GraphicsServices capability-query surface.
//!
//! Applications used these private functions to probe optional device
//! features. iDared currently exposes none of those private capabilities, so
//! the truthful result is unavailable (`false`/`NULL`), not a missing dylib or
//! an unresolved function pointer.

use crate::dyld::{export_c_func, FunctionExports, HostDylib};
use crate::frameworks::core_foundation::{cf_string::CFStringRef, CFTypeRef};
use crate::mem::Ptr;
use crate::Environment;

fn GSSystemHasCapability(env: &mut Environment, capability: CFStringRef) -> bool {
    env.diagnostic_campaign_event(
        "graphics-services",
        "GSSystemHasCapability capability,result",
        [capability.to_bits(), 0, 0, 0, 0, 0, 0, 0],
    );
    if !capability.is_null() {
        log_dbg!(
            "GSSystemHasCapability({:?} '{}') -> false",
            capability,
            crate::frameworks::foundation::ns_string::to_rust_string(env, capability),
        );
    }
    false
}

fn GSSystemCopyCapability(env: &mut Environment, capability: CFStringRef) -> CFTypeRef {
    env.diagnostic_campaign_event(
        "graphics-services",
        "GSSystemCopyCapability capability,result",
        [capability.to_bits(), 0, 0, 0, 0, 0, 0, 0],
    );
    if !capability.is_null() {
        log_dbg!(
            "GSSystemCopyCapability({:?} '{}') -> NULL",
            capability,
            crate::frameworks::foundation::ns_string::to_rust_string(env, capability),
        );
    }
    Ptr::null()
}

const FUNCTIONS: FunctionExports = &[
    export_c_func!(GSSystemHasCapability(_)),
    export_c_func!(GSSystemCopyCapability(_)),
];

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/PrivateFrameworks/GraphicsServices.framework/GraphicsServices",
    aliases: &[],
    class_exports: &[],
    constant_exports: &[],
    function_exports: &[FUNCTIONS],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;

    #[test]
    fn dev_248_unavailable_capability_results_are_typed_and_coherent() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let capability = ns_string::get_static_str(&mut env, "telephony");
        assert!(!GSSystemHasCapability(&mut env, capability));
        assert!(GSSystemCopyCapability(&mut env, capability).is_null());
    }
}
