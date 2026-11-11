/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSMethodSignature`.
//!
//! Resources:
//! - [Type encodings](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html#//apple_ref/doc/uid/TP40008048-CH100)
//! - [Method signatures](https://gcc.gnu.org/onlinedocs/gcc-4.9.0/gcc/Method-signatures.html)

use crate::environment::Environment;
use crate::frameworks::foundation::NSUInteger;
use crate::libc::string::strncpy;
use crate::mem::{ConstPtr, GuestUSize, MutPtr};
use crate::objc::{autorelease, id, objc_classes, ClassExports, HostObject};
use crate::dyld::FunctionExports;
use crate::export_c_func;

struct NSMethodSignatureHostObject {
    return_type: ConstPtr<u8>,
    arg_types: Vec<(ConstPtr<u8>, GuestUSize)>,
}
impl HostObject for NSMethodSignatureHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSMethodSignature: NSObject

+ (id)signatureWithObjCTypes:(ConstPtr<u8>)types {
    let (return_type, arg_types) = parse_signature(env, types);
    let host_object = Box::new(NSMethodSignatureHostObject {
        return_type,
        arg_types,
    });
    let res = env.objc.alloc_object(this, host_object, &mut env.mem);
    autorelease(env, res)
}

- (NSUInteger)numberOfArguments {
    env.objc.borrow::<NSMethodSignatureHostObject>(this).arg_types.len().try_into().unwrap()
}

- (ConstPtr<u8>)getArgumentTypeAtIndex:(NSUInteger)idx {
    env.objc.borrow::<NSMethodSignatureHostObject>(this).arg_types[idx as usize].0
}

- (ConstPtr<u8>)methodReturnType {
    env.objc.borrow::<NSMethodSignatureHostObject>(this).return_type
}

- (NSUInteger)methodReturnLength {
    let return_type = env.objc.borrow::<NSMethodSignatureHostObject>(this).return_type;
    let (_, size, _) = parse_type_layout(env, return_type);
    size
}

- (())dealloc {
    let host_obj = env.objc.borrow::<NSMethodSignatureHostObject>(this);
    env.mem.free(host_obj.return_type.cast_mut().cast());
    for (arg_type, _) in &host_obj.arg_types {
        env.mem.free((*arg_type).cast_mut().cast());
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

fn parse_signature(
    env: &mut Environment,
    sig: ConstPtr<u8>,
) -> (ConstPtr<u8>, Vec<(ConstPtr<u8>, GuestUSize)>) {
    // first parse return type and total size in bytes
    let (scanned, return_type_idx, total_size) = parse_signature_inner(env, sig);
    let return_type_str: ConstPtr<u8> = env.mem.calloc(return_type_idx + 1).cast().cast_const();
    _ = strncpy(env, return_type_str.cast_mut(), sig, return_type_idx);
    log_dbg!(
        "Return type: '{}'; Total size {} bytes",
        env.mem.cstr_at_utf8(return_type_str).unwrap(),
        total_size
    );
    let mut scan_idx = scanned;
    let mut res = Vec::new();
    loop {
        let c = env.mem.read(sig + scan_idx);
        if c == b'\0' {
            break;
        }
        let (scanned, arg_type_idx, offset) = parse_signature_inner(env, sig + scan_idx);
        let arg_type_str: ConstPtr<u8> = env.mem.calloc(arg_type_idx + 1).cast().cast_const();
        _ = strncpy(env, arg_type_str.cast_mut(), sig + scan_idx, arg_type_idx);
        log_dbg!(
            "Arg {} type '{}' at offset {}",
            res.len(),
            env.mem.cstr_at_utf8(arg_type_str).unwrap(),
            offset
        );
        res.push((arg_type_str, offset));
        scan_idx += scanned;
    }
    (return_type_str, res)
}

fn parse_signature_inner(env: &mut Environment, curr: ConstPtr<u8>) -> (GuestUSize, u32, u32) {
    let mut idx = 0;
    let c = env.mem.read(curr);
    match c {
        b'^' => {
            // pointer
            let (scanned, read, size) = parse_signature_inner(env, curr + 1);
            (scanned + 1, read + 1, size)
        }
        b'v' | b'@' | b':' | b'f' | b'c' | b'*' | b'i' | b'I' | b's' | b'S' | b'l'
        | b'L' | b'q' | b'Q' | b'd' | b'B' | b'#' | b'?' => {
            idx += 1;
            let mut size = 0;
            while let cc @ b'0'..=b'9' = env.mem.read(curr + idx) {
                size = size * 10 + (cc - b'0') as u32;
                idx += 1;
            }
            (idx, 1, size)
        }
        _ => unimplemented!("parse_signature_inner: {}", c as char),
    }
}

fn align_up(value: NSUInteger, alignment: NSUInteger) -> NSUInteger {
    if alignment <= 1 {
        value
    } else {
        (value + alignment - 1) & !(alignment - 1)
    }
}

/// Parse one Objective-C type encoding and return the number of bytes consumed,
/// its size and its alignment for the 32-bit iOS ABI.
pub(crate) fn parse_type_layout(
    env: &Environment,
    type_ptr: ConstPtr<u8>,
) -> (GuestUSize, NSUInteger, NSUInteger) {
    let mut idx: GuestUSize = 0;

    while matches!(
        env.mem.read(type_ptr + idx),
        b'r' | b'n' | b'N' | b'o' | b'O' | b'R' | b'V'
    ) {
        idx += 1;
    }

    let encoding = env.mem.read(type_ptr + idx);
    idx += 1;
    let (size, alignment) = match encoding {
        b'v' => (0, 1),
        b'c' | b'C' | b'B' => (1, 1),
        b's' | b'S' => (2, 2),
        b'i' | b'I' | b'l' | b'L' | b'f' => (4, 4),
        b'q' | b'Q' | b'd' => (8, 8),
        b'@' => {
            // Blocks are encoded as @?, and an object may include a quoted
            // class name. Both still have pointer layout.
            if env.mem.read(type_ptr + idx) == b'?' {
                idx += 1;
            } else if env.mem.read(type_ptr + idx) == b'"' {
                idx += 1;
                while env.mem.read(type_ptr + idx) != b'"' {
                    idx += 1;
                }
                idx += 1;
            }
            (4, 4)
        }
        b'#' | b':' | b'*' => (4, 4),
        b'^' => {
            let (scanned, _, _) = parse_type_layout(env, type_ptr + idx);
            idx += scanned;
            (4, 4)
        }
        b'[' => {
            let mut count = 0;
            while let digit @ b'0'..=b'9' = env.mem.read(type_ptr + idx) {
                count = count * 10 + (digit - b'0') as NSUInteger;
                idx += 1;
            }
            let (scanned, element_size, element_alignment) =
                parse_type_layout(env, type_ptr + idx);
            idx += scanned;
            if env.mem.read(type_ptr + idx) == b']' {
                idx += 1;
            }
            (
                align_up(element_size, element_alignment) * count,
                element_alignment,
            )
        }
        b'{' | b'(' => {
            let closing = if encoding == b'{' { b'}' } else { b')' };
            while env.mem.read(type_ptr + idx) != b'='
                && env.mem.read(type_ptr + idx) != closing
            {
                idx += 1;
            }
            if env.mem.read(type_ptr + idx) == b'=' {
                idx += 1;
            }

            let is_union = encoding == b'(';
            let mut aggregate_size = 0;
            let mut aggregate_alignment = 1;
            while env.mem.read(type_ptr + idx) != closing {
                // Modern encodings may include quoted field names.
                if env.mem.read(type_ptr + idx) == b'"' {
                    idx += 1;
                    while env.mem.read(type_ptr + idx) != b'"' {
                        idx += 1;
                    }
                    idx += 1;
                }
                let (scanned, field_size, field_alignment) =
                    parse_type_layout(env, type_ptr + idx);
                idx += scanned;
                aggregate_alignment = aggregate_alignment.max(field_alignment);
                if is_union {
                    aggregate_size = aggregate_size.max(field_size);
                } else {
                    aggregate_size = align_up(aggregate_size, field_alignment) + field_size;
                }
            }
            idx += 1;
            (
                align_up(aggregate_size, aggregate_alignment),
                aggregate_alignment,
            )
        }
        b'b' => {
            let mut bits = 0;
            while let digit @ b'0'..=b'9' = env.mem.read(type_ptr + idx) {
                bits = bits * 10 + (digit - b'0') as NSUInteger;
                idx += 1;
            }
            ((bits + 7) / 8, 1)
        }
        b'?' => (0, 1),
        other => {
            log!(
                "Warning: NSGetSizeAndAlignment encountered unknown type encoding {:?}",
                other as char
            );
            (0, 1)
        }
    };

    // Method encodings can append a stack offset to a type.
    if matches!(env.mem.read(type_ptr + idx), b'+' | b'-') {
        idx += 1;
    }
    while matches!(env.mem.read(type_ptr + idx), b'0'..=b'9') {
        idx += 1;
    }

    (idx, size, alignment)
}

fn NSGetSizeAndAlignment(
    env: &mut Environment,
    type_ptr: ConstPtr<u8>,
    size_ptr: MutPtr<NSUInteger>,
    alignment_ptr: MutPtr<NSUInteger>,
) -> ConstPtr<u8> {
    let (scanned, size, alignment) = parse_type_layout(env, type_ptr);
    if !size_ptr.is_null() {
        env.mem.write(size_ptr, size);
    }
    if !alignment_ptr.is_null() {
        env.mem.write(alignment_ptr, alignment);
    }
    type_ptr + scanned
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(NSGetSizeAndAlignment(_, _, _))];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_and_alignment_handles_nested_types() {
        let mut env = Environment::new_fake_for_test();
        let encoding = env
            .mem
            .alloc_and_write_cstr(b"{Example=c[3i]d}")
            .cast_const();
        let size = env.mem.alloc_and_write(0);
        let alignment = env.mem.alloc_and_write(0);

        let end = NSGetSizeAndAlignment(&mut env, encoding, size, alignment);

        assert_eq!(env.mem.read(size), 24);
        assert_eq!(env.mem.read(alignment), 8);
        assert_eq!(env.mem.read(end), b'\0');
    }
}
