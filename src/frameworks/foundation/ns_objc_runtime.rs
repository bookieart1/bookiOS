//! Things from `NSObjCRuntime.h`.

use super::ns_string;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, SafeRead};
use crate::objc::{id, nil, Class, SEL};
use crate::Environment;

/// Prefix of the 32-bit Objective-C 2 runtime's protocol_t structure. The
/// compiler-emitted protocol object's mangled name immediately follows isa.
#[repr(C, packed)]
struct GuestProtocol {
    _isa: Class,
    mangled_name: ConstPtr<u8>,
}
unsafe impl SafeRead for GuestProtocol {}

fn NSStringFromSelector(env: &mut Environment, selector: SEL) -> id {
    // TODO: caching?
    let string = selector.as_str(&env.mem).to_string();
    ns_string::from_rust_string(env, string)
}

fn NSSelectorFromString(env: &mut Environment, string: id) -> SEL {
    // TODO: avoid copy?
    let string = ns_string::to_rust_string(env, string);
    env.objc.register_host_selector(string.into(), &mut env.mem)
}

pub fn NSStringFromClass(env: &mut Environment, class: Class) -> id {
    if class.is_null() {
        return nil;
    }
    // TODO: caching?
    let string = env.objc.get_class_name(class).to_string();
    ns_string::from_rust_string(env, string)
}

fn NSStringFromProtocol(env: &mut Environment, protocol: ConstPtr<GuestProtocol>) -> id {
    if protocol.is_null() {
        return nil;
    }
    let protocol = env.mem.read(protocol);
    let name = env
        .mem
        .cstr_at_utf8(protocol.mangled_name)
        .expect("Objective-C protocol name is not valid UTF-8")
        .to_owned();
    ns_string::from_rust_string(env, name)
}

fn NSClassFromString(env: &mut Environment, string: id) -> Class {
    if string == nil {
        env.diagnostic_campaign_event("class-lookup", "NSClassFromString:nil", [0; 8]);
        return nil;
    }
    // TODO: avoid copy?
    let requested = ns_string::to_rust_string(env, string);
    let previously_registered = env
        .objc
        .all_classes()
        .any(|(name, _)| name == &requested);
    let result = env.objc.lookup_class(&requested, &mut env.mem).unwrap_or(nil);
    let detail = format!(
        "NSClassFromString request={requested:?} registered-before={previously_registered} result={:#010x}",
        result.to_bits()
    );
    env.diagnostic_campaign_event(
        "class-lookup",
        &detail,
        [
            string.to_bits(),
            result.to_bits(),
            u32::from(previously_registered),
            0,
            0,
            0,
            0,
            0,
        ],
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;

    #[test]
    fn class_from_string_returns_known_class_or_nil() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let known = ns_string::from_rust_string(&mut env, "NSObject".to_owned());
        let missing = ns_string::from_rust_string(
            &mut env,
            "DefinitelyNotAnIPhoneOSClass".to_owned(),
        );
        assert!(!NSClassFromString(&mut env, known).is_null());
        assert!(NSClassFromString(&mut env, missing).is_null());
        assert!(NSClassFromString(&mut env, nil).is_null());
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSStringFromSelector(_)),
    export_c_func!(NSSelectorFromString(_)),
    export_c_func!(NSClassFromString(_)),
    export_c_func!(NSStringFromClass(_)),
    export_c_func!(NSStringFromProtocol(_)),
];
