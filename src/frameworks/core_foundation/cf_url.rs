/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFURL`.
//!
//! This is toll-free bridged to `NSURL` in Apple's implementation. Here it is
//! the same type.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::CFIndex;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::cf_string::{
    kCFStringEncodingASCII, kCFStringEncodingUTF8, CFStringConvertEncodingToNSStringEncoding,
    CFStringEncoding, CFStringRef,
};
use crate::frameworks::foundation::ns_file_manager::NSFileSize;
use crate::frameworks::foundation::ns_string::{
    from_rust_string, get_static_str, to_rust_string, NSASCIIStringEncoding, NSUTF8StringEncoding,
};
use crate::frameworks::foundation::NSUInteger;
use crate::mem::{ConstPtr, MutPtr, Ptr};
use crate::objc::{id, msg, msg_class, nil, release, retain};
use crate::Environment;

pub type CFURLRef = super::CFTypeRef;

// Historical CFURL access-property key, available from iPhone OS 2.0. Its
// value is the key's public name (see CFURLAccess.c's CONST_STRING_DECL).
pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFURLFileExists",
        HostConstant::NSString("kCFURLFileExists"),
    ),
    (
        "_kCFURLFileLength",
        HostConstant::NSString("kCFURLFileLength"),
    ),
];

type CFURLPathStyle = CFIndex;
const kCFURLPOSIXPathStyle: CFURLPathStyle = 0;
#[allow(dead_code)]
const kCFURLHFSPathStyle: CFURLPathStyle = 1;
#[allow(dead_code)]
const kCFURLWindowsPathStyle: CFURLPathStyle = 2;

pub fn CFURLGetFileSystemRepresentation(
    env: &mut Environment,
    url: CFURLRef,
    resolve_against_base: bool,
    buffer: MutPtr<u8>,
    buffer_size: CFIndex,
) -> bool {
    if resolve_against_base {
        // this function usually called to resolve resources from the main
        // bundle
        // thus, the url should already be an absolute path name
        // TODO: use absoluteURL instead once implemented
        let path = msg![env; url path];
        // TODO: avoid copy
        assert!(to_rust_string(env, path).starts_with('/'));
    }
    let buffer_size: NSUInteger = buffer_size.try_into().unwrap();

    msg![env; url getFileSystemRepresentation:buffer
                                    maxLength:buffer_size]
}

/// Historical CFURL resource-property access used by pre-iOS-5 games.  This
/// Supported legacy keys return their documented value types; unsupported
/// keys fail instead of returning a fabricated value.
fn CFURLCreatePropertyFromResource(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    url: CFURLRef,
    property: CFStringRef,
    error_code: MutPtr<i32>,
) -> super::CFTypeRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    if !error_code.is_null() {
        env.mem.write(error_code, 0);
    }
    if url.is_null() || property.is_null() {
        if !error_code.is_null() {
            env.mem.write(error_code, -10); // kCFURLUnknownError
        }
        return Ptr::null();
    }

    let file_exists_key = get_static_str(env, "kCFURLFileExists");
    let file_length_key = get_static_str(env, "kCFURLFileLength");
    let requests_file_exists: bool = msg![env; property isEqual:file_exists_key];
    let requests_file_length: bool = msg![env; property isEqual:file_length_key];
    if !requests_file_exists && !requests_file_length {
        log!(
            "Warning: CFURLCreatePropertyFromResource unsupported property {:?}",
            property
        );
        if !error_code.is_null() {
            env.mem.write(error_code, -12); // kCFURLPropertyKeyUnavailableError
        }
        return Ptr::null();
    }

    let path: id = msg![env; url path];
    let manager: id = msg_class![env; NSFileManager defaultManager];
    if requests_file_exists {
        let exists: bool = msg![env; manager fileExistsAtPath:path];
        let value: id = msg_class![env; NSNumber numberWithBool:exists];
        retain(env, value);
        return value;
    }

    let no_error: MutPtr<id> = Ptr::null();
    let attributes: id = msg![env; manager attributesOfItemAtPath:path error:no_error];
    if attributes == nil {
        if !error_code.is_null() {
            env.mem.write(error_code, -4); // kCFURLResourceNotFoundError
        }
        return Ptr::null();
    }
    let size_key = get_static_str(env, NSFileSize);
    let value: id = msg![env; attributes objectForKey:size_key];
    if value == nil {
        if !error_code.is_null() {
            env.mem.write(error_code, -12);
        }
        return Ptr::null();
    }
    retain(env, value);
    value
}

pub fn CFURLCreateFromFileSystemRepresentation(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    buffer: ConstPtr<u8>,
    buffer_size: CFIndex,
    is_directory: bool,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented

    let buffer_size: NSUInteger = buffer_size.try_into().unwrap();

    let string: id = msg_class![env; NSString alloc];
    let string: id = msg![env; string initWithBytes:buffer
                                             length:buffer_size
                                           encoding:NSUTF8StringEncoding];

    let url: id = msg_class![env; NSURL alloc];
    let res = msg![env; url initFileURLWithPath:string isDirectory:is_directory];
    release(env, string);
    res
}

fn CFURLCreateWithBytes(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    url_bytes: ConstPtr<u8>,
    length: CFIndex,
    encoding: CFStringEncoding,
    base_url: CFURLRef,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    if !matches!(encoding, kCFStringEncodingASCII | kCFStringEncodingUTF8) {
        return Ptr::null();
    }
    assert!(base_url.is_null()); // TODO

    // TODO: interpret percent escape sequences using encoding as well
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let length: NSUInteger = length.try_into().unwrap();

    let string: id = msg_class![env; NSString alloc];
    let string: id = msg![env; string initWithBytes:url_bytes
                                             length:length
                                           encoding:encoding];

    if string == nil {
        return Ptr::null();
    }

    assert!(!to_rust_string(env, string).contains("://")); // TODO

    // Assume file URL case here
    let url: id = msg_class![env; NSURL alloc];
    let res = msg![env; url initFileURLWithPath:string];
    release(env, string);
    res
}

fn CFURLCreateWithFileSystemPath(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    file_path: CFStringRef,
    style: CFURLPathStyle,
    is_directory: bool,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert_eq!(style, kCFURLPOSIXPathStyle);
    let url: id = msg_class![env; NSURL alloc];
    msg![env; url initFileURLWithPath:file_path isDirectory:is_directory]
}

fn CFURLCreateWithString(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    url_string: CFStringRef,
    base_url: CFURLRef,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert!(base_url.is_null()); // TODO
    let url: id = msg_class![env; NSURL alloc];
    msg![env; url initWithString:url_string]
}

fn CFURLCreateStringByAddingPercentEscapes(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    original_string: CFStringRef,
    characters_to_leave_unescaped: CFStringRef,
    legal_url_characters_to_be_escaped: CFStringRef,
    encoding: CFStringEncoding,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    let ns_encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    assert!(ns_encoding == NSUTF8StringEncoding || ns_encoding == NSASCIIStringEncoding);

    let original = to_rust_string(env, original_string);
    let leave_unescaped = if characters_to_leave_unescaped.is_null() {
        String::new()
    } else {
        to_rust_string(env, characters_to_leave_unescaped).into_owned()
    };
    let force_escaped = if legal_url_characters_to_be_escaped.is_null() {
        String::new()
    } else {
        to_rust_string(env, legal_url_characters_to_be_escaped).into_owned()
    };

    // RFC 2396's unreserved and reserved URL characters. Existing percent
    // escapes are preserved. Callers can explicitly add or remove characters
    // from this set through the two optional arguments above.
    const URL_SAFE: &[u8] = b"-_.!~*'();/?:@&=+$,#%";
    let mut escaped = String::with_capacity(original.len());
    for &byte in original.as_bytes() {
        let ch = byte as char;
        let must_escape = byte.is_ascii() && force_escaped.contains(ch);
        let is_safe = byte.is_ascii_alphanumeric()
            || URL_SAFE.contains(&byte)
            || (byte.is_ascii() && leave_unescaped.contains(ch));
        if is_safe && !must_escape {
            escaped.push(ch);
        } else {
            use std::fmt::Write;
            write!(&mut escaped, "%{byte:02X}").unwrap();
        }
    }

    // The Create rule transfers ownership of this retained NSString to the
    // caller, which is exactly what from_rust_string returns.
    from_rust_string(env, escaped)
}

pub fn CFURLCopyPathExtension(env: &mut Environment, url: CFURLRef) -> CFStringRef {
    let path = msg![env; url path];
    let ext = msg![env; path pathExtension];
    msg![env; ext copy]
}

fn CFURLCopyPath(env: &mut Environment, url: CFURLRef) -> CFStringRef {
    if url.is_null() {
        return url;
    }
    let path: id = msg![env; url path];
    if path.is_null() {
        path
    } else {
        msg![env; path copy]
    }
}

fn CFURLCopyFileSystemPath(
    env: &mut Environment,
    url: CFURLRef,
    style: CFURLPathStyle,
) -> CFStringRef {
    assert_eq!(style, kCFURLPOSIXPathStyle);
    let path: CFStringRef = msg![env; url path];
    msg![env; path copy]
}

fn CFURLCreateCopyAppendingPathComponent(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    url: CFURLRef,
    path_component: CFStringRef,
    is_directory: bool,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let new_url =
        msg![env; url URLByAppendingPathComponent:path_component isDirectory:is_directory];
    msg![env; new_url copy]
}

fn CFURLCreateCopyDeletingLastPathComponent(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    url: CFURLRef,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let new_url = msg![env; url URLByDeletingLastPathComponent];
    msg![env; new_url copy]
}

fn CFURLHasDirectoryPath(env: &mut Environment, url: CFURLRef) -> bool {
    assert!(!url.is_null());

    let path = msg![env; url path];
    if msg![env; path isEqual:(get_static_str(env, "//"))] {
        // Special case
        return false;
    }
    // Note: cannot use `lastPathComponent` here!
    let components: id = msg![env; path pathComponents];
    let count: NSUInteger = msg![env; components count];
    if count == 0 {
        return false;
    }
    let last: id = msg![env; components objectAtIndex:(count - 1)];
    msg![env; last isEqual:(get_static_str(env, "/"))]
        || msg![env; last isEqual:(get_static_str(env, "."))]
        || msg![env; last isEqual:(get_static_str(env, ".."))]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFURLGetFileSystemRepresentation(_, _, _, _)),
    export_c_func!(CFURLCreatePropertyFromResource(_, _, _, _)),
    export_c_func!(CFURLCreateFromFileSystemRepresentation(_, _, _, _)),
    export_c_func!(CFURLCreateWithBytes(_, _, _, _, _)),
    export_c_func!(CFURLCreateWithFileSystemPath(_, _, _, _)),
    export_c_func!(CFURLCreateWithString(_, _, _)),
    export_c_func!(CFURLCreateStringByAddingPercentEscapes(_, _, _, _, _)),
    export_c_func!(CFURLCopyPathExtension(_)),
    export_c_func!(CFURLCopyPath(_)),
    export_c_func!(CFURLCopyFileSystemPath(_, _)),
    export_c_func!(CFURLCreateCopyAppendingPathComponent(_, _, _, _)),
    export_c_func!(CFURLCreateCopyDeletingLastPathComponent(_, _)),
    export_c_func!(CFURLHasDirectoryPath(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_file_property_constants_and_copy_path_contract() {
        let constant_names: Vec<_> = CONSTANTS.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            constant_names,
            ["_kCFURLFileExists", "_kCFURLFileLength"]
        );
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let path = from_rust_string(env, "/tmp/example".to_owned());
        let url: id = msg_class![env; NSURL fileURLWithPath:path];
        let exists_key = get_static_str(env, "kCFURLFileExists");
        let error = env.mem.alloc_and_write(123i32);
        let exists = CFURLCreatePropertyFromResource(
            env,
            kCFAllocatorDefault,
            url,
            exists_key,
            error,
        );
        assert_ne!(exists, nil);
        assert!(!msg![env; exists boolValue]);
        assert_eq!(env.mem.read(error), 0);
        release(env, exists);
        env.mem.free(error.cast());

        let copied = CFURLCopyPath(env, url);
        assert_eq!(to_rust_string(env, copied), "/tmp/example");
        release(env, copied);
        release(env, path);
        assert_eq!(CFURLCopyPath(env, nil), nil);
    }

    #[test]
    fn dev_248_create_with_bytes_accepts_valid_utf8_and_rejects_malformed_utf8() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        env.is_app_picker = true;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let valid: MutPtr<u8> = env.mem.alloc(10).cast();
        env.mem
            .bytes_at_mut(valid.cast(), 10)
            .copy_from_slice(b"/tmp/caf\xc3\xa9");
        let url = CFURLCreateWithBytes(
            env,
            kCFAllocatorDefault,
            valid.cast_const(),
            10,
            kCFStringEncodingUTF8,
            nil,
        );
        assert_ne!(url, nil);
        let path: id = msg![env; url path];
        assert_eq!(to_rust_string(env, path), "/tmp/café");
        release(env, url);

        let invalid: MutPtr<u8> = env.mem.alloc(2).cast();
        env.mem
            .bytes_at_mut(invalid.cast(), 2)
            .copy_from_slice(&[b'/', 0xff]);
        let bad = CFURLCreateWithBytes(
            env,
            kCFAllocatorDefault,
            invalid.cast_const(),
            2,
            kCFStringEncodingUTF8,
            nil,
        );
        assert_eq!(bad, nil);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
