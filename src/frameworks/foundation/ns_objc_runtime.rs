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
        return nil;
    }
    // TODO: avoid copy?
    let string = ns_string::to_rust_string(env, string);

    // While this method is supposed to return nil if the class is not found,
    // touchHLE is missing many classes that apps might expect to be present,
    // so this could be troublesome. So, let's use get_known_class, which panics
    // when it can't find the class. We could except certain classes or apps if
    // we need to.
    env.objc.get_known_class(&string, &mut env.mem)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSStringFromSelector(_)),
    export_c_func!(NSSelectorFromString(_)),
    export_c_func!(NSClassFromString(_)),
    export_c_func!(NSStringFromClass(_)),
    export_c_func!(NSStringFromProtocol(_)),
];
