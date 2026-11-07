/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFString` and `CFMutableString`.
//!
//! This is toll-free bridged to `NSString` and `NSMutableString` in
//! Apple's implementation. Here it is the same type.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::cf_dictionary::CFDictionaryRef;
use super::cf_locale::CFLocaleRef;
use super::cf_type::{CFTypeID, CF_STRING_TYPE_ID};
use super::{kCFNotFound, CFComparisonResult, CFIndex, CFOptionFlags, CFRange};
use crate::abi::{DotDotDot, VaList};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{ns_string, unichar, NSNotFound, NSRange, NSUInteger};
use crate::libc::string::strlen;
use crate::mem::{ConstPtr, GuestUSize, MutPtr};
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;

pub type CFStringRef = super::CFTypeRef;
pub type CFMutableStringRef = CFStringRef;

pub type CFStringEncoding = u32;
pub const kCFStringEncodingMacRoman: CFStringEncoding = 0;
pub const kCFStringEncodingASCII: CFStringEncoding = 0x600;
pub const kCFStringEncodingUTF8: CFStringEncoding = 0x8000100;
pub const kCFStringEncodingUnicode: CFStringEncoding = 0x100;
pub const kCFStringEncodingUTF16: CFStringEncoding = kCFStringEncodingUnicode;
pub const kCFStringEncodingUTF16BE: CFStringEncoding = 0x10000100;
pub const kCFStringEncodingUTF16LE: CFStringEncoding = 0x14000100;
pub const kCFStringEncodingISOLatin1: CFStringEncoding = 0x0201;
pub const kCFStringEncodingDOSKorean: CFStringEncoding = 0x0422;
pub const kCFStringEncodingUTF32: CFStringEncoding = 0x0c000100;
pub const kCFStringEncodingUTF32BE: CFStringEncoding = 0x18000100;
pub const kCFStringEncodingUTF32LE: CFStringEncoding = 0x1c000100;

fn CFStringGetTypeID(_env: &mut Environment) -> CFTypeID {
    CF_STRING_TYPE_ID
}

fn CFStringGetSystemEncoding(_env: &mut Environment) -> CFStringEncoding {
    // The historical iPhone OS system encoding is MacRoman. This is the
    // documented Core Foundation fallback, not the user's preferred locale.
    kCFStringEncodingMacRoman
}

fn CFStringAppend(
    env: &mut Environment,
    the_string: CFMutableStringRef,
    appended_string: CFStringRef,
) {
    msg![env; the_string appendString:appended_string]
}

fn CFStringAppendCString(
    env: &mut Environment,
    string: CFMutableStringRef,
    c_string: ConstPtr<u8>,
    encoding: CFStringEncoding,
) {
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    // TODO: avoid copying
    let to_append: id = msg_class![env; NSString stringWithCString:c_string encoding:encoding];
    msg![env; string appendString:to_append]
}

fn CFStringAppendFormat(
    env: &mut Environment,
    string: CFMutableStringRef,
    // Apple's own docs say these are unimplemented!
    _format_options: CFDictionaryRef,
    format: CFStringRef,
    dots: DotDotDot,
) {
    let res = ns_string::with_format(env, format, dots.start());
    let to_append: id = ns_string::from_rust_string(env, res);
    msg![env; string appendString:to_append]
}

pub fn CFStringConvertEncodingToNSStringEncoding(
    _env: &mut Environment,
    encoding: CFStringEncoding,
) -> ns_string::NSStringEncoding {
    match encoding {
        kCFStringEncodingMacRoman => ns_string::NSMacOSRomanStringEncoding,
        kCFStringEncodingASCII => ns_string::NSASCIIStringEncoding,
        kCFStringEncodingUTF8 => ns_string::NSUTF8StringEncoding,
        kCFStringEncodingUTF16 => ns_string::NSUTF16StringEncoding,
        kCFStringEncodingUTF16BE => ns_string::NSUTF16BigEndianStringEncoding,
        kCFStringEncodingUTF16LE => ns_string::NSUTF16LittleEndianStringEncoding,
        kCFStringEncodingISOLatin1 => ns_string::NSISOLatin1StringEncoding,
        kCFStringEncodingDOSKorean => ns_string::NSDOSKoreanStringEncoding,
        kCFStringEncodingUTF32 => ns_string::NSUTF32StringEncoding,
        kCFStringEncodingUTF32BE => ns_string::NSUTF32BigEndianStringEncoding,
        kCFStringEncodingUTF32LE => ns_string::NSUTF32LittleEndianStringEncoding,
        _ => unimplemented!("Unhandled: CFStringEncoding {:#x}", encoding),
    }
}
fn CFStringConvertNSStringEncodingToEncoding(
    _env: &mut Environment,
    encoding: ns_string::NSStringEncoding,
) -> CFStringEncoding {
    match encoding {
        ns_string::NSMacOSRomanStringEncoding => kCFStringEncodingMacRoman,
        ns_string::NSASCIIStringEncoding => kCFStringEncodingASCII,
        ns_string::NSUTF8StringEncoding => kCFStringEncodingUTF8,
        ns_string::NSUTF16StringEncoding => kCFStringEncodingUTF16,
        ns_string::NSUTF16BigEndianStringEncoding => kCFStringEncodingUTF16BE,
        ns_string::NSUTF16LittleEndianStringEncoding => kCFStringEncodingUTF16LE,
        ns_string::NSISOLatin1StringEncoding => kCFStringEncodingISOLatin1,
        ns_string::NSDOSKoreanStringEncoding => kCFStringEncodingDOSKorean,
        ns_string::NSUTF32StringEncoding => kCFStringEncodingUTF32,
        ns_string::NSUTF32BigEndianStringEncoding => kCFStringEncodingUTF32BE,
        ns_string::NSUTF32LittleEndianStringEncoding => kCFStringEncodingUTF32LE,
        _ => unimplemented!("Unhandled: NSStringEncoding {:#x}", encoding),
    }
}

/// Returns Core Foundation's canonical IANA charset name for the encodings we
/// currently support. Unknown encodings have no mapping and return NULL.
fn iana_charset_name(encoding: CFStringEncoding) -> Option<&'static str> {
    Some(match encoding {
        kCFStringEncodingMacRoman => "macintosh",
        kCFStringEncodingASCII => "us-ascii",
        kCFStringEncodingUTF8 => "utf-8",
        kCFStringEncodingUTF16 => "utf-16",
        kCFStringEncodingUTF16BE => "utf-16be",
        kCFStringEncodingUTF16LE => "utf-16le",
        kCFStringEncodingISOLatin1 => "iso-8859-1",
        kCFStringEncodingDOSKorean => "cp949",
        kCFStringEncodingUTF32 => "utf-32",
        kCFStringEncodingUTF32BE => "utf-32be",
        kCFStringEncodingUTF32LE => "utf-32le",
        _ => return None,
    })
}

fn CFStringConvertEncodingToIANACharSetName(
    env: &mut Environment,
    encoding: CFStringEncoding,
) -> CFStringRef {
    let Some(name) = iana_charset_name(encoding) else {
        return nil;
    };
    ns_string::get_static_str(env, name)
}

fn CFStringCreateCopy(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    the_string: CFStringRef,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    msg![env; the_string copy]
}

fn CFStringCreateMutable(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    max_length: CFIndex,
) -> CFMutableStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert_eq!(max_length, 0);
    msg_class![env; NSMutableString new]
}

fn CFStringCreateMutableCopy(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    max_length: CFIndex,
    the_string: CFStringRef,
) -> CFMutableStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert_eq!(max_length, 0);
    msg![env; the_string mutableCopy]
}

fn CFStringCreateWithBytes(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    bytes: ConstPtr<u8>,
    num_bytes: CFIndex,
    encoding: CFStringEncoding,
    is_external: bool,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let length: NSUInteger = num_bytes.try_into().unwrap();
    let mut owned = env.mem.bytes_at(bytes, length).to_vec();
    let mut effective_encoding = encoding;
    if is_external {
        let strip_prefix = |bytes: &mut Vec<u8>, prefix: &[u8]| {
            if bytes.starts_with(prefix) {
                bytes.drain(..prefix.len());
                true
            } else {
                false
            }
        };
        match encoding {
            kCFStringEncodingUTF8 => {
                strip_prefix(&mut owned, &[0xef, 0xbb, 0xbf]);
            }
            kCFStringEncodingUTF16 => {
                effective_encoding = if strip_prefix(&mut owned, &[0xfe, 0xff]) {
                    kCFStringEncodingUTF16BE
                } else if strip_prefix(&mut owned, &[0xff, 0xfe]) {
                    kCFStringEncodingUTF16LE
                } else {
                    // External Unicode representations without a BOM use
                    // network (big-endian) byte order.
                    kCFStringEncodingUTF16BE
                };
            }
            kCFStringEncodingUTF16BE => {
                strip_prefix(&mut owned, &[0xfe, 0xff]);
            }
            kCFStringEncodingUTF16LE => {
                strip_prefix(&mut owned, &[0xff, 0xfe]);
            }
            kCFStringEncodingUTF32 => {
                effective_encoding = if strip_prefix(&mut owned, &[0x00, 0x00, 0xfe, 0xff]) {
                    kCFStringEncodingUTF32BE
                } else if strip_prefix(&mut owned, &[0xff, 0xfe, 0x00, 0x00]) {
                    kCFStringEncodingUTF32LE
                } else {
                    kCFStringEncodingUTF32BE
                };
            }
            kCFStringEncodingUTF32BE => {
                strip_prefix(&mut owned, &[0x00, 0x00, 0xfe, 0xff]);
            }
            kCFStringEncodingUTF32LE => {
                strip_prefix(&mut owned, &[0xff, 0xfe, 0x00, 0x00]);
            }
            _ => {}
        }
    }
    if matches!(
        effective_encoding,
        kCFStringEncodingUTF16 | kCFStringEncodingUTF16BE | kCFStringEncodingUTF16LE
    ) && !owned.len().is_multiple_of(2)
        || matches!(
            effective_encoding,
            kCFStringEncodingUTF32 | kCFStringEncodingUTF32BE | kCFStringEncodingUTF32LE
        ) && !owned.len().is_multiple_of(4)
    {
        return nil;
    }
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, effective_encoding);
    let ns_string: id = msg_class![env; NSString alloc];
    if owned.is_empty() {
        return msg![env; ns_string initWithBytes:bytes length:0u32 encoding:encoding];
    }
    // `CFStringCreateWithBytes` has copying ownership. A temporary guest buffer
    // lets the existing NSString decoder remain the one source of conversion
    // semantics while making that ownership explicit.
    let copied: MutPtr<u8> = env.mem.alloc(owned.len().try_into().unwrap()).cast();
    env.mem
        .bytes_at_mut(copied.cast(), owned.len().try_into().unwrap())
        .copy_from_slice(&owned);
    let result: id = msg![env; ns_string initWithBytes:(copied.cast_const())
                                             length:(owned.len() as u32)
                                           encoding:encoding];
    env.mem.free(copied.cast());
    result
}

fn CFStringCreateWithCString(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    c_string: ConstPtr<u8>,
    encoding: CFStringEncoding,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let ns_string: id = msg_class![env; NSString alloc];
    msg![env; ns_string initWithCString:c_string encoding:encoding]
}

fn CFStringCreateWithCStringNoCopy(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    c_string: ConstPtr<u8>,
    encoding: CFStringEncoding,
    deallocator: CFAllocatorRef,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    assert!(env.mem.read(deallocator).is_null()); // unimplemented
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let c_len: GuestUSize = strlen(env, c_string);
    let ns_string: id = msg_class![env; NSString alloc];
    // Docs of CFStringCreateWithCStringNoCopy says caller should never assume
    // that the object is using the external buffer (it could be copied or even
    // dumped). So we can "safely" invoke a method which does copy!
    msg![env; ns_string initWithBytes:c_string length:c_len encoding:encoding]
}

fn CFStringCreateWithFormat(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    format_options: CFDictionaryRef,
    format: CFStringRef,
    args: DotDotDot,
) -> CFStringRef {
    CFStringCreateWithFormatAndArguments(env, allocator, format_options, format, args.start())
}

fn CFStringCreateWithFormatAndArguments(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    // Apple's own docs say these are unimplemented!
    _format_options: CFDictionaryRef,
    format: CFStringRef,
    args: VaList,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let res = ns_string::with_format(env, format, args);
    ns_string::from_rust_string(env, res)
}

fn CFStringCreateWithSubstring(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    the_string: CFStringRef,
    range: CFRange,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range.length.try_into().unwrap(),
    };
    let res: id = msg![env; the_string substringWithRange:range];
    msg![env; res copy]
}

pub type CFStringCompareFlags = CFOptionFlags;

fn CFStringCompare(
    env: &mut Environment,
    a: CFStringRef,
    b: CFStringRef,
    flags: CFStringCompareFlags,
) -> CFComparisonResult {
    msg![env; a compare:b options:flags]
}

fn CFStringCompareWithOptions(
    env: &mut Environment,
    a: CFStringRef,
    b: CFStringRef,
    range: CFRange,
    flags: CFStringCompareFlags,
) -> CFComparisonResult {
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range.length.try_into().unwrap(),
    };
    // TODO: avoid copying
    let a_sub: id = msg![env; a substringWithRange:range];
    msg![env; a_sub compare:b options:flags]
}

fn CFStringDelete(env: &mut Environment, string: CFMutableStringRef, range: CFRange) {
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range.length.try_into().unwrap(),
    };
    msg![env; string deleteCharactersInRange:range]
}

fn CFStringGetCharacterAtIndex(
    env: &mut Environment,
    the_string: CFStringRef,
    idx: CFIndex,
) -> unichar {
    let idx: NSUInteger = idx.try_into().unwrap();
    msg![env; the_string characterAtIndex:idx]
}

fn CFStringGetCharacters(
    env: &mut Environment,
    string: CFStringRef,
    range: CFRange,
    buffer: MutPtr<unichar>,
) {
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range.length.try_into().unwrap(),
    };
    msg![env; string getCharacters:buffer range:range]
}

fn CFStringGetCharactersPtr(_env: &mut Environment, _the_string: CFStringRef) -> ConstPtr<unichar> {
    // NULL is expected if the function cannot provide a buffer of Unicode
    // characters `efficiently`. Moreover, the same doc claims that the caller
    // should not `count on receiving a non-NULL result from this function
    // under any circumstances`. Win-win situation, if you ask me!
    ConstPtr::null()
}

fn CFStringGetCStringPtr(
    env: &mut Environment,
    the_string: CFStringRef,
    encoding: CFStringEncoding,
) -> ConstPtr<u8> {
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    msg![env; the_string cStringUsingEncoding:encoding]
}

fn CFStringGetCString(
    env: &mut Environment,
    a: CFStringRef,
    buffer: MutPtr<u8>,
    buffer_size: CFIndex,
    encoding: CFStringEncoding,
) -> bool {
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let buffer_size = buffer_size as NSUInteger;
    msg![env; a getCString:buffer maxLength:buffer_size encoding:encoding]
}

fn CFStringGetBytes(
    env: &mut Environment,
    string: CFStringRef,
    range: CFRange,
    encoding: CFStringEncoding,
    loss_byte: u8,
    is_external: bool,
    buffer: MutPtr<u8>,
    max_buf_len: CFIndex,
    used_buf_len: MutPtr<CFIndex>,
) -> CFIndex {
    assert_eq!(loss_byte, 0);
    assert!(!is_external); // TODO

    let range_len = range.length;
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range_len.try_into().unwrap(),
    };
    // TODO: avoid copying
    let substring: id = msg![env; string substringWithRange:range];

    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let buffer_size: NSUInteger = max_buf_len.try_into().unwrap();
    let success: bool =
        ns_string::get_bytes_buffer_inner(env, substring, buffer, buffer_size, encoding, false);
    assert!(success); // TODO
    let length: NSUInteger = msg![env; substring length];
    assert_eq!(length, range_len.try_into().unwrap());

    if !used_buf_len.is_null() {
        let result_bytes_length: NSUInteger =
            msg![env; substring lengthOfBytesUsingEncoding:encoding];
        env.mem
            .write(used_buf_len, result_bytes_length.try_into().unwrap());
    }

    length.try_into().unwrap()
}

fn CFStringGetFileSystemRepresentation(
    env: &mut Environment,
    string: CFStringRef,
    buffer: MutPtr<u8>,
    max_buflen: CFIndex,
) -> bool {
    let max_buflen: NSUInteger = max_buflen.try_into().unwrap();
    msg![env; string getFileSystemRepresentation:buffer maxLength:max_buflen]
}

fn CFStringGetLength(env: &mut Environment, the_string: CFStringRef) -> CFIndex {
    let length: NSUInteger = msg![env; the_string length];
    length.try_into().unwrap()
}

fn CFStringGetFastestEncoding(
    _env: &mut Environment,
    _the_string: CFStringRef,
) -> CFStringEncoding {
    // touchHLE stores NSString contents as Rust UTF-8 strings, making UTF-8
    // both the fastest and a lossless representation of every string.
    kCFStringEncodingUTF8
}

fn CFStringGetSmallestEncoding(
    _env: &mut Environment,
    _the_string: CFStringRef,
) -> CFStringEncoding {
    kCFStringEncodingUTF8
}

fn CFStringGetIntValue(env: &mut Environment, string: CFStringRef) -> i32 {
    // TODO: check for allowed characters
    msg![env; string intValue]
}

fn CFStringFind(
    env: &mut Environment,
    string: CFStringRef,
    to_find: CFStringRef,
    options: CFStringCompareFlags,
) -> CFRange {
    let range: NSRange = msg![env; string rangeOfString:to_find options:options];
    let location: CFIndex = if range.location == NSNotFound as NSUInteger {
        // NSNotFound and kCFNotFound are not the same!
        kCFNotFound
    } else {
        range.location.try_into().unwrap()
    };
    CFRange {
        location,
        length: range.length.try_into().unwrap(),
    }
}

fn CFStringFindWithOptions(
    env: &mut Environment,
    string: CFStringRef,
    to_find: CFStringRef,
    range: CFRange,
    options: CFStringCompareFlags,
    result: MutPtr<CFRange>,
) -> bool {
    let range = NSRange {
        location: range.location.try_into().unwrap(),
        length: range.length.try_into().unwrap(),
    };
    let found: NSRange = msg![env; string rangeOfString:to_find options:options range:range];
    if found.location == NSNotFound as NSUInteger {
        return false;
    }
    if !result.is_null() {
        env.mem.write(
            result,
            CFRange {
                location: found.location.try_into().unwrap(),
                length: found.length.try_into().unwrap(),
            },
        );
    }
    true
}

fn CFStringHasSuffix(env: &mut Environment, the_string: CFStringRef, suffix: CFStringRef) -> bool {
    msg![env; the_string hasSuffix:suffix]
}

fn CFStringUppercase(env: &mut Environment, string: CFStringRef, _locale: CFLocaleRef) {
    // TODO: account for locale
    let uppercase: id = msg![env; string uppercaseString];
    msg![env; string setString:uppercase]
}

type ConstStr255Param = ConstPtr<u8>;
type StringPtr = MutPtr<u8>;

fn CFStringCreateWithPascalString(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    p_str: ConstStr255Param,
    encoding: CFStringEncoding,
) -> CFStringRef {
    let len: CFIndex = env.mem.read(p_str).into();
    let res = CFStringCreateWithBytes(env, allocator, p_str + 1, len, encoding, false);
    assert_eq!(len, CFStringGetLength(env, res));
    log_dbg!(
        "CFStringCreateWithPascalString('{}')",
        ns_string::to_rust_string(env, res)
    );
    res
}

fn CFStringGetPascalString(
    env: &mut Environment,
    the_string: CFStringRef,
    buffer: StringPtr,
    buffer_size: CFIndex,
    encoding: CFStringEncoding,
) -> bool {
    log_dbg!(
        "CFStringGetPascalString('{}')",
        ns_string::to_rust_string(env, the_string)
    );
    let len = CFStringGetLength(env, the_string);
    // first byte of Pascal string is length
    assert!((len + 1) <= buffer_size);
    let len_char: u8 = len.try_into().unwrap();
    env.mem.write(buffer, len_char);
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    ns_string::get_bytes_buffer_inner(
        env,
        the_string,
        buffer + 1,
        len_char.into(),
        encoding,
        false,
    )
}

type CFStringNormalizationForm = CFIndex;

fn CFStringNormalize(
    env: &mut Environment,
    the_string: CFMutableStringRef,
    the_form: CFStringNormalizationForm,
) {
    let str = ns_string::to_rust_string(env, the_string);
    log!("TODO: CFStringNormalize('{}', {})", str, the_form);
    assert!(str.is_ascii()); // TODO
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFStringGetTypeID()),
    export_c_func!(CFStringGetSystemEncoding()),
    export_c_func!(CFStringAppend(_, _)),
    export_c_func!(CFStringAppendCString(_, _, _)),
    export_c_func!(CFStringAppendFormat(_, _, _, _)),
    export_c_func!(CFStringConvertEncodingToNSStringEncoding(_)),
    export_c_func!(CFStringConvertEncodingToIANACharSetName(_)),
    export_c_func!(CFStringConvertNSStringEncodingToEncoding(_)),
    export_c_func!(CFStringCreateCopy(_, _)),
    export_c_func!(CFStringCreateMutable(_, _)),
    export_c_func!(CFStringCreateMutableCopy(_, _, _)),
    export_c_func!(CFStringCreateWithBytes(_, _, _, _, _)),
    export_c_func!(CFStringCreateWithCString(_, _, _)),
    export_c_func!(CFStringCreateWithCStringNoCopy(_, _, _, _)),
    export_c_func!(CFStringCreateWithFormat(_, _, _, _)),
    export_c_func!(CFStringCreateWithFormatAndArguments(_, _, _, _)),
    export_c_func!(CFStringCreateWithSubstring(_, _, _)),
    export_c_func!(CFStringCompare(_, _, _)),
    export_c_func!(CFStringCompareWithOptions(_, _, _, _)),
    export_c_func!(CFStringDelete(_, _)),
    export_c_func!(CFStringGetCharacterAtIndex(_, _)),
    export_c_func!(CFStringGetCharacters(_, _, _)),
    export_c_func!(CFStringGetCharactersPtr(_)),
    export_c_func!(CFStringGetCStringPtr(_, _)),
    export_c_func!(CFStringGetCString(_, _, _, _)),
    export_c_func!(CFStringGetBytes(_, _, _, _, _, _, _, _)),
    export_c_func!(CFStringGetFileSystemRepresentation(_, _, _)),
    export_c_func!(CFStringGetIntValue(_)),
    export_c_func!(CFStringGetLength(_)),
    export_c_func!(CFStringGetFastestEncoding(_)),
    export_c_func!(CFStringGetSmallestEncoding(_)),
    export_c_func!(CFStringFind(_, _, _)),
    export_c_func!(CFStringFindWithOptions(_, _, _, _, _)),
    export_c_func!(CFStringHasSuffix(_, _)),
    export_c_func!(CFStringUppercase(_, _)),
    export_c_func!(CFStringCreateWithPascalString(_, _, _)),
    export_c_func!(CFStringGetPascalString(_, _, _, _)),
    export_c_func!(CFStringNormalize(_, _)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::core_foundation::cf_type::CFGetTypeID;
    use crate::frameworks::foundation::ns_string::{from_rust_string, to_rust_string};
    use crate::mem::Ptr;
    use crate::objc::{id, release};

    fn create_with_bytes(
        env: &mut Environment,
        bytes: &[u8],
        encoding: CFStringEncoding,
        is_external: bool,
    ) -> id {
        let guest_bytes = env.mem.alloc_and_write_cstr(bytes);
        let result = CFStringCreateWithBytes(
            env,
            Ptr::null(),
            guest_bytes.cast_const(),
            bytes.len().try_into().unwrap(),
            encoding,
            is_external,
        );
        env.mem.free(guest_bytes.cast());
        result
    }

    #[test]
    fn supported_iana_charset_names_are_canonical() {
        let mut env = Environment::new_fake_for_test_with_thread();
        assert_eq!(
            CFStringGetSystemEncoding(&mut env),
            kCFStringEncodingMacRoman
        );
        assert_eq!(
            iana_charset_name(kCFStringEncodingMacRoman),
            Some("macintosh")
        );
        assert_eq!(iana_charset_name(kCFStringEncodingASCII), Some("us-ascii"));
        assert_eq!(iana_charset_name(kCFStringEncodingUTF8), Some("utf-8"));
        assert_eq!(iana_charset_name(kCFStringEncodingUTF16), Some("utf-16"));
        assert_eq!(
            iana_charset_name(kCFStringEncodingISOLatin1),
            Some("iso-8859-1")
        );
        assert_eq!(iana_charset_name(kCFStringEncodingDOSKorean), Some("cp949"));
        assert_eq!(iana_charset_name(u32::MAX), None);
    }

    #[test]
    fn dos_korean_bridges_to_nsstring_cp949_encoding() {
        let mut env = Environment::new_fake_for_test_with_thread();
        assert_eq!(
            CFStringConvertEncodingToNSStringEncoding(&mut env, kCFStringEncodingDOSKorean,),
            ns_string::NSDOSKoreanStringEncoding
        );
        assert_eq!(
            CFStringConvertNSStringEncodingToEncoding(
                &mut env,
                ns_string::NSDOSKoreanStringEncoding,
            ),
            kCFStringEncodingDOSKorean
        );
    }

    #[test]
    fn type_id_is_shared_by_immutable_and_mutable_string_classes() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let immutable = from_rust_string(env, "immutable".to_owned());
        let mutable: id = msg_class![env; NSMutableString new];
        let string_type = CFStringGetTypeID(env);
        assert_ne!(string_type, 0);
        assert_eq!(CFGetTypeID(env, immutable), string_type);
        assert_eq!(CFGetTypeID(env, mutable), string_type);
    }

    #[test]
    fn find_with_options_honors_the_supplied_range() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let haystack = from_rust_string(env, "one two TWO".to_owned());
        let needle = from_rust_string(env, "two".to_owned());
        let result = env.mem.alloc_and_write(CFRange {
            location: -1,
            length: 0,
        });
        assert!(CFStringFindWithOptions(
            env,
            haystack,
            needle,
            CFRange {
                location: 4,
                length: 7
            },
            1,
            result,
        ));
        let found = env.mem.read(result);
        assert_eq!((found.location, found.length), (4, 3));
        env.mem.free(result.cast());
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn dev_245_external_representations_honor_boms_validation_and_copying() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;

        let utf8 = create_with_bytes(env, b"\xef\xbb\xbfhello", kCFStringEncodingUTF8, true);
        assert_eq!(to_rust_string(env, utf8), "hello");
        release(env, utf8);

        let utf8_without_external =
            create_with_bytes(env, b"\xef\xbb\xbfhello", kCFStringEncodingUTF8, false);
        assert_eq!(to_rust_string(env, utf8_without_external), "\u{feff}hello");
        release(env, utf8_without_external);

        let utf16_be =
            create_with_bytes(env, &[0xfe, 0xff, 0x00, b'A'], kCFStringEncodingUTF16, true);
        assert_eq!(to_rust_string(env, utf16_be), "A");
        release(env, utf16_be);

        let utf16_le =
            create_with_bytes(env, &[0xff, 0xfe, b'A', 0x00], kCFStringEncodingUTF16, true);
        assert_eq!(to_rust_string(env, utf16_le), "A");
        release(env, utf16_le);

        let utf16_be_explicit = create_with_bytes(
            env,
            &[0xfe, 0xff, 0x00, b'B'],
            kCFStringEncodingUTF16BE,
            true,
        );
        assert_eq!(to_rust_string(env, utf16_be_explicit), "B");
        release(env, utf16_be_explicit);

        let utf16_le_explicit = create_with_bytes(
            env,
            &[0xff, 0xfe, b'C', 0x00],
            kCFStringEncodingUTF16LE,
            true,
        );
        assert_eq!(to_rust_string(env, utf16_le_explicit), "C");
        release(env, utf16_le_explicit);

        let utf32_be = create_with_bytes(
            env,
            &[0x00, 0x00, 0xfe, 0xff, 0x00, 0x00, 0x00, b'D'],
            kCFStringEncodingUTF32,
            true,
        );
        assert_eq!(to_rust_string(env, utf32_be), "D");
        release(env, utf32_be);

        let utf32_le = create_with_bytes(
            env,
            &[0xff, 0xfe, 0x00, 0x00, b'E', 0x00, 0x00, 0x00],
            kCFStringEncodingUTF32,
            true,
        );
        assert_eq!(to_rust_string(env, utf32_le), "E");
        release(env, utf32_le);

        assert_eq!(
            create_with_bytes(env, &[0x00], kCFStringEncodingUTF16, true),
            nil
        );
        assert_eq!(
            create_with_bytes(env, &[0xff], kCFStringEncodingUTF8, true),
            nil
        );
        assert_eq!(
            create_with_bytes(env, &[0x00, 0x00, 0x00], kCFStringEncodingUTF32, true),
            nil
        );

        let mutable_input = env.mem.alloc_and_write_cstr(b"copy");
        let copied = CFStringCreateWithBytes(
            env,
            Ptr::null(),
            mutable_input.cast_const(),
            4,
            kCFStringEncodingUTF8,
            false,
        );
        env.mem.write(mutable_input, b'X');
        assert_eq!(to_rust_string(env, copied), "copy");
        env.mem.free(mutable_input.cast());
        release(env, copied);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
