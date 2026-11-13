/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `netdb.h`

use crate::dyld::{ConstantExports, FunctionExports, HostConstant};
use crate::export_c_func;
use crate::libc::sys::socket::{sockaddr, AF_INET, SOCK_DGRAM, SOCK_STREAM};
use crate::mem::{guest_size_of, ConstPtr, MutPtr, Ptr, SafeRead};
use crate::Environment;
use std::collections::HashMap;
use std::io::Write;
use std::net::Ipv4Addr;

const AI_PASSIVE: i32 = 0x1;

pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;

const EAI_FAIL: i32 = 4;
const HOST_NOT_FOUND: i32 = 1;
const TRY_AGAIN: i32 = 2;
const NO_RECOVERY: i32 = 3;
const NO_DATA: i32 = 4;

#[allow(non_camel_case_types)]
pub type socklen_t = u32;

/// Darwin's 32-bit `struct hostent`. `gethostbyname()` returns a pointer to
/// process-owned static storage whose contents may be replaced by a later
/// call.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[allow(non_camel_case_types)]
struct hostent {
    h_name: MutPtr<u8>,
    h_aliases: MutPtr<MutPtr<u8>>,
    h_addrtype: i32,
    h_length: i32,
    h_addr_list: MutPtr<MutPtr<u8>>,
}
unsafe impl SafeRead for hostent {}

#[derive(Default)]
pub struct State {
    hostent: Option<MutPtr<hostent>>,
    name: Option<MutPtr<u8>>,
    aliases: Option<MutPtr<MutPtr<u8>>>,
    address: Option<MutPtr<u8>>,
    address_list: Option<MutPtr<MutPtr<u8>>>,
    h_errno: Option<MutPtr<i32>>,
    error_strings: HashMap<i32, MutPtr<u8>>,
    gai_error_strings: HashMap<i32, MutPtr<u8>>,
}

fn h_errno_ptr(env: &mut Environment) -> MutPtr<i32> {
    if env.libc_state.netdb.h_errno.is_none() {
        env.libc_state.netdb.h_errno = Some(env.mem.alloc_and_write(0));
    }
    env.libc_state.netdb.h_errno.unwrap()
}

fn set_h_errno(env: &mut Environment, value: i32) {
    let pointer = h_errno_ptr(env);
    env.mem.write(pointer, value);
}

fn __h_errno(env: &mut Environment) -> MutPtr<i32> {
    h_errno_ptr(env)
}

fn h_error_text(error: i32) -> &'static str {
    match error {
        0 => "Resolver Error 0 (no error)",
        HOST_NOT_FOUND => "Unknown host",
        TRY_AGAIN => "Host name lookup failure",
        NO_RECOVERY => "Unknown server error",
        NO_DATA => "No address associated with name",
        _ => "Unknown resolver error",
    }
}

fn hstrerror(env: &mut Environment, error: i32) -> MutPtr<u8> {
    if let Some(&pointer) = env.libc_state.netdb.error_strings.get(&error) {
        return pointer;
    }
    let pointer = env
        .mem
        .alloc_and_write_cstr(h_error_text(error).as_bytes());
    env.libc_state.netdb.error_strings.insert(error, pointer);
    pointer
}

fn gai_error_text(error: i32) -> &'static str {
    // Darwin's historical EAI values and strings come from the BSD
    // getaddrinfo contract. Keep this namespace separate from h_errno: the
    // numeric values overlap but describe different classes of error.
    match error {
        0 => "Success",
        1 => "Address family for hostname not supported",
        2 => "Name could not be resolved at this time",
        3 => "Flags parameter had an invalid value",
        EAI_FAIL => "Non-recoverable failure in name resolution",
        5 => "Address family not recognized",
        6 => "Memory allocation failure",
        7 => "No address associated with hostname",
        8 => "Name does not resolve",
        9 => "Service was not recognized for socket type",
        10 => "Intended socket type was not recognized",
        11 => "System error returned in errno",
        12 => "Invalid value for hints",
        13 => "Resolved protocol is unknown",
        14 => "Argument buffer overflow",
        _ => "Unknown error",
    }
}

fn gai_strerror(env: &mut Environment, error: i32) -> MutPtr<u8> {
    if let Some(&pointer) = env.libc_state.netdb.gai_error_strings.get(&error) {
        return pointer;
    }
    let pointer = env
        .mem
        .alloc_and_write_cstr(gai_error_text(error).as_bytes());
    env.libc_state
        .netdb
        .gai_error_strings
        .insert(error, pointer);
    pointer
}

fn herror(env: &mut Environment, prefix: ConstPtr<u8>) {
    let error_pointer = h_errno_ptr(env);
    let error = env.mem.read(error_pointer);
    let mut message = String::new();
    if !prefix.is_null() {
        let prefix = env.mem.cstr_at_utf8(prefix).unwrap_or("<invalid>");
        if !prefix.is_empty() {
            message.push_str(prefix);
            message.push_str(": ");
        }
    }
    message.push_str(h_error_text(error));
    message.push('\n');
    let _ = std::io::stderr().write_all(message.as_bytes());
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[allow(non_camel_case_types)]
pub struct addrinfo {
    ai_flags: i32,
    ai_family: i32,
    ai_socktype: i32,
    ai_protocol: i32,
    ai_addrlen: socklen_t,
    ai_canonname: MutPtr<u8>,
    ai_addr: MutPtr<sockaddr>,
    ai_next: MutPtr<addrinfo>,
}
unsafe impl SafeRead for addrinfo {}

fn getaddrinfo(
    env: &mut Environment,
    node_name: MutPtr<u8>,
    serv_name: MutPtr<u8>,
    hints: ConstPtr<addrinfo>,
    res: MutPtr<MutPtr<addrinfo>>,
) -> i32 {
    if !env.options.network_access {
        log_dbg!(
            "Network access is disabled, getaddrinfo({:?}, {:?}, {:?}, {:?}) -> EAI_FAIL",
            node_name,
            serv_name,
            hints,
            res
        );
        return EAI_FAIL;
    }

    assert!(node_name.is_null()); // TODO

    let hint = env.mem.read(hints);
    let ai_flags = hint.ai_flags;
    assert_eq!(ai_flags, AI_PASSIVE);
    let ai_family = hint.ai_family;
    assert_eq!(ai_family, AF_INET);
    assert!(hint.ai_socktype == SOCK_STREAM || hint.ai_socktype == SOCK_DGRAM);
    assert!(
        hint.ai_protocol == IPPROTO_TCP || hint.ai_protocol == IPPROTO_UDP || hint.ai_protocol == 0
    );
    let ai_addrlen = hint.ai_addrlen;
    assert_eq!(ai_addrlen, 0);
    assert!(hint.ai_canonname.is_null());
    assert!(hint.ai_addr.is_null());
    assert!(hint.ai_next.is_null());

    let mut addr_info = hint;
    let port: u16 = env.mem.cstr_at_utf8(serv_name).unwrap().parse().unwrap();
    log_dbg!("getaddrinfo: port {}", port);
    let addr = sockaddr::from_ipv4_parts([0; 4], port);

    let tmp_addr = env.mem.alloc_and_write(addr);
    addr_info.ai_addr = tmp_addr;
    addr_info.ai_addrlen = guest_size_of::<sockaddr>();

    let tmp_addr_info = env.mem.alloc_and_write(addr_info);
    env.mem.write(res, tmp_addr_info);

    0 // Success
}

fn freeaddrinfo(env: &mut Environment, addrinfo: MutPtr<addrinfo>) {
    let addrinfo_val = env.mem.read(addrinfo);
    assert!(addrinfo_val.ai_next.is_null()); // TODO
    let ai_addrlen = addrinfo_val.ai_addrlen;
    assert_eq!(ai_addrlen, guest_size_of::<sockaddr>());
    env.mem.free(addrinfo_val.ai_addr.cast());
    env.mem.free(addrinfo.cast());
}

fn gethostbyname(env: &mut Environment, name: ConstPtr<u8>) -> MutPtr<hostent> {
    let name_text = env.mem.cstr_at_utf8(name).unwrap().to_owned();
    let Ok(address) = name_text.parse::<Ipv4Addr>() else {
        log!(
            "TODO: gethostbyname({:?} {:?}) for a hostname requires DNS => NULL",
            name,
            name_text
        );
        set_h_errno(env, HOST_NOT_FOUND);
        return Ptr::null();
    };

    set_h_errno(env, 0);

    // Numeric IPv4 literals are resolved locally by historical BSD/Darwin
    // gethostbyname(). This does not perform DNS and does not grant network
    // access: a later connect/send still follows the emulator's offline rules.
    if env.libc_state.netdb.name.is_none() {
        env.libc_state.netdb.name = Some(env.mem.alloc(16).cast());
    }
    let name_buffer = env.libc_state.netdb.name.unwrap();
    let name_bytes = name_text.as_bytes();
    assert!(name_bytes.len() < 16);
    env.mem
        .bytes_at_mut(name_buffer, name_bytes.len().try_into().unwrap())
        .copy_from_slice(name_bytes);
    env.mem.write(name_buffer + name_bytes.len() as u32, b'\0');

    if env.libc_state.netdb.address.is_none() {
        env.libc_state.netdb.address = Some(env.mem.alloc(4).cast());
    }
    let address_buffer = env.libc_state.netdb.address.unwrap();
    env.mem
        .bytes_at_mut(address_buffer, 4)
        .copy_from_slice(&address.octets());

    if env.libc_state.netdb.aliases.is_none() {
        let aliases = env.mem.alloc(guest_size_of::<MutPtr<u8>>()).cast();
        env.mem.write(aliases, MutPtr::<u8>::null());
        env.libc_state.netdb.aliases = Some(aliases);
    }
    let aliases = env.libc_state.netdb.aliases.unwrap();
    if env.libc_state.netdb.address_list.is_none() {
        let address_list = env
            .mem
            .alloc(guest_size_of::<MutPtr<u8>>() * 2)
            .cast();
        env.mem.write(address_list, address_buffer);
        env.mem
            .write(address_list + 1, MutPtr::<u8>::null());
        env.libc_state.netdb.address_list = Some(address_list);
    }
    let address_list = env.libc_state.netdb.address_list.unwrap();
    // The address buffer itself is stable, but rewrite the first pointer in
    // case this state was initialized by a future alternate path.
    env.mem.write(address_list, address_buffer);

    let value = hostent {
        h_name: name_buffer,
        h_aliases: aliases,
        h_addrtype: AF_INET,
        h_length: 4,
        h_addr_list: address_list,
    };
    if env.libc_state.netdb.hostent.is_none() {
        env.libc_state.netdb.hostent = Some(env.mem.alloc_and_write(value));
    }
    let result = env.libc_state.netdb.hostent.unwrap();
    env.mem.write(result, value);
    log_dbg!("gethostbyname({:?}) => {:?} ({})", name_text, result, address);
    result
}

fn gethostent(env: &mut Environment) -> MutPtr<hostent> {
    log!("TODO: gethostent() => NULL");
    set_h_errno(env, HOST_NOT_FOUND);
    Ptr::null()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(getaddrinfo(_, _, _, _)),
    export_c_func!(freeaddrinfo(_)),
    export_c_func!(gethostbyname(_)),
    export_c_func!(gethostent()),
    export_c_func!(__h_errno()),
    export_c_func!(hstrerror(_)),
    export_c_func!(gai_strerror(_)),
    export_c_func!(herror(_)),
];

pub const CONSTANTS: ConstantExports = &[(
    "_h_errno",
    HostConstant::Custom(|env| h_errno_ptr(env).cast().cast_const()),
)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gethostbyname_resolves_numeric_ipv4_without_network_access() {
        let mut env = Environment::new_fake_for_test();
        assert!(!env.options.network_access);
        let name = env.mem.alloc_and_write_cstr(b"218.145.70.37");
        let result = gethostbyname(&mut env, name.cast_const());
        assert!(!result.is_null());
        let host = env.mem.read(result);
        let address_type = host.h_addrtype;
        let address_length = host.h_length;
        let host_name = host.h_name;
        let host_address_list = host.h_addr_list;
        let host_aliases = host.h_aliases;
        assert_eq!(address_type, AF_INET);
        assert_eq!(address_length, 4);
        assert_eq!(
            env.mem.cstr_at_utf8(host_name.cast_const()),
            Ok("218.145.70.37")
        );
        let first_address = env.mem.read(host_address_list);
        assert_eq!(
            env.mem.bytes_at(first_address.cast_const(), 4),
            &[218, 145, 70, 37]
        );
        assert!(env.mem.read(host_address_list + 1).is_null());
        assert!(env.mem.read(host_aliases).is_null());
    }

    #[test]
    fn gethostbyname_does_not_fabricate_dns() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"example.invalid");
        assert!(gethostbyname(&mut env, name.cast_const()).is_null());
        let error_pointer = __h_errno(&mut env);
        assert_eq!(env.mem.read(error_pointer), HOST_NOT_FOUND);
        let error_text = hstrerror(&mut env, HOST_NOT_FOUND);
        assert_eq!(
            env.mem.cstr_at_utf8(error_text.cast_const()),
            Ok("Unknown host")
        );
    }

    #[test]
    fn development_256_gai_strerror_uses_darwin_eai_namespace_and_stable_storage() {
        let mut env = Environment::new_fake_for_test();

        let fail = gai_strerror(&mut env, EAI_FAIL);
        assert_eq!(
            env.mem.cstr_at_utf8(fail.cast_const()),
            Ok("Non-recoverable failure in name resolution")
        );
        assert_eq!(gai_strerror(&mut env, EAI_FAIL), fail);

        // h_errno value 4 has a different meaning and must not alias EAI_FAIL.
        let no_data = hstrerror(&mut env, NO_DATA);
        assert_ne!(fail, no_data);
        assert_eq!(
            env.mem.cstr_at_utf8(no_data.cast_const()),
            Ok("No address associated with name")
        );

        let unknown = gai_strerror(&mut env, -1);
        assert_eq!(
            env.mem.cstr_at_utf8(unknown.cast_const()),
            Ok("Unknown error")
        );
    }
}
