/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSString` class cluster, including `NSMutableString`.
//!
//! Resources:
//! - Apple's [String Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Strings/introStrings.html)

mod path_algorithms;

use super::ns_error::{
    NSCocoaErrorDomain, NSFilePathErrorKey, NSFileReadInapplicableStringEncodingError,
    NSFileReadNoSuchFileError,
};
use super::ns_keyed_archiver::set_value_to_encode_for_current_key;
use super::ns_url_connection::{make_url_error_for_url, NSURLErrorNotConnectedToInternet};
use super::{ns_array, ns_dictionary, ns_keyed_unarchiver};
use super::{
    unichar, NSComparisonResult, NSInteger, NSNotFound, NSOrderedAscending, NSOrderedDescending,
    NSOrderedSame, NSRange, NSUInteger,
};
use crate::abi::VaList;
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::_nib_archive_decoder;
use crate::frameworks::uikit::ui_font::{
    self, UILineBreakMode, UILineBreakModeWordWrap, UITextAlignment, UITextAlignmentLeft,
};
use crate::fs::GuestPath;
use crate::mach_o::MachO;
use crate::mem::{guest_size_of, ConstPtr, ConstVoidPtr, GuestUSize, Mem, MutPtr, Ptr, SafeRead};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, Class, ClassExports,
    HostObject, NSZonePtr, ObjC,
};
use crate::{fs, Environment};
use encoding_rs::{EUC_KR, MACINTOSH, SHIFT_JIS, WINDOWS_1252};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::iter::Peekable;
use std::string::FromUtf16Error;

pub type NSStringEncoding = NSUInteger;
pub type NSStringEncodingConversionOptions = NSUInteger;
pub const NSStringEncodingConversionAllowLossy: NSStringEncodingConversionOptions = 1;
pub const NSStringEncodingConversionExternalRepresentation: NSStringEncodingConversionOptions = 2;
pub const NSASCIIStringEncoding: NSUInteger = 1;
pub const NSUTF8StringEncoding: NSUInteger = 4;
pub const NSISOLatin1StringEncoding: NSUInteger = 5;
pub const NSNonLossyASCIIStringEncoding: NSUInteger = 7;
pub const NSShiftJISStringEncoding: NSUInteger = 8;
pub const NSUnicodeStringEncoding: NSUInteger = 10;
pub const NSWindowsCP1252StringEncoding: NSUInteger = 12;
pub const NSMacOSRomanStringEncoding: NSUInteger = 30;
/// NSString representation of Core Foundation's DOS Korean encoding (CP949).
pub const NSDOSKoreanStringEncoding: NSUInteger = 0x80000422;
pub const NSUTF16StringEncoding: NSUInteger = NSUnicodeStringEncoding;
pub const NSUTF16BigEndianStringEncoding: NSUInteger = 0x90000100;
pub const NSUTF16LittleEndianStringEncoding: NSUInteger = 0x94000100;
pub const NSUTF32StringEncoding: NSUInteger = 0x8c000100;
pub const NSUTF32BigEndianStringEncoding: NSUInteger = 0x98000100;
pub const NSUTF32LittleEndianStringEncoding: NSUInteger = 0x9c000100;

pub type NSStringCompareOptions = NSUInteger;
pub const NSCaseInsensitiveSearch: NSUInteger = 1;
pub const NSLiteralSearch: NSUInteger = 2;
pub const NSBackwardsSearch: NSUInteger = 4;
pub const NSAnchoredSearch: NSUInteger = 8;
pub const NSNumericSearch: NSUInteger = 64;

/// Encodings that C strings (null-terminated byte strings) can use.
const C_STRING_FRIENDLY_ENCODINGS: &[NSStringEncoding] = &[
    NSASCIIStringEncoding,
    NSUTF8StringEncoding,
    NSWindowsCP1252StringEncoding,
    NSMacOSRomanStringEncoding,
    NSISOLatin1StringEncoding,
    NSNonLossyASCIIStringEncoding,
    NSDOSKoreanStringEncoding,
];

pub const NSMaximumStringLength: NSUInteger = (i32::MAX - 1) as _;

#[derive(Default)]
pub struct State {
    static_str_pool: HashMap<&'static str, id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.framework_state.foundation.ns_string
    }
}

fn populate_file_read_error(env: &mut Environment, path: id, out_error: MutPtr<id>) {
    if out_error.is_null() {
        return;
    }
    let user_info = if path == nil {
        nil
    } else {
        let key = get_static_str(env, NSFilePathErrorKey);
        msg_class![env; NSDictionary dictionaryWithObject:path forKey:key]
    };
    let domain = get_static_str(env, NSCocoaErrorDomain);
    let error: id = msg_class![env; NSError errorWithDomain:domain
                                                 code:NSFileReadNoSuchFileError
                                             userInfo:user_info];
    env.mem.write(out_error, error);
}

fn populate_string_encoding_error(env: &mut Environment, path: id, out_error: MutPtr<id>) {
    if out_error.is_null() {
        return;
    }
    let user_info = if path == nil {
        nil
    } else {
        let key = get_static_str(env, NSFilePathErrorKey);
        msg_class![env; NSDictionary dictionaryWithObject:path forKey:key]
    };
    let domain = get_static_str(env, NSCocoaErrorDomain);
    let error: id = msg_class![env; NSError errorWithDomain:domain
                                                 code:NSFileReadInapplicableStringEncodingError
                                             userInfo:user_info];
    env.mem.write(out_error, error);
}

/// Reuse the historical BOM/default-C-string detection used by the deprecated
/// convenience initializer. The `usedEncoding:error:` family exposes the
/// selected value to its caller instead of silently discarding it.
fn detect_file_string_encoding(bytes: &[u8]) -> NSStringEncoding {
    if bytes.len() > 1 && (bytes[..2] == [0xfe, 0xff] || bytes[..2] == [0xff, 0xfe]) {
        NSUTF16StringEncoding
    } else if bytes.len() > 2 && bytes[..3] == [0xef, 0xbb, 0xbf] {
        NSUTF8StringEncoding
    } else {
        // The existing Foundation implementation uses this value as its
        // default C-string encoding.
        NSUTF8StringEncoding
    }
}

/// Constant strings embedded in the app binary use this struct. The name is
/// according to Ghidra, the rest is guesswork.
#[allow(non_camel_case_types)]
struct cfstringStruct {
    _isa: Class,
    flags: u32,
    bytes: ConstPtr<u8>,
    length: NSUInteger,
}
unsafe impl SafeRead for cfstringStruct {}

type Utf16String = Vec<u16>;
type Utf32String = Vec<u32>;

/// Belongs to _touchHLE_NSString.
enum StringHostObject {
    Utf8(Cow<'static, str>),
    /// Not necessarily well-formed UTF-16: might contain unpaired surrogates.
    Utf16(Utf16String),
    Utf32(Utf32String),
}
impl HostObject for StringHostObject {}
impl StringHostObject {
    /// Decode a string while preserving Objective-C's failable initializer
    /// semantics for malformed UTF-8. Other encodings still use the existing
    /// decoder until their error handling is made failable as well.
    fn try_decode(bytes: Cow<[u8]>, encoding: NSStringEncoding) -> Option<StringHostObject> {
        if encoding == NSNonLossyASCIIStringEncoding {
            return decode_non_lossy_ascii(&bytes)
                .map(|string| StringHostObject::Utf8(Cow::Owned(string)));
        }
        if encoding == NSUTF8StringEncoding {
            if bytes.is_empty() {
                return Some(StringHostObject::Utf8(Cow::Borrowed("")));
            }
            let string = String::from_utf8(bytes.into_owned()).ok()?;
            return Some(StringHostObject::Utf8(Cow::Owned(string)));
        }
        Some(Self::decode(bytes, encoding))
    }

    fn decode(bytes: Cow<[u8]>, encoding: NSStringEncoding) -> StringHostObject {
        if bytes.is_empty() {
            return StringHostObject::Utf8(Cow::Borrowed(""));
        }

        // TODO: error handling

        match encoding {
            NSASCIIStringEncoding => {
                assert!(bytes.iter().all(|byte| byte.is_ascii()));
                // Safety: guaranteed by above assertion
                let string = unsafe { String::from_utf8_unchecked(bytes.into_owned()) };
                StringHostObject::Utf8(Cow::Owned(string))
            }
            NSMacOSRomanStringEncoding => {
                let (cow, encoding_used, had_errors) = MACINTOSH.decode(&bytes);
                assert_eq!(encoding_used, MACINTOSH);
                assert!(!had_errors);
                StringHostObject::Utf8(Cow::Owned(cow.into_owned()))
            }
            NSISOLatin1StringEncoding => StringHostObject::Utf8(Cow::Owned(
                bytes.iter().map(|&byte| char::from(byte)).collect(),
            )),
            NSUTF8StringEncoding => {
                let string = String::from_utf8(bytes.into_owned()).unwrap();
                StringHostObject::Utf8(Cow::Owned(string))
            }
            NSWindowsCP1252StringEncoding => {
                let (cow, encoding_used, had_errors) = WINDOWS_1252.decode(&bytes);
                assert_eq!(encoding_used, WINDOWS_1252);
                assert!(!had_errors);
                StringHostObject::Utf8(Cow::Owned(cow.into_owned()))
            }
            NSShiftJISStringEncoding => {
                let (cow, encoding_used, had_errors) = SHIFT_JIS.decode(&bytes);
                assert_eq!(encoding_used, SHIFT_JIS);
                assert!(!had_errors);
                log_dbg!("ShiftJIS decoded {:?}", cow);
                StringHostObject::Utf8(Cow::Owned(cow.to_string()))
            }
            NSDOSKoreanStringEncoding => {
                // encoding_rs' EUC_KR implementation includes the Windows-949
                // extension used by Apple's historical DOS Korean encoding.
                let (cow, encoding_used, had_errors) = EUC_KR.decode(&bytes);
                assert_eq!(encoding_used, EUC_KR);
                assert!(!had_errors);
                StringHostObject::Utf8(Cow::Owned(cow.into_owned()))
            }
            NSUTF16StringEncoding
            | NSUTF16BigEndianStringEncoding
            | NSUTF16LittleEndianStringEncoding => {
                assert!(bytes.len().is_multiple_of(2));

                let is_big_endian = match encoding {
                    NSUTF16BigEndianStringEncoding => true,
                    NSUTF16LittleEndianStringEncoding => false,
                    NSUTF16StringEncoding => match &bytes[0..2] {
                        [0xFE, 0xFF] => true,
                        [0xFF, 0xFE] => false,
                        // Assuming NSUTF16LittleEndianStringEncoding if no BOM
                        // is present
                        // TODO: it seems that foundation can prefix string
                        // with BOM bytes?
                        _ => false,
                    },
                    _ => unreachable!(),
                };
                // TODO: Should the BOM be stripped? Always/sometimes/never?

                StringHostObject::Utf16(if is_big_endian {
                    bytes
                        .chunks(2)
                        .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()))
                        .collect()
                } else {
                    bytes
                        .chunks(2)
                        .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                        .collect()
                })
            }
            NSUTF32StringEncoding
            | NSUTF32BigEndianStringEncoding
            | NSUTF32LittleEndianStringEncoding => {
                assert!(bytes.len().is_multiple_of(4));

                let is_big_endian = match encoding {
                    NSUTF32LittleEndianStringEncoding => false,
                    NSUTF32BigEndianStringEncoding => true,
                    NSUTF32StringEncoding => match &bytes[0..4] {
                        [0x00, 0x00, 0xFE, 0xFF] => true,
                        [0xFF, 0xFE, 0x00, 0x00] => false,
                        // Assuming NSUTF32LittleEndianStringEncoding if no BOM
                        // is present
                        _ => false,
                    },
                    _ => unreachable!(),
                };
                // TODO: Should the BOM be stripped? Always/sometimes/never?

                StringHostObject::Utf32(if is_big_endian {
                    bytes
                        .chunks(4)
                        .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                        .collect()
                } else {
                    bytes
                        .chunks(4)
                        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                        .collect()
                })
            }
            _ => panic!("Unimplemented encoding: {encoding:#x}"),
        }
    }
    fn to_utf8(&self) -> Result<Cow<'static, str>, FromUtf16Error> {
        match self {
            StringHostObject::Utf8(utf8) => Ok(utf8.clone()),
            StringHostObject::Utf16(utf16) => Ok(Cow::Owned(String::from_utf16(utf16)?)),
            StringHostObject::Utf32(utf32) => Ok(Cow::Owned(Self::string_from_utf32(utf32))),
        }
    }

    /// Copy a bounded UTF-8 representation without allocating or changing the
    /// backing representation. This is for the flight recorder, which must not
    /// call guest methods or perturb the object it is observing.
    fn copy_diagnostic_utf8(&self, output: &mut [u8]) -> usize {
        fn append_character(output: &mut [u8], written: &mut usize, character: char) -> bool {
            let mut encoded = [0; 4];
            let encoded = character.encode_utf8(&mut encoded).as_bytes();
            let Some(end) = written.checked_add(encoded.len()) else {
                return false;
            };
            if end > output.len() {
                return false;
            }
            output[*written..end].copy_from_slice(encoded);
            *written = end;
            true
        }

        match self {
            Self::Utf8(value) => {
                let mut written = value.len().min(output.len());
                while !value.is_char_boundary(written) {
                    written -= 1;
                }
                output[..written].copy_from_slice(&value.as_bytes()[..written]);
                written
            }
            Self::Utf16(value) => {
                let mut written = 0;
                for character in char::decode_utf16(value.iter().copied()) {
                    let character = character.unwrap_or(char::REPLACEMENT_CHARACTER);
                    if !append_character(output, &mut written, character) {
                        break;
                    }
                }
                written
            }
            Self::Utf32(value) => {
                let mut written = 0;
                for &character in value {
                    let character =
                        char::from_u32(character).unwrap_or(char::REPLACEMENT_CHARACTER);
                    if !append_character(output, &mut written, character) {
                        break;
                    }
                }
                written
            }
        }
    }
    /// Mutate the object, converting to UTF-16 if the string was not already
    /// UTF-16. Returns a reference to the UTF-16 content and a boolean that is
    /// [true] if a conversion happened.
    fn convert_to_utf16_inplace(&mut self) -> (&mut Utf16String, bool) {
        let converted = match self {
            Self::Utf8(_) | Self::Utf32(_) => {
                *self = Self::Utf16(self.iter_code_units().collect());
                true
            }
            Self::Utf16(_) => false,
        };
        let Self::Utf16(utf16) = self else {
            unreachable!();
        };
        (utf16, converted)
    }
    /// Iterate over the string as UTF-16 code units.
    fn iter_code_units(&self) -> CodeUnitIterator<'_> {
        match self {
            StringHostObject::Utf8(utf8) => CodeUnitIterator::Utf8(utf8.encode_utf16()),
            StringHostObject::Utf16(utf16) => CodeUnitIterator::Utf16(utf16.iter()),
            StringHostObject::Utf32(utf32) => CodeUnitIterator::Utf32(Utf32ToUtf16Iter::new(utf32)),
        }
    }
    fn string_from_utf32(utf32: &Utf32String) -> String {
        utf32
            .iter()
            .map(|&cp| char::from_u32(cp).unwrap())
            .collect()
    }
}

/// Decode Foundation's historical non-lossy ASCII representation. Printable
/// ASCII is copied directly; non-ASCII UTF-16 code units are represented with
/// `\\uXXXX`, and legacy writers may use three-digit octal byte escapes.
/// Malformed escapes make the failable NSString initializer return nil.
fn decode_non_lossy_ascii(bytes: &[u8]) -> Option<String> {
    let mut units = Vec::<u16>::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte != b'\\' {
            if !byte.is_ascii() {
                return None;
            }
            units.push(u16::from(byte));
            index += 1;
            continue;
        }

        index += 1;
        let escaped = *bytes.get(index)?;
        match escaped {
            b'\\' | b'"' | b'\'' => {
                units.push(u16::from(escaped));
                index += 1;
            }
            b'n' => {
                units.push(u16::from(b'\n'));
                index += 1;
            }
            b'r' => {
                units.push(u16::from(b'\r'));
                index += 1;
            }
            b't' => {
                units.push(u16::from(b'\t'));
                index += 1;
            }
            b'b' => {
                units.push(u16::from(b'\x08'));
                index += 1;
            }
            b'f' => {
                units.push(u16::from(b'\x0c'));
                index += 1;
            }
            b'u' | b'U' => {
                let digits = bytes.get(index + 1..index + 5)?;
                let mut value = 0u16;
                for &digit in digits {
                    value = value
                        .checked_mul(16)?
                        .checked_add(u16::from((digit as char).to_digit(16)? as u8))?;
                }
                units.push(value);
                index += 5;
            }
            b'0'..=b'7' => {
                let digits = bytes.get(index..index + 3)?;
                let mut value = 0u16;
                for &digit in digits {
                    if !(b'0'..=b'7').contains(&digit) {
                        return None;
                    }
                    value = value * 8 + u16::from(digit - b'0');
                }
                units.push(value);
                index += 3;
            }
            _ => return None,
        }
    }
    String::from_utf16(&units).ok()
}

#[derive(Clone)]
struct Utf32ToUtf16Iter<'a> {
    iter: std::slice::Iter<'a, u32>,
    extra: u16,
}

impl<'a> Utf32ToUtf16Iter<'a> {
    pub fn new(slice: &'a [u32]) -> Self {
        Self {
            iter: slice.iter(),
            extra: 0,
        }
    }
}

impl<'a> Iterator for Utf32ToUtf16Iter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        if self.extra != 0 {
            let tmp = self.extra;
            self.extra = 0;
            return Some(tmp);
        }

        self.iter.next().map(|&c| {
            let mut buf = [0; 2];
            let n = char::from_u32(c).unwrap().encode_utf16(&mut buf).len();
            if n == 2 {
                self.extra = buf[1];
            }
            buf[0]
        })
    }
}

enum CodeUnitIterator<'a> {
    Utf8(std::str::EncodeUtf16<'a>),
    Utf16(std::slice::Iter<'a, u16>),
    Utf32(Utf32ToUtf16Iter<'a>),
}
impl Iterator for CodeUnitIterator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            CodeUnitIterator::Utf8(iter) => iter.next(),
            CodeUnitIterator::Utf16(iter) => iter.next().copied(),
            CodeUnitIterator::Utf32(iter) => iter.next(),
        }
    }
}
impl Clone for CodeUnitIterator<'_> {
    fn clone(&self) -> Self {
        match self {
            CodeUnitIterator::Utf8(iter) => CodeUnitIterator::Utf8(iter.clone()),
            CodeUnitIterator::Utf16(iter) => CodeUnitIterator::Utf16(iter.clone()),
            CodeUnitIterator::Utf32(iter) => CodeUnitIterator::Utf32(iter.clone()),
        }
    }
}
impl CodeUnitIterator<'_> {
    /// If the sequence of code units in `prefix` is a prefix of `self`,
    /// return [Some] with `self` advanced past that prefix, otherwise [None].
    ///
    /// Code units comparison is done conditional to `case_insensitive` bool:
    /// if it's true, the code units are converted to chars first and compared
    /// as lowercase variants, otherwise the match is exact.
    fn strip_prefix(&self, prefix: &CodeUnitIterator, case_insensitive: bool) -> Option<Self> {
        let mut self_match = self.clone();
        let mut prefix_match = prefix.clone();
        loop {
            match prefix_match.next() {
                None => {
                    return Some(self_match);
                }
                Some(prefix_c) => {
                    let self_c = self_match.next();
                    if case_insensitive {
                        self_c?;
                        let (Some(a_c), Some(b_c)) = (
                            char::from_u32(self_c.unwrap() as u32),
                            char::from_u32(prefix_c as u32),
                        ) else {
                            panic!("Invalid chars in the strings!");
                        };
                        if !a_c.to_lowercase().eq(b_c.to_lowercase()) {
                            return None;
                        }
                    } else if self_c != Some(prefix_c) {
                        return None;
                    }
                }
            }
        }
    }
}

/// Helper for formatting methods. They can't call eachother currently due to
/// full vararg passthrough being missing.
pub fn with_format(env: &mut Environment, format: id, args: VaList) -> String {
    let format_string = to_rust_string(env, format);

    log_dbg!("Formatting {:?} ({:?})", format, format_string);

    let res = crate::libc::stdio::printf::printf_inner::<true, _>(
        env,
        |_, idx| {
            if idx as usize == format_string.len() {
                b'\0'
            } else {
                format_string.as_bytes()[idx as usize]
            }
        },
        args,
    );
    match String::from_utf8_lossy(&res) {
        Cow::Borrowed(str) => str.to_owned(),
        Cow::Owned(string) => {
            // TODO: Support UTF-16 printf directly
            log!(
                "Warning: invalid UTF-8 sequence replaced with U+FFFD in UTF-16 string formatting"
            );
            string
        }
    }
}

pub fn from_rust_ordering(ordering: std::cmp::Ordering) -> NSComparisonResult {
    match ordering {
        std::cmp::Ordering::Less => NSOrderedAscending,
        std::cmp::Ordering::Equal => NSOrderedSame,
        std::cmp::Ordering::Greater => NSOrderedDescending,
    }
}

struct StringsFileParser {
    chars: Vec<char>,
    position: usize,
}

impl StringsFileParser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_trivia(&mut self) -> Option<()> {
        loop {
            while self
                .chars
                .get(self.position)
                .is_some_and(|c| c.is_whitespace())
            {
                self.position += 1;
            }
            if self.chars.get(self.position) == Some(&'/')
                && self.chars.get(self.position + 1) == Some(&'/')
            {
                self.position += 2;
                while self.chars.get(self.position).is_some_and(|c| *c != '\n') {
                    self.position += 1;
                }
            } else if self.chars.get(self.position) == Some(&'/')
                && self.chars.get(self.position + 1) == Some(&'*')
            {
                self.position += 2;
                while !(self.chars.get(self.position) == Some(&'*')
                    && self.chars.get(self.position + 1) == Some(&'/'))
                {
                    if self.position >= self.chars.len() {
                        return None;
                    }
                    self.position += 1;
                }
                self.position += 2;
            } else {
                return Some(());
            }
        }
    }

    fn parse_quoted(&mut self) -> Option<String> {
        if self.chars.get(self.position) != Some(&'"') {
            return None;
        }
        self.position += 1;
        let mut result = String::new();
        loop {
            let ch = *self.chars.get(self.position)?;
            self.position += 1;
            match ch {
                '"' => return Some(result),
                '\\' => {
                    let escaped = *self.chars.get(self.position)?;
                    self.position += 1;
                    match escaped {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'u' | 'U' => {
                            let digits: String = self
                                .chars
                                .get(self.position..self.position + 4)?
                                .iter()
                                .collect();
                            self.position += 4;
                            let scalar = u32::from_str_radix(&digits, 16).ok()?;
                            result.push(char::from_u32(scalar)?);
                        }
                        other => result.push(other),
                    }
                }
                other => result.push(other),
            }
        }
    }

    fn parse_bare_until(&mut self, delimiter: char) -> Option<String> {
        let start = self.position;
        while self
            .chars
            .get(self.position)
            .is_some_and(|c| *c != delimiter)
        {
            self.position += 1;
        }
        if self.position == start {
            return None;
        }
        Some(
            self.chars[start..self.position]
                .iter()
                .collect::<String>()
                .trim()
                .to_owned(),
        )
    }

    fn parse_value(&mut self, delimiter: char) -> Option<String> {
        self.skip_trivia()?;
        if self.chars.get(self.position) == Some(&'"') {
            self.parse_quoted()
        } else {
            self.parse_bare_until(delimiter)
        }
    }

    fn parse(mut self) -> Option<Vec<(String, String)>> {
        let mut entries = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.position == self.chars.len() {
                return Some(entries);
            }
            let key = self.parse_value('=')?;
            self.skip_trivia()?;
            if self.chars.get(self.position) != Some(&'=') {
                return None;
            }
            self.position += 1;
            let value = self.parse_value(';')?;
            self.skip_trivia()?;
            if self.chars.get(self.position) != Some(&';') {
                return None;
            }
            self.position += 1;
            entries.push((key, value));
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSString is an abstract class. A subclass must provide:
// - (NSUInteger)length;
// - (unichar)characterAtIndex:(NSUInteger)index;
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSString.
@implementation NSString: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSString might be subclassed by something which needs allocWithZone:
    // to have the normal behaviour. Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSString", &mut env.mem));
    msg_class![env; _touchHLE_NSString allocWithZone:zone]
}

+ (id)string {
    let str: id = msg![env; this new];
    autorelease(env, str)
}

+ (id)stringWithString:(id)string { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithString:string];
    autorelease(env, new)
}

+ (id)stringWithUTF8String:(ConstPtr<u8>)utf8_string {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUTF8String:utf8_string];
    autorelease(env, new)
}

+ (id)stringWithCString:(ConstPtr<u8>)c_string {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCString:c_string];
    autorelease(env, new)
}

+ (id)stringWithCString:(ConstPtr<u8>)c_string length:(NSUInteger)length {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCString:c_string length:length];
    autorelease(env, new)
}

+ (id)stringWithCString:(ConstPtr<u8>)c_string
               encoding:(NSStringEncoding)encoding {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCString:c_string encoding:encoding];
    autorelease(env, new)
}

+ (id)stringWithContentsOfFile:(id)path { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path];
    autorelease(env, new)
}

+ (id)stringWithContentsOfURL:(id)url { // NSURL*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfURL:url];
    autorelease(env, new)
}

+ (id)stringWithContentsOfFile:(id)path // NSString*
                      encoding:(NSStringEncoding)encoding
                         error:(MutPtr<id>)error { // NSError**
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path
                                              encoding:encoding
                                                 error:error];
    autorelease(env, new)
}

+ (id)stringWithContentsOfFile:(id)path // NSString*
                  usedEncoding:(MutPtr<NSStringEncoding>)used_encoding
                         error:(MutPtr<id>)error { // NSError**
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path
                                         usedEncoding:used_encoding
                                                error:error];
    autorelease(env, new)
}

+ (id)stringWithContentsOfURL:(id)url // NSURL*
                     encoding:(NSStringEncoding)encoding
                         error:(MutPtr<id>)error { // NSError**
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfURL:url
                                             encoding:encoding
                                                error:error];
    autorelease(env, new)
}

+ (id)stringWithFormat:(id)format, // NSString*
                       ...args {
    let res = with_format(env, format, args.start());
    let res = from_rust_string(env, res);
    let res = autorelease(env, res);

    // This will return _touchHLE_NSString or _touchHLE_NSMutableString
    msg![env; this stringWithString:res]
}

+ (id)stringWithCharacters:(ConstPtr<unichar>)characters length:(NSUInteger)length {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCharacters:characters length:length];
    autorelease(env, new)
}

+ (id)pathWithComponents:(id)components {
    let count: NSUInteger = msg![env; components count];
    if count == 0 {
        return get_static_str(env, "");
    }
    let mut res = msg_class![env; NSString new];
    let enumerator: id = msg![env; components objectEnumerator];
    loop {
        let next: id = msg![env; enumerator nextObject];
        if next == nil {
            break;
        }
        let len: NSUInteger = msg![env; next length];
        if len == 0 {
            continue;
        }
        // FIXME: this leads to O(N^2) for N char string, but it should be O(N)
        res = msg![env; res stringByAppendingPathComponent:next];
    }
    log_dbg!("pathWithComponents: {} -> '{}'", {
        let desc = msg![env; components description];
        to_rust_string(env, desc)
    }, to_rust_string(env, res));
    res
}

+ (NSStringEncoding)defaultCStringEncoding {
    // I don't want to figure out what that is on all platforms, and the use
    // I've seen of this method was on ASCII strings, so let's just hardcode
    // UTF-8 and hope that works.
    NSUTF8StringEncoding
}

- (id)propertyListFromStringsFileFormat {
    let source = to_rust_string(env, this).into_owned();
    let Some(parsed) = StringsFileParser::new(&source).parse() else {
        log!("Warning: propertyListFromStringsFileFormat could not parse strings data");
        return nil;
    };
    let mut objects = Vec::with_capacity(parsed.len());
    for (key, value) in parsed {
        let key = from_rust_string(env, key);
        let value = from_rust_string(env, value);
        objects.push((key, value));
    }
    let dictionary = ns_dictionary::dict_from_keys_and_objects(env, &objects);
    for (key, value) in objects {
        release(env, key);
        release(env, value);
    }
    autorelease(env, dictionary)
}

- (id)initWithUTF8String:(ConstPtr<u8>)utf8_string {
    msg![env; this initWithCString:utf8_string encoding:NSUTF8StringEncoding]
}

- (id)initWithCString:(ConstPtr<u8>)c_string {
    let encoding: NSStringEncoding = msg_class![env; NSString defaultCStringEncoding];
    msg![env; this initWithCString:c_string encoding:encoding]
}

- (id)initWithCString:(ConstPtr<u8>)c_string length:(NSUInteger)len {
    let encoding: NSStringEncoding = msg_class![env; NSString defaultCStringEncoding];
    msg![env; this initWithBytes:c_string length:len encoding:encoding]
}

- (id)initWithCString:(ConstPtr<u8>)c_string
             encoding:(NSStringEncoding)encoding {
    assert!(C_STRING_FRIENDLY_ENCODINGS.contains(&encoding), "encoding {encoding}");
    let len: NSUInteger = env.mem.cstr_at(c_string).len().try_into().unwrap();
    msg![env; this initWithBytes:c_string length:len encoding:encoding]
}

- (id)initWithBytesNoCopy:(MutPtr<u8>)bytes
                   length:(NSUInteger)length
                 encoding:(NSStringEncoding)encoding
             freeWhenDone:(bool)free_when_done {
    // Our backing representation owns its decoded contents, so it is safe to
    // release a transferred guest buffer as soon as initialization finishes.
    let bytes_const = bytes.cast_const();
    let new: id = msg![env; this initWithBytes:bytes_const
                                    length:length
                                  encoding:encoding];
    if free_when_done && !bytes.is_null() {
        env.mem.free(bytes.cast_void());
    }
    new
}

- (id)dataUsingEncoding:(NSStringEncoding)encoding {
    msg![env; this dataUsingEncoding:encoding allowLossyConversion:false]
}

- (bool)canBeConvertedToEncoding:(NSStringEncoding)encoding {
    // Foundation answers whether a lossless conversion of the whole string is
    // possible. Use the abstract NSString primitives rather than assuming the
    // receiver is one of touchHLE's concrete string classes.
    let length: NSUInteger = msg![env; this length];
    let units: Vec<u16> = (0..length)
        .map(|index| msg![env; this characterAtIndex:index])
        .collect();
    encode_utf16_units(&units, encoding, false, false).is_some()
}

// These are the two methods that have to be overridden by subclasses, so these
// implementations don't have to care about foreign subclasses.
- (NSUInteger)length {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);

    // To know what length the string has in UTF-16, we need to convert it to
    // UTF-16. If `length` is used, it's likely other methods that operate on
    // UTF-16 code unit boundaries will also be used (e.g. `characterAt:`), so
    // persisting the UTF-16 version lets us potentially optimize future method
    // calls. This is a heuristic though and won't always be optimal.
    let (utf16, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} length]: converted string to UTF-16", this);
    }

    utf16.len().try_into().unwrap()
}
- (u16)characterAtIndex:(NSUInteger)index {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);

    // The string has to be in UTF-16 to get O(1) rather than O(n) indexing, and
    // it's likely this method will be called many times, so converting it to
    // UTF-16 as early as possible and persisting that representation is
    // probably best for performance. This is a heuristic though and won't
    // always be optimal.
    let (utf16, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} characterAtIndex:{:?}]: converted string to UTF-16", this, index);
    }

    // TODO: raise exception instead of panicking?
    utf16[index as usize]
}

- (NSUInteger)lengthOfBytesUsingEncoding:(NSStringEncoding)encoding {
    if C_STRING_FRIENDLY_ENCODINGS.contains(&encoding) {
        let length: NSUInteger = msg![env; this length];
        let units: Vec<u16> = (0..length)
            .map(|index| msg![env; this characterAtIndex:index])
            .collect();
        return encode_utf16_units(&units, encoding, false, false)
            .map(|bytes| bytes.len().try_into().unwrap())
            .unwrap_or(0);
    }
    unimplemented!("lengthOfBytesUsingEncoding: {}", encoding)
}

- (NSRange)rangeOfString:(id)search_string {
    msg![env; this rangeOfString:search_string options:0u32]
}

- (NSRange)rangeOfString:(id)search_string
                 options:(NSStringCompareOptions)options { // NSString *
    let len: NSUInteger = msg![env; this length];
    range_of_string(env, this, search_string, options, NSRange { location: 0, length: len })
}

- (NSRange)rangeOfString:(id)search_string
                 options:(NSStringCompareOptions)options
                   range:(NSRange)range { // NSString *
    range_of_string(env, this, search_string, options, range)
}

- (NSRange)rangeOfCharacterFromSet:(id)set { // NSCharacterSet *
    let length: NSUInteger = msg![env; this length];
    range_of_character_from_set(env, this, set, 0, NSRange { location: 0, length })
}

- (NSRange)rangeOfCharacterFromSet:(id)set // NSCharacterSet *
                              options:(NSStringCompareOptions)options {
    let length: NSUInteger = msg![env; this length];
    range_of_character_from_set(env, this, set, options, NSRange { location: 0, length })
}

- (NSRange)rangeOfCharacterFromSet:(id)set // NSCharacterSet *
                              options:(NSStringCompareOptions)options
                                range:(NSRange)range {
    range_of_character_from_set(env, this, set, options, range)
}

- (id)description {
    this
}
// TODO: debugDescription, localized description (is that a thing for NSString?)

- (NSUInteger)hash {
    // TODO: avoid copying
    super::hash_helper(&to_rust_string(env, this))
}
- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSString class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
    // TODO: avoid copying
    to_rust_string(env, this) == to_rust_string(env, other)
}
- (bool)isEqualToString:(id)other { // NSString*
    if this == other {
        return true;
    }
    if other == nil {
        return false;
    }
    // TODO: avoid copying
    to_rust_string(env, this) == to_rust_string(env, other)
}

- (bool)hasPrefix:(id)str { // NSString*
    // TODO: avoid copying
    let str = to_rust_string(env, str).to_string();
    to_rust_string(env, this).starts_with(&str)
}

- (bool)hasSuffix:(id)str { // NSString*
    // TODO: avoid copying
    let str = to_rust_string(env, str).to_string();
    to_rust_string(env, this).ends_with(&str)
}

- (NSComparisonResult)localizedCompare:(id)other { // NSString*
    // TODO: use current locale
    // TODO: support `compatibility equivalence` in the Unicode standard
    // More info: https://www.objc.io/issues/9-strings/unicode/
    assert!(to_rust_string(env, this).is_ascii());
    assert!(to_rust_string(env, other).is_ascii());
    msg![env; this compare:other]
}

- (NSComparisonResult)localizedCaseInsensitiveCompare:(id)other  { // NSString*
    // TODO: same as localizedCompare
    assert!(to_rust_string(env, this).is_ascii());
    assert!(to_rust_string(env, other).is_ascii());
    msg![env; this caseInsensitiveCompare:other]
}

- (NSComparisonResult)compare:(id)other { // NSString*
    msg![env; this compare:other options:NSLiteralSearch]
}

- (NSComparisonResult)caseInsensitiveCompare:(id)other { //NSString*
    msg![env; this compare:other options:NSCaseInsensitiveSearch]
}

- (NSComparisonResult)compare:(id)other // NSString*
                      options:(NSStringCompareOptions)options
                        range:(NSRange)range {
    // TODO: avoid substring copying
    let substr = msg![env; this substringWithRange:range];
    msg![env; substr compare:other options:options]
}

- (NSComparisonResult)compare:(id)other options:(NSStringCompareOptions)mask { // NSString*
    fn ascii_number(iter: &mut Peekable<CodeUnitIterator>, leftmost_digit: char) -> u32 {
        let mut num = leftmost_digit.to_digit(10).unwrap();
        while let Some(a_digit_char) = iter.next_if(
            |&x| char::from_u32(x as u32).is_some_and(|y| y.is_ascii_digit())
        ) {
            num = num * 10 + char::from_u32(a_digit_char as u32).unwrap().to_digit(10).unwrap();
        }
        num
    }

    if other == nil {
        let regs = env.cpu.regs();
        log!("NSString compare:options: received nil: receiver={:?} mask={:#x} pc={:#x} lr={:#x}; preserving native-contract failure", this, mask, regs[15], regs[14]);
    }
    assert_ne!(other, nil);

    // TODO: support foreign subclasses (perhaps via a helper function that
    // copies the string first)
    let mut a_iter = env.objc.borrow::<StringHostObject>(this).iter_code_units().peekable();
    let mut b_iter = env.objc.borrow::<StringHostObject>(other).iter_code_units().peekable();

    // By default, no mask is a literal search
    let mask = if mask == 0 {
        NSLiteralSearch
    } else {
        mask
    };

    // TODO: OR'ing of compare options
    match mask {
        NSCaseInsensitiveSearch => {
            loop {
                let a_next = a_iter.next();
                let b_next = b_iter.next();
                let (Some(a_unit), Some(b_unit)) = (a_next, b_next) else {
                    return from_rust_ordering(a_next.cmp(&b_next));
                };
                let (Some(a_c), Some(b_c)) = (char::from_u32(a_unit as u32), char::from_u32(b_unit as u32)) else {
                    panic!("Invalid chars in the strings!");
                };

                let insensitive_order = a_c.to_lowercase().cmp(b_c.to_lowercase());
                if insensitive_order != std::cmp::Ordering::Equal {
                    return from_rust_ordering(insensitive_order);
                }
            }
        },
        NSLiteralSearch => {
            from_rust_ordering(a_iter.cmp(b_iter))
        },
        NSNumericSearch => {
            loop {
                let a_next = a_iter.next();
                let b_next = b_iter.next();
                let (Some(a_unit), Some(b_unit)) = (a_next, b_next) else {
                    return from_rust_ordering(a_next.cmp(&b_next));
                };
                let (Some(a_c), Some(b_c)) = (char::from_u32(a_unit as u32), char::from_u32(b_unit as u32)) else {
                    panic!("Invalid chars in the strings!");
                };

                if a_c.is_ascii_digit() && b_c.is_ascii_digit() {
                    let a_int = ascii_number(&mut a_iter, a_c);
                    let b_int = ascii_number(&mut b_iter, b_c);

                    let numeric_order = a_int.cmp(&b_int);
                    if numeric_order != std::cmp::Ordering::Equal {
                        return from_rust_ordering(numeric_order);
                    }
                } else {
                    let char_order = a_c.cmp(&b_c);
                    if char_order != std::cmp::Ordering::Equal {
                        return from_rust_ordering(char_order);
                    }
                }
            }
        },
        mask => unimplemented!("Other mask: {mask}"),
    }
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    let str_mut: id = msg_class![env; NSMutableString alloc];
    // TODO: use `initWithString:`
    let str_mut: id = msg![env; str_mut init];
    () = msg![env; str_mut setString:this];
    str_mut
}

- (bool)getFileSystemRepresentation:(MutPtr<u8>)buffer
                          maxLength:(NSUInteger)buffer_size {
    msg![env; this getCString:buffer
                    maxLength:buffer_size
                     encoding:NSUTF8StringEncoding]
}

- (bool)getCString:(MutPtr<u8>)buffer
         maxLength:(NSUInteger)buffer_size
          encoding:(NSStringEncoding)encoding {
    get_bytes_buffer_inner(env, this, buffer, buffer_size, encoding, true)
}
- (())getCString:(MutPtr<u8>)buffer {
    let encoding: NSStringEncoding = msg_class![env; NSString defaultCStringEncoding];

    // Prevent slice out-of-range error
    let length = (u32::MAX - buffer.to_bits()).min(NSMaximumStringLength);
    let res: bool = msg![env; this getCString:buffer
                                    maxLength:length
                                     encoding:encoding];
    assert!(res);
}

- (bool)getBytes:(MutPtr<u8>)buffer
        maxLength:(NSUInteger)max_length
        usedLength:(MutPtr<NSUInteger>)used_length
          encoding:(NSStringEncoding)encoding
           options:(NSStringEncodingConversionOptions)options
             range:(NSRange)range
    remainingRange:(MutPtr<NSRange>)remaining_range {
    get_bytes_range_inner(
        env,
        this,
        buffer,
        max_length,
        used_length,
        encoding,
        options,
        range,
        remaining_range,
    )
}

- (id)componentsSeparatedByString:(id)separator { // NSString*
    // TODO: support foreign subclasses (perhaps via a helper function that
    // copies the string first)
    let mut main_iter = env.objc.borrow::<StringHostObject>(this)
        .iter_code_units();
    let sep_iter = env.objc.borrow::<StringHostObject>(separator)
        .iter_code_units();

    // TODO: zero-length separator support
    assert!(sep_iter.clone().next().is_some());

    let mut components = Vec::<Utf16String>::new();
    let mut current_component: Utf16String = Vec::new();
    loop {
        if let Some(new_main_iter) = main_iter.strip_prefix(&sep_iter, /* case_insensitive: */ false) {
            // matched separator, end current component
            components.push(std::mem::take(&mut current_component));
            main_iter = new_main_iter;
        } else {
            // no separator match, extend the current component
            match main_iter.next() {
                Some(cur) => current_component.push(cur),
                None => break,
            }
        }
    }
    components.push(current_component);

    // TODO: For a foreign subclass of NSString, do we have to return that
    // subclass? The signature implies this isn't the case and it's probably not
    // worth the effort, but it's an interesting question.
    let class = env.objc.get_known_class("_touchHLE_NSString", &mut env.mem);

    let component_ns_strings = components.drain(..).map(|utf16| {
        let host_object = Box::new(StringHostObject::Utf16(utf16));
        env.objc.alloc_object(class, host_object, &mut env.mem)
    }).collect();
    let array = ns_array::from_vec(env, component_ns_strings);
    autorelease(env, array)
}

- (())getCharacters:(MutPtr<unichar>)buffer
              range:(NSRange)range {
    // TODO: avoid copying
    let ranged = msg![env; this substringWithRange:range];
    msg![env; ranged getCharacters:buffer]
}

- (())getCharacters:(MutPtr<unichar>)buffer {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);

    // this conversion maybe not most optimal heuristic
    let (utf16, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} getCharacters:{:?}]: converted string to UTF-16", this, buffer);
    }

    let len: GuestUSize = guest_size_of::<unichar>() * utf16.len() as GuestUSize;
    let tmp_vec: Vec<u8> = utf16.iter().flat_map(|c| u16::to_le_bytes(*c)).collect();
    _ = env.mem.bytes_at_mut(buffer.cast(), len).write(tmp_vec.as_slice()).unwrap();
}

- (ConstPtr<u8>)cStringUsingEncoding:(NSStringEncoding)encoding {
    let length: NSUInteger = msg![env; this length];
    let units: Vec<u16> = (0..length)
        .map(|index| msg![env; this characterAtIndex:index])
        .collect();
    // Foundation returns NULL when the receiver cannot be represented in the
    // requested encoding. Valid Latin-1 text must not trip an ASCII assertion.
    let Some(bytes) = encode_utf16_units(&units, encoding, false, false) else {
        return Ptr::null();
    };
    let null_size: GuestUSize = match encoding {
        NSUTF8StringEncoding
        | NSASCIIStringEncoding
        | NSMacOSRomanStringEncoding
        | NSISOLatin1StringEncoding
        | NSWindowsCP1252StringEncoding => 1,
        NSUTF16LittleEndianStringEncoding => 2,
        NSUTF32LittleEndianStringEncoding | NSUTF32BigEndianStringEncoding => 4,
        _ => unimplemented!()
    };
    let bytes_size = bytes.len() as GuestUSize;
    let total_size: GuestUSize = bytes_size + null_size;
    let c_string: MutPtr<u8> = env.mem.alloc(total_size).cast();
    _ = env.mem.bytes_at_mut(c_string, bytes_size).write(&bytes).unwrap();
    env.mem
        .bytes_at_mut(c_string + bytes_size, null_size)
        .fill(0);
    // NSData will handle releasing the string (it is autoreleased)
    let _: id = msg_class![env; NSData dataWithBytesNoCopy:(c_string.cast_void())
                                                    length:total_size];
    c_string.cast_const()
}

- (ConstPtr<u8>)cString {
    // TODO: use default C-string encoding of the current locale
    // TODO: raise NSCharacterConversionException if couldn't represent
    msg![env; this UTF8String]
}

- (ConstPtr<u8>)UTF8String {
    msg![env; this cStringUsingEncoding:NSUTF8StringEncoding]
}

- (id)substringToIndex:(NSUInteger)to {
    let mut res_utf16: Utf16String = Vec::with_capacity(to as usize);

    for_each_code_unit(env, this, |idx, c| {
        if idx < to {
            res_utf16.push(c);
        }
    });

    let res = msg_class![env; _touchHLE_NSString alloc];
    *env.objc.borrow_mut(res) = StringHostObject::Utf16(res_utf16);
    autorelease(env, res)
}

- (id)substringFromIndex:(NSUInteger)from {
    let mut res_utf16: Utf16String = Vec::with_capacity(from as usize);

    for_each_code_unit(env, this, |idx, c| {
        if idx >= from {
            res_utf16.push(c);
        }
    });

    let res = msg_class![env; _touchHLE_NSString alloc];
    *env.objc.borrow_mut(res) = StringHostObject::Utf16(res_utf16);
    autorelease(env, res)
}

- (id)stringByTrimmingCharactersInSet:(id)set { // NSCharacterSet*
    let initial_length: NSUInteger = msg![env; this length];

    let mut res_start: NSUInteger = 0;
    let mut res_end = initial_length;

    while res_start < initial_length {
        let c: u16 = msg![env; this characterAtIndex:res_start];
        if msg![env; set characterIsMember:c] {
            res_start += 1;
        } else {
            break;
        }
    }

    while res_end > res_start {
        let c: u16 = msg![env; this characterAtIndex:(res_end - 1)];
        if msg![env; set characterIsMember:c] {
            res_end -= 1;
        } else {
            break;
        }
    }

    assert!(res_end >= res_start);
    let res_length = res_end - res_start;

    if res_length == initial_length {
        let ret = msg![env; this copy];
        autorelease(env, ret)
    } else {
        let range = NSRange{ location: res_start, length: res_length };
        let string: id = msg![env; this substringWithRange:range];
        string
    }
}

- (id)stringByReplacingOccurrencesOfString:(id)target // NSString*
                                withString:(id)replacement { // NSString*
    let length: NSUInteger = msg![env; this length];
    let range = NSRange { location: 0, length };
    msg![env; this stringByReplacingOccurrencesOfString:target
                                             withString:replacement
                                                options:0u32
                                                  range:range]
}

- (id)stringByReplacingOccurrencesOfString:(id)target // NSString*
                                withString:(id)replacement // NSString*
                                   options:(NSStringCompareOptions)options
                                     range:(NSRange)range {
    let loc = range.location;
    let len = range.length;
    let left: id = msg![env; this substringToIndex:loc];
    let middle: id = msg![env; this substringWithRange:range];
    let right: id = msg![env; this substringFromIndex:(loc + len)];
    let new_middle: id = string_by_replacing_occurrences_inner(env, middle, target, replacement, options);
    let res: id = msg![env; left stringByAppendingString:new_middle];
    msg![env; res stringByAppendingString:right]
}

- (id)stringByReplacingCharactersInRange:(NSRange)range
                               withString:(id)replacement { // NSString*
    assert_ne!(replacement, nil);
    let end = range.location.checked_add(range.length).unwrap();
    let length: NSUInteger = msg![env; this length];
    assert!(end <= length);

    let location = range.location;
    let left: id = msg![env; this substringToIndex:location];
    let right: id = msg![env; this substringFromIndex:end];
    let result: id = msg![env; left stringByAppendingString:replacement];
    msg![env; result stringByAppendingString:right]
}

- (id)stringByAppendingString:(id)other { // NSString*
    assert!(other != nil); // TODO: raise exception

    // TODO: ideally, don't convert to UTF-16 here
    let this_len: NSUInteger = msg![env; this length];
    let other_len: NSUInteger = msg![env; other length];
    let mut new_utf16 = Vec::with_capacity((this_len + other_len) as usize);
    for_each_code_unit(env, this, |_idx, c| {
        new_utf16.push(c);
    });
    for_each_code_unit(env, other, |_idx, c| {
        new_utf16.push(c);
    });

    // TODO: For a foreign subclass of NSString, do we have to return that
    // subclass? The signature implies this isn't the case and it's probably not
    // worth the effort, but it's an interesting question.
    let class = env.objc.get_known_class("_touchHLE_NSString", &mut env.mem);
    let host_object = Box::new(StringHostObject::Utf16(new_utf16));
    env.objc.alloc_object(class, host_object, &mut env.mem)
}

- (id)stringByAppendingFormat:(id)format, ...args {
    let new_string = with_format(env, format,  args.start());
    let new_string = from_rust_string(env, new_string);
    let new_string = msg![env; this stringByAppendingString:new_string];
    autorelease(env, new_string)
}

- (id)stringByDeletingLastPathComponent {
    let string = to_rust_string(env, this); // TODO: avoid copying
    let (res, _) = path_algorithms::split_last_path_component(&string);
    let new_string = from_rust_string(env, String::from(res));
    autorelease(env, new_string)
}

- (id)lastPathComponent {
    let string = to_rust_string(env, this); // TODO: avoid copying
    let (_, res) = path_algorithms::split_last_path_component(&string);
    let new_string = from_rust_string(env, String::from(res));
    autorelease(env, new_string)
}

- (id)pathComponents {
    let string = to_rust_string(env, this); // TODO: avoid copying
    let vec = path_algorithms::split_path_components(&string);
    let vec = vec.iter().map(|component| {
        from_rust_string(env, component.to_string())
    }).collect();
    let array = ns_array::from_vec(env, vec);
    autorelease(env, array)
}

- (id)stringByDeletingPathExtension {
    let string = to_rust_string(env, this); // TODO: avoid copying
    let (res, _) = path_algorithms::split_path_extension(&string);
    let new_string = from_rust_string(env, String::from(res));
    autorelease(env, new_string)
}

- (id)pathExtension {
    let string = to_rust_string(env, this); // TODO: avoid copying
    let (_, res) = path_algorithms::split_path_extension(&string);
    let new_string = from_rust_string(env, String::from(res));
    autorelease(env, new_string)
}

- (ConstPtr<u8>)fileSystemRepresentation {
    let file_manager: id = msg_class![env; NSFileManager defaultManager];
    // This behavior was confirmed on the iOS Simulator
    msg![env; file_manager fileSystemRepresentationWithPath:this]
}

- (id)stringByAddingPercentEscapesUsingEncoding:(NSStringEncoding)encoding {
    assert!(encoding == NSASCIIStringEncoding || encoding == NSUTF8StringEncoding); // TODO: other encodings
    let str = to_rust_string(env, this);
    // RFC 2396 unreserved and reserved characters are left as-is. Everything
    // else is escaped byte-by-byte using the requested UTF-8/ASCII-compatible
    // representation. This also preserves existing percent escapes.
    const URL_SAFE: &[u8] = b"-_.~!*'();:@&=+$,/?%#";
    let mut escaped = String::with_capacity(str.len());
    for &byte in str.as_bytes() {
        if byte.is_ascii_alphanumeric() || URL_SAFE.contains(&byte) {
            escaped.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(&mut escaped, "%{byte:02X}").unwrap();
        }
    }
    let escaped_string = from_rust_string(env, escaped);
    autorelease(env, escaped_string)
}

- (id)stringByAppendingPathComponent:(id)component { // NSString*
    // TODO: avoid copying
    let base_str = to_rust_string(env, this);
    let component_str = to_rust_string(env, component);
    let res = path_algorithms::string_by_appending_path_component(&base_str, &component_str);
    log_dbg!("'{}' + '{}' -> '{}'", base_str, component_str, res);
    let new_string = from_rust_string(env, res);
    autorelease(env, new_string)
}

- (id)stringByAppendingPathExtension:(id)extension { // NSString*
    // FIXME: handle edge cases like trailing '/' (may differ from Rust!)
    let mut combined = to_rust_string(env, this).into_owned();
    // TODO: avoid copying
    let extension_string = to_rust_string(env, extension);
    if !extension_string.is_empty(){
        combined.push('.');
        combined.push_str(&extension_string);
    }

    let new_string = from_rust_string(env, combined);
    autorelease(env, new_string)
}

- (id)stringByExpandingTildeInPath {
    let path = to_rust_string(env, this);

    let new_path_str = if let Some(new_path) = path.strip_prefix('~') {
        // ~ and anything up until the first / is stripped
        // This was confirmed using a test app on iOS
        // Examples (of what is placed after home directory):
        //  "~"            -> ""
        //  "~/"           -> ""
        //  "~user"        -> ""
        //  "~/Documents"  -> "/Documents"
        //  "~foo/bar"     -> "/bar"
        //  "~~foo/bar"    -> "/bar"
        let within_home_dir = new_path.split_once('/').map(|x| x.1).unwrap_or("");

        let guest_path = env.fs.home_directory().join(within_home_dir);
        let resolved = fs::resolve_path(&guest_path, None);
        format!("/{}", resolved.join("/"))
    } else {
        // If called on a path with no leading ~ do nothing
        path.to_string()
    };

    log_dbg!("[(NSString *){:?} stringByExpandingTildeInPath] {} -> {}", this, path, new_path_str);

    let new_string = from_rust_string(env, new_path_str);
    autorelease(env, new_string)
}

- (id)stringByStandardizingPath {
    let expanded: id = msg![env; this stringByExpandingTildeInPath];
    let path = to_rust_string(env, expanded); // TODO: avoid copying
    // TODO: Removing an initial component of "/private/var/automount",
    //       "/var/automount”, or "/private” from the path
    assert!(!path.starts_with("/private"));
    assert!(!path.starts_with("/var/automount"));
    // Reducing empty components and references to the current directory
    let path = path.replace("//", "/").replace("/./", "/");
    // Removing a trailing slash from the last component.
    let path = path_algorithms::trim_trailing_slashes(&path);
    // For absolute paths only, resolve references to the parent directory
    let new_path_str = if path.starts_with('/') {
        assert!(!path.starts_with("/.."));
        // Note: while we are using fs function, it's just string manipulation
        // here.
        let resolved = fs::resolve_path(GuestPath::new(path), None);
        let new_path = format!("/{}", resolved.join("/"));
        assert!(!new_path.contains(".."));
        new_path
    } else {
        String::from(path)
    };
    log_dbg!("[(NSString *){:?} stringByStandardizingPath] {} -> {}", this, to_rust_string(env, this), new_path_str);
    let new_string = from_rust_string(env, new_path_str);
    autorelease(env, new_string)
}

- (id)stringsByAppendingPaths:(id)paths {
    let count: NSUInteger = msg![env; paths count];
    let mut_arr: id = msg_class![env; NSMutableArray new];
    for i in 0..count {
        let path: id = msg![env; paths objectAtIndex:i];
        let new: id = msg![env; this stringByAppendingPathComponent:path];
        () = msg![env; mut_arr addObject:new];
    }
    let arr = msg![env; mut_arr copy];
    release(env, mut_arr);
    autorelease(env, arr)
}

// These come from a category in UIKit (UIStringDrawing).
// TODO: Implement categories so we can completely move the code to UIFont.
// TODO: More `sizeWithFont:` variants
- (CGSize)sizeWithFont:(id)font { // UIFont*
    // TODO: avoid copy
    let text = to_rust_string(env, this);
    ui_font::size_with_font(env, font, &text, None)
}
- (CGSize)sizeWithFont:(id)font // UIFont*
     constrainedToSize:(CGSize)size {
    msg![env; this sizeWithFont:font
              constrainedToSize:size
                  lineBreakMode:UILineBreakModeWordWrap]
}
- (CGSize)sizeWithFont:(id)font // UIFont*
     constrainedToSize:(CGSize)size
         lineBreakMode:(UILineBreakMode)line_break_mode {
    // TODO: avoid copy
    let text = to_rust_string(env, this);
    ui_font::size_with_font(env, font, &text, Some((size, line_break_mode)))
}

- (CGSize)drawAtPoint:(CGPoint)point
             withFont:(id)font { // UIFont*
    // TODO: avoid copy
    let text = to_rust_string(env, this);
    ui_font::draw_at_point(env, font, &text, point, None)
}

- (CGSize)drawAtPoint:(CGPoint)point
             forWidth:(CGFloat)width
             withFont:(id)font // UIFont*
        lineBreakMode:(UILineBreakMode)line_break_mode {
    // TODO: avoid copy
    let text = to_rust_string(env, this);
    ui_font::draw_at_point(env, font, &text, point, Some((width, line_break_mode)))
}

- (CGSize)drawInRect:(CGRect)rect
            withFont:(id)font { // UIFont*
    msg![env; this drawInRect:rect
                     withFont:font
                lineBreakMode:UILineBreakModeWordWrap
                    alignment:UITextAlignmentLeft]
}
- (CGSize)drawInRect:(CGRect)rect
            withFont:(id)font // UIFont*
       lineBreakMode:(UILineBreakMode)line_break_mode {
    msg![env; this drawInRect:rect
                     withFont:font
                lineBreakMode:line_break_mode
                    alignment:UITextAlignmentLeft]
}
- (CGSize)drawInRect:(CGRect)rect
            withFont:(id)font // UIFont*
       lineBreakMode:(UILineBreakMode)line_break_mode
           alignment:(UITextAlignment)align {
    // TODO: avoid copy
    let text = to_rust_string(env, this);
    ui_font::draw_in_rect(env, font, &text, rect, line_break_mode, align)
}

- (bool)writeToFile:(id)path // NSString*
         atomically:(bool)use_aux_file {
    let encoding: NSStringEncoding = msg_class![env; NSString defaultCStringEncoding];
    let error: MutPtr<id> = Ptr::null();
    msg![env; this writeToFile:path atomically:use_aux_file encoding:encoding error:error]
}

- (bool)writeToFile:(id)path // NSString*
         atomically:(bool)use_aux_file
           encoding:(NSStringEncoding)encoding
              error:(MutPtr<id>)error { // NSError**
    assert!(encoding == NSUTF8StringEncoding || encoding == NSASCIIStringEncoding);

    let string = to_rust_string(env, this);
    let c_string = env.mem.alloc_and_write_cstr(string.as_bytes());
    // This should not include a NULL terminator!
    let length: NSUInteger = string.len().try_into().unwrap();
    // NSData will handle releasing the string (it is autoreleased)
    let data: id = msg_class![env; NSData dataWithBytesNoCopy:(c_string.cast_void())
                                                    length:length];

    // TODO: write extended attributes about text encoding
    let success: bool = msg![env; data writeToFile:path atomically:use_aux_file];
    if !success && !error.is_null() {
        todo!(); // TODO: create an NSError if requested
    }
    success
}

- (f32)floatValue {
    float_value_common(env, this)
}
- (f64)doubleValue {
    float_value_common(env, this)
}

- (NSInteger)integerValue {
    msg![env; this intValue]
}
- (i32)intValue {
    let st = to_rust_string(env, this);
    let st = st.trim_start();
    let mut cutoff = st.len();
    for (i, c) in st.char_indices() {
        if !c.is_ascii_digit() && c != '+' && c != '-' {
            cutoff = i;
            break;
        }
    }
    // TODO: handle over/underflow properly
    st[..cutoff].parse().unwrap_or(0)
}

- (i64)longLongValue {
    parse_signed_decimal_prefix_i64(&to_rust_string(env, this))
}

- (id)lowercaseString {
    // TODO: check if rust methods are consistent with ObjC one
    let str = to_rust_string(env, this).to_lowercase();
    let res = from_rust_string(env, str);
    autorelease(env, res)
}

- (id)uppercaseString {
    // TODO: check if rust methods are consistent with ObjC one
    let str = to_rust_string(env, this).to_uppercase();
    let res = from_rust_string(env, str);
    autorelease(env, res)
}

@end

// NSMutableString is an abstract class. A subclass must everything
// NSString provides, plus:
// - (void)replaceCharactersInRange:(NSRange)range withString:(NSString)string;
// Note that it inherits from NSString, so we must ensure we override any
// default methods that would be inappropriate for mutability.
@implementation NSMutableString: NSString

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSMutableString might be subclassed by something
    // which needs allocWithZone: to have the normal behaviour.
    // Unimplemented: call superclass alloc then.
    assert!(this == env.objc.get_known_class("NSMutableString", &mut env.mem));
    msg_class![env; _touchHLE_NSMutableString allocWithZone:zone]
}

+ (id)stringWithCapacity:(NSUInteger)capacity {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCapacity:capacity];
    autorelease(env, new)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    let new: id = msg_class![env; NSString alloc];
    msg![env; new initWithString:this]
}

- (())appendString:(id)a_string { // NSString*
    assert_ne!(a_string, nil);
    // TODO: this is inefficient? append in place instead
    let new: id = msg![env; this stringByAppendingString:a_string];
    () = msg![env; this setString:new];
}

- (())insertString:(id)a_string // NSString*
            atIndex:(NSUInteger)index {
    assert_ne!(a_string, nil);
    let range = NSRange { location: index, length: 0 };
    () = msg![env; this replaceCharactersInRange:range withString:a_string];
}

- (())deleteCharactersInRange:(NSRange)range {
    let location = range.location;
    let length = range.length;

    let left: id = if location == 0 {
        get_static_str(env, "")
    } else {
        let left_range = NSRange {
            location: 0,
            length: location,
        };
        msg![env; this substringWithRange:left_range]
    };

    let idx_after_removal = location + length;
    let lenght_str: NSUInteger = msg![env; this length];
    let right: id = if idx_after_removal == lenght_str {
        get_static_str(env, "")
    } else {
        let right_range = NSRange {
            location: idx_after_removal,
            length: lenght_str - idx_after_removal,
        };
        msg![env; this substringWithRange:right_range]
    };

    let res: id = msg![env; left stringByAppendingString:right];
    () = msg![env; this setString:res];
}

- (NSUInteger)replaceOccurrencesOfString:(id)target // NSString*
                               withString:(id)replacement // NSString*
                                  options:(NSStringCompareOptions)options
                                    range:(NSRange)search_range {
    assert_ne!(target, nil);
    assert_ne!(replacement, nil);

    // Find every non-overlapping match before mutating the receiver. Besides
    // preventing replacement text from being searched again, this preserves
    // the original UTF-16 search range when target or replacement aliases the
    // mutable receiver.
    let mut matches = Vec::new();
    let backwards = options & NSBackwardsSearch != 0;
    let mut remaining = search_range;
    loop {
        let found = range_of_string(env, this, target, options, remaining);
        if found.location == NSNotFound as NSUInteger {
            break;
        }
        matches.push(found);
        if options & NSAnchoredSearch != 0 {
            break;
        }

        if backwards {
            remaining.length = found.location - search_range.location;
        } else {
            let next = found.location.checked_add(found.length).unwrap();
            let search_end = search_range
                .location
                .checked_add(search_range.length)
                .unwrap();
            remaining = NSRange {
                location: next,
                length: search_end - next,
            };
        }
    }

    // Apply from the end so every recorded UTF-16 range stays valid even when
    // the replacement has a different length. Copying also gives stable
    // behavior when replacement is the receiver itself.
    matches.sort_by_key(|range| range.location);
    let stable_replacement: id = msg![env; replacement copy];
    for &found in matches.iter().rev() {
        () = msg![env; this replaceCharactersInRange:found withString:stable_replacement];
    }
    release(env, stable_replacement);

    matches.len().try_into().unwrap()
}

@end

// Our private subclass that is the single implementation of NSString for the
// time being.
@implementation _touchHLE_NSString: NSString

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(StringHostObject::Utf8(Cow::Borrowed("")));
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: more init methods

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    let class: Class = msg![env; coder class];
    let keyed_unarch_class: Class = msg_class![env; NSKeyedUnarchiver class];
    let nib_archive_class: Class = msg_class![env; _touchHLE_NIBArchiveDecoder class];
    let new_str = if env.objc.class_is_subclass_of(class, keyed_unarch_class) {
        ns_keyed_unarchiver::decode_current_string(env, coder)
    } else if env.objc.class_is_subclass_of(class, nib_archive_class) {
        _nib_archive_decoder::decode_current_string(env, coder)
    } else {
        unimplemented!();
    };
    release(env, this);
    new_str
}
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

- (id)initWithData:(id)data // NSData *
          encoding:(NSStringEncoding)encoding {
    if data == nil {
        release(env, this);
        return nil;
    }
    let bytes: ConstVoidPtr = msg![env; data bytes];
    let bytes: ConstPtr<u8> = bytes.cast();
    let length: NSUInteger = msg![env; data length];
    let new = msg![env; this initWithBytes:bytes length:length encoding:encoding];
    if new != nil {
        log_dbg!("initWithData:encoding: {}", to_rust_string(env, new));
    }
    new
}

- (id)initWithFormat:(id)format, // NSString*
                     ...args {
    init_with_format_inner(env, this, format, args.start())
}

- (id)initWithFormat:(id)format // NSString*
           arguments:(VaList)args {
    init_with_format_inner(env, this, format, args)
}

- (id)initWithBytes:(ConstPtr<u8>)bytes
             length:(NSUInteger)len
           encoding:(NSStringEncoding)encoding {
    let slice = env.mem.bytes_at(bytes, len);
    let Some(host_object) = StringHostObject::try_decode(Cow::Borrowed(slice), encoding) else {
        release(env, this);
        return nil;
    };

    *env.objc.borrow_mut(this) = host_object;

    this
}

- (id)initWithCharacters:(ConstPtr<unichar>)characters length:(NSUInteger)len {
    assert!(!characters.is_null());
    let num_bytes = len * 2;
    msg![env; this initWithBytes:(characters.cast::<u8>())
                          length:num_bytes
                        encoding:NSUTF16StringEncoding]
}

- (id)initWithString:(id)string { // NSString *
    // TODO: optimize for more common cases (or maybe just call copy?)
    let mut code_units = Vec::new();
    for_each_code_unit(env, string, |_, c| code_units.push(c));
    *env.objc.borrow_mut(this) = StringHostObject::Utf16(code_units);
    this
}

- (id)initWithContentsOfFile:(id)path { // NSString*
    if path == nil {
        return nil;
    }
    // TODO: avoid copy?
    let path = to_rust_string(env, path);
    let Ok(bytes) = env.fs.read(GuestPath::new(&path)) else {
        return nil;
    };
    let encoding = detect_file_string_encoding(&bytes);

    let Some(host_object) = StringHostObject::try_decode(Cow::Owned(bytes), encoding) else {
        release(env, this);
        return nil;
    };
    *env.objc.borrow_mut(this) = host_object;
    this
}

- (id)initWithContentsOfFile:(id)path // NSString*
                usedEncoding:(MutPtr<NSStringEncoding>)used_encoding
                       error:(MutPtr<id>)error { // NSError**
    if path == nil {
        populate_file_read_error(env, path, error);
        release(env, this);
        return nil;
    }
    let path_string = to_rust_string(env, path);
    let Ok(bytes) = env.fs.read(GuestPath::new(&path_string)) else {
        populate_file_read_error(env, path, error);
        release(env, this);
        return nil;
    };
    let encoding = detect_file_string_encoding(&bytes);
    let Some(host_object) = StringHostObject::try_decode(Cow::Owned(bytes), encoding) else {
        populate_string_encoding_error(env, path, error);
        release(env, this);
        return nil;
    };
    if !used_encoding.is_null() {
        env.mem.write(used_encoding, encoding);
    }
    *env.objc.borrow_mut(this) = host_object;
    this
}

// Historical convenience initializer. Like initWithContentsOfFile:, this
// chooses an encoding from a BOM and otherwise uses the default C-string
// encoding. NSData performs the URL loading so unsupported non-file schemes
// fail cleanly instead of being mistaken for guest filesystem paths.
- (id)initWithContentsOfURL:(id)url { // NSURL*
    if url == nil {
        release(env, this);
        return nil;
    }
    let data: id = msg_class![env; NSData dataWithContentsOfURL:url];
    if data == nil {
        release(env, this);
        return nil;
    }

    let bytes_ptr: ConstVoidPtr = msg![env; data bytes];
    let length: NSUInteger = msg![env; data length];
    let bytes = env.mem.bytes_at(bytes_ptr.cast(), length).to_vec();
    let encoding = if bytes.len() > 1
        && (bytes[..2] == [0xFE, 0xFF] || bytes[..2] == [0xFF, 0xFE])
    {
        NSUTF16StringEncoding
    } else if bytes.len() > 2 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        NSUTF8StringEncoding
    } else {
        msg_class![env; NSString defaultCStringEncoding]
    };

    let Some(host_object) = StringHostObject::try_decode(Cow::Owned(bytes), encoding) else {
        release(env, this);
        return nil;
    };
    *env.objc.borrow_mut(this) = host_object;
    this
}

- (id)initWithContentsOfFile:(id)path // NSString*
                    encoding:(NSStringEncoding)encoding
                       error:(MutPtr<id>)error { // NSError**
    // TODO: avoid copy?
    let path_string = to_rust_string(env, path);
    let Ok(bytes) = env.fs.read(GuestPath::new(&path_string)) else {
        populate_file_read_error(env, path, error);
        release(env, this);
        return nil;
    };

    let Some(host_object) = StringHostObject::try_decode(Cow::Owned(bytes), encoding) else {
        // TODO: populate NSError when the caller requests it.
        assert!(error.is_null());
        release(env, this);
        return nil;
    };
    *env.objc.borrow_mut(this) = host_object;
    this
}

- (id)initWithContentsOfURL:(id)url // NSURL*
                    encoding:(NSStringEncoding)encoding
                       error:(MutPtr<id>)error { // NSError**
    let data: id = msg_class![env; NSData dataWithContentsOfURL:url];
    if data == nil {
        if !error.is_null() {
            let url_error =
                make_url_error_for_url(env, url, NSURLErrorNotConnectedToInternet);
            env.mem.write(error, url_error);
        }
        release(env, this);
        return nil;
    }
    // TODO: error handling for encoding
    msg![env; this initWithData:data encoding:encoding]
}

- (bool)isAbsolutePath {
    // TODO: avoid copy?
    let path = to_rust_string(env, this);
    path.starts_with('/') || path.starts_with('~')
}


- (bool)boolValue {
    let string = to_rust_string(env, this);
    let string = string.trim_start_matches(|c: char| {
        c.is_ascii_whitespace() || c == '-' || c == '+' || c == '0'
    });

    let matching_values = "YyTt123456789";
    string.chars()
        .next()
        .map(|c| matching_values.contains(c))
        .unwrap_or(false)
}

- (id)dataUsingEncoding:(NSStringEncoding)encoding
   allowLossyConversion:(bool)lossy {
    data_using_encoding_lossy_inner(env, this, encoding, lossy)
}

- (id)componentsSeparatedByCharactersInSet:(id)cset { // NSCharacterSet*
    let string = {
        let host_object = env.objc.borrow_mut::<StringHostObject>(this);
        let (orig_string, did_convert) = host_object.convert_to_utf16_inplace();
        if did_convert {
            log_dbg!("[{:?} componentsSeparatedByCharactersInSet]: converted string to UTF-16", this);
        }
        orig_string.clone()
    };

    let substrings: Vec<&[u16]> = {
        string.split(|&c| msg![env; cset characterIsMember:c]).collect()
    };

    let substrings: Vec<id> = substrings.into_iter().map(|substr| {
        from_u16_vec(env, substr.to_vec())
    }).collect();

    let res = ns_array::from_vec(env, substrings);
    autorelease(env, res)
}

- (id)substringWithRange:(NSRange)range {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);
    let (orig_string, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} substringWithRange]: converted string to UTF-16", this);
    }
    let host_string =
        orig_string[(range.location as usize)..((range.location + range.length) as usize)].to_vec();
    let res = from_u16_vec(env, host_string);
    autorelease(env, res)
}

- (NSRange)lineRangeForRange:(NSRange)range {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);
    let (orig_string, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} lineRangeForRange]: converted string to UTF-16", this);
    }
    let (start, end, _) = line_range_helper(orig_string, range, true, true);
    NSRange { location: start, length: end - start }
}

- (())getLineStart:(MutPtr<NSUInteger>)start_ptr
               end:(MutPtr<NSUInteger>)end_ptr
       contentsEnd:(MutPtr<NSUInteger>)contents_end_ptr
          forRange:(NSRange)range {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);
    let (orig_string, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} getLineStart]: converted string to UTF-16", this);
    }

    let get_start = !start_ptr.is_null();
    let get_end = !end_ptr.is_null() || !contents_end_ptr.is_null();
    let (start, end, contents_end) = line_range_helper(orig_string, range, get_start, get_end);

    if !start_ptr.is_null() {
        env.mem.write(start_ptr, start);
    }

    if !end_ptr.is_null() {
        env.mem.write(end_ptr, end);
    }

    if !contents_end_ptr.is_null() {
        env.mem.write(contents_end_ptr, contents_end);
    }
}
@end

// Specialised subclass for static-lifetime strings.
// See `get_static_str`.
@implementation _touchHLE_NSString_Static: _touchHLE_NSString

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(StringHostObject::Utf8(Cow::Borrowed("")));
    env.objc.alloc_static_object(this, host_object, &mut env.mem)
}

- (id) retain { this }
- (()) release {}
- (id) autorelease { this }

@end

// Specialised subclasses for static-lifetime strings from the guest app binary.
@implementation _touchHLE_NSString_CFConstantString_UTF8: _touchHLE_NSString_Static

- (ConstPtr<u8>)UTF8String {
    let cfstringStruct { bytes, .. } = env.mem.read(this.cast());

    bytes
}

@end

@implementation _touchHLE_NSString_CFConstantString_UTF16: _touchHLE_NSString_Static
@end

@implementation _touchHLE_NSMutableString: NSMutableString

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(StringHostObject::Utf8(Cow::Borrowed("")));
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCapacity:(NSUInteger)_capacity {
    // TODO: capacity
    msg![env; this init]
}

- (id)initWithBytes:(ConstPtr<u8>)bytes
             length:(NSUInteger)len
           encoding:(NSStringEncoding)encoding {
    let slice = env.mem.bytes_at(bytes, len);
    let Some(host_object) = StringHostObject::try_decode(Cow::Borrowed(slice), encoding) else {
        release(env, this);
        return nil;
    };

    *env.objc.borrow_mut(this) = host_object;

    this
}

- (id)initWithFormat:(id)format, // NSString*
                     ...args {
    init_with_format_inner(env, this, format, args.start())
}

- (id)initWithFormat:(id)format // NSString*
           arguments:(VaList)args {
    init_with_format_inner(env, this, format, args)
}

- (id)initWithString:(id)string { // NSString*
    () = msg![env; this setString:string];
    this
}

// NSCoding implementation
- (())encodeWithCoder:(id)coder {
    encode_with_coder_inner(env, this, coder)
}

- (id)initWithCString:(ConstPtr<u8>)c_string
             encoding:(NSStringEncoding)encoding {
    assert!(C_STRING_FRIENDLY_ENCODINGS.contains(&encoding), "encoding {encoding}");
    let len: NSUInteger = env.mem.cstr_at(c_string).len().try_into().unwrap();
    msg![env; this initWithBytes:c_string length:len encoding:encoding]
}

- (id)dataUsingEncoding:(NSStringEncoding)encoding
   allowLossyConversion:(bool)lossy {
    data_using_encoding_lossy_inner(env, this, encoding, lossy)
}

- (())appendFormat:(id)format, // NSString*
                   ...args {
    assert_ne!(format, nil);
    let res = with_format(env, format, args.start());
    *env.objc.borrow_mut(this) = StringHostObject::Utf8(format!("{}{}", to_rust_string(env, this), res).into());
}

- (())setString:(id)a_string { // NSString*
    assert_ne!(a_string, nil);
    let str = to_rust_string(env, a_string);
    let host_object = StringHostObject::Utf8(str);
    *env.objc.borrow_mut(this) = host_object;
}

- (())replaceCharactersInRange:(NSRange)range
                     withString:(id)a_string { // NSString*
    assert_ne!(a_string, nil);
    let replacement: Vec<u16> = to_rust_string(env, a_string).encode_utf16().collect();
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);
    let (units, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} replaceCharactersInRange:withString:]: converted string to UTF-16", this);
    }
    let start: usize = range.location.try_into().unwrap();
    let length: usize = range.length.try_into().unwrap();
    let end = start.checked_add(length).unwrap();
    assert!(start <= units.len() && end <= units.len());
    units.splice(start..end, replacement);
}

- (id)substringWithRange:(NSRange)range {
    let host_object = env.objc.borrow_mut::<StringHostObject>(this);
    let (orig_string, did_convert) = host_object.convert_to_utf16_inplace();
    if did_convert {
        log_dbg!("[{:?} substringWithRange]: converted string to UTF-16", this);
    }
    let host_string =
        orig_string[(range.location as usize)..((range.location + range.length) as usize)].to_vec();
    let res = from_u16_vec(env, host_string);
    autorelease(env, res)
}

@end

};

/// Shared NSCoding implementation for the concrete immutable and mutable
/// NSString subclasses.
fn encode_with_coder_inner(env: &mut Environment, this: id, coder: id) {
    let string = to_rust_string(env, this);

    // TODO: use some kind of substitution instead?
    // See "Making Substitutions During Coding" in the doc https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Archiving/Articles/codingobjects.html
    set_value_to_encode_for_current_key(env, coder, plist::Value::String(string.to_string()));
}

/// This helper is used in `initWithFormat:` on our private subclasses
/// _touchHLE_NSString and _touchHLE_NSMutableString
fn init_with_format_inner(env: &mut Environment, this: id, format: id, args: VaList) -> id {
    let res = with_format(env, format, args);
    *env.objc.borrow_mut::<StringHostObject>(this) = StringHostObject::Utf8(res.into());
    this
}

/// This helper is used in `dataUsingEncoding:allowLossyConversion:` on our
/// private subclasses _touchHLE_NSString and _touchHLE_NSMutableString
fn data_using_encoding_lossy_inner(
    env: &mut Environment,
    this: id,
    encoding: NSStringEncoding,
    lossy: bool,
) -> id {
    let length: NSUInteger = msg![env; this length];
    let units: Vec<u16> = (0..length)
        .map(|index| msg![env; this characterAtIndex:index])
        .collect();
    let Some(bytes) = encode_utf16_units(&units, encoding, lossy, false) else {
        return nil;
    };
    let c_string = env.mem.alloc_and_write_cstr(&bytes);
    let length: NSUInteger = bytes.len().try_into().unwrap();

    msg_class![env; NSData dataWithBytesNoCopy:(c_string.cast_void()) length:length]
}

/// For use by [crate::dyld]: Handle static strings listed in the app binary.
/// Sets up host objects and updates `isa` fields
/// (`___CFConstantStringClassReference` is ignored by our dyld).
pub fn register_constant_strings(bin: &MachO, mem: &mut Mem, objc: &mut ObjC) {
    let Some(cfstrings) = bin.get_section("__cfstring") else {
        return;
    };

    assert!(cfstrings.size % guest_size_of::<cfstringStruct>() == 0);
    let base: ConstPtr<cfstringStruct> = Ptr::from_bits(cfstrings.addr);
    for i in 0..(cfstrings.size / guest_size_of::<cfstringStruct>()) {
        let cfstr_ptr = base + i;
        let cfstringStruct {
            _isa,
            flags,
            bytes,
            length,
        } = mem.read(cfstr_ptr);

        // Constant CFStrings should (probably) only ever have flags 0x7c8 and
        // 0x7d0.
        // See https://lists.llvm.org/pipermail/cfe-dev/2008-August/002518.html
        let (host_object, class_name) = if flags == 0x7C8 {
            // ASCII
            let decoded = std::str::from_utf8(mem.bytes_at(bytes, length)).unwrap();

            (
                StringHostObject::Utf8(Cow::Owned(String::from(decoded))),
                "_touchHLE_NSString_CFConstantString_UTF8",
            )
        } else if flags == 0x7D0 {
            // UTF16 (length is in code units, not bytes)
            let decoded = mem
                .bytes_at(bytes, length * 2)
                .chunks(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect();

            (
                StringHostObject::Utf16(decoded),
                "_touchHLE_NSString_CFConstantString_UTF16",
            )
        } else {
            panic!("Bad CFTypeID for constant string: {flags:#x}");
        };

        objc.register_static_object(cfstr_ptr.cast().cast_mut(), Box::new(host_object));

        let new_isa = objc.get_known_class(class_name, mem);
        mem.write(cfstr_ptr.cast().cast_mut(), new_isa);
    }
}

/// Shortcut for host code: get an NSString corresponding to a `&'static str`,
/// which does not have to be released and is never deallocated.
pub fn get_static_str(env: &mut Environment, from: &'static str) -> id {
    if let Some(&existing) = State::get(env).static_str_pool.get(from) {
        existing
    } else {
        let new = msg_class![env; _touchHLE_NSString_Static alloc];
        *env.objc.borrow_mut(new) = StringHostObject::Utf8(Cow::Borrowed(from));
        State::get(env).static_str_pool.insert(from, new);
        new
    }
}

/// Shortcut for host code, roughly equivalent to
/// `[[NSString alloc] initWithUTF8String:]` in the proper API.
pub fn from_rust_string(env: &mut Environment, from: String) -> id {
    let string: id = msg_class![env; _touchHLE_NSString alloc];
    let host_object: &mut StringHostObject = env.objc.borrow_mut(string);
    *host_object = StringHostObject::Utf8(Cow::Owned(from));
    string
}

/// Shortcut for host code, roughly equivalent to
/// `[[NSMutableString alloc] initWithUTF8String:]` in the proper API.
pub fn mutable_from_rust_string(env: &mut Environment, from: String) -> id {
    let string: id = msg_class![env; _touchHLE_NSMutableString alloc];
    let host_object: &mut StringHostObject = env.objc.borrow_mut(string);
    *host_object = StringHostObject::Utf8(Cow::Owned(from));
    string
}

/// Shortcut for host code, allocs and inits with the given u16 vec.
pub fn from_u16_vec(env: &mut Environment, from: Vec<u16>) -> id {
    let string: id = msg_class![env; _touchHLE_NSString alloc];
    let host_object: &mut StringHostObject = env.objc.borrow_mut(string);
    *host_object = StringHostObject::Utf16(from);
    string
}

/// Shortcut for host code, provides a view of a string in UTF-8.
/// Warning: This may panic if the string is not valid UTF-16!
///
/// TODO: Try to avoid allocating a new String in more cases.
///
/// TODO: Try to avoid converting from UTF-16 in more cases.
pub fn to_rust_string(env: &mut Environment, string: id) -> Cow<'static, str> {
    // TODO: handle foreign subclasses of NSString
    env.objc
        .borrow_mut::<StringHostObject>(string)
        .to_utf8()
        .unwrap()
}

/// Copy an NSString's contents for bounded diagnostics without guest calls,
/// allocation, mutation or a panic for an unrelated/invalid object.
pub(crate) fn copy_diagnostic_utf8(objc: &ObjC, string: id, output: &mut [u8]) -> Option<usize> {
    let mut host_object = objc.get_host_object(string)?;
    loop {
        if let Some(string) = host_object.as_any().downcast_ref::<StringHostObject>() {
            return Some(string.copy_diagnostic_utf8(output));
        }
        host_object = host_object.as_superclass()?;
    }
}

/// Shortcut for host code, calls a callback once for each UTF-16 code-unit in a
/// string. This is equivalent to a for loop using the `length` and
/// `characterAtIndex:` methods, but much more efficient.
pub fn for_each_code_unit<F>(env: &mut Environment, string: id, mut f: F)
where
    F: FnMut(NSUInteger, u16),
{
    // TODO: handle foreign subclasses of NSString
    let mut idx: NSUInteger = 0;
    env.objc
        .borrow::<StringHostObject>(string)
        .iter_code_units()
        .for_each(|c| {
            f(idx, c);
            idx += 1;
        });
}

/// Helper function for `rangeOfString:options:` method
/// Note: this implementation is linear
fn is_match_at_position<F: Fn(u16, u16) -> bool>(
    env: &mut Environment,
    the_string: id,
    search_string: id,
    start: NSUInteger,
    len: NSUInteger,
    len_search: NSUInteger,
    compare_fn: F,
) -> bool {
    (0..len_search).all(|j| {
        let curr: NSUInteger = start + j;
        if curr < len {
            let a_c: u16 = msg![env; the_string characterAtIndex:curr];
            let b_c: u16 = msg![env; search_string characterAtIndex:j];
            compare_fn(a_c, b_c)
        } else {
            false
        }
    })
}

fn range_of_character_from_set(
    env: &mut Environment,
    string: id,
    set: id,
    options: NSStringCompareOptions,
    range: NSRange,
) -> NSRange {
    assert!(
        set != nil,
        "rangeOfCharacterFromSet requires a character set"
    );
    let length: NSUInteger = msg![env; string length];
    let range_end = range.location.checked_add(range.length).unwrap();
    assert!(
        range_end <= length,
        "rangeOfCharacterFromSet range is out of bounds"
    );

    let unsupported = options & !(NSBackwardsSearch | NSAnchoredSearch);
    if unsupported != 0 {
        log!(
            "Warning: ignoring unsupported NSString character-search option bits {:#x}",
            unsupported
        );
    }
    if range.length == 0 {
        return NSRange {
            location: NSNotFound as NSUInteger,
            length: 0,
        };
    }

    let backwards = options & NSBackwardsSearch != 0;
    let anchored = options & NSAnchoredSearch != 0;
    let matches = |env: &mut Environment, idx| {
        let c: u16 = msg![env; string characterAtIndex:idx];
        msg![env; set characterIsMember:c]
    };

    if anchored {
        let idx = if backwards {
            range_end - 1
        } else {
            range.location
        };
        return if matches(env, idx) {
            NSRange {
                location: idx,
                length: 1,
            }
        } else {
            NSRange {
                location: NSNotFound as NSUInteger,
                length: 0,
            }
        };
    }

    if backwards {
        let mut idx = range_end;
        while idx > range.location {
            idx -= 1;
            if matches(env, idx) {
                return NSRange {
                    location: idx,
                    length: 1,
                };
            }
        }
    } else {
        for idx in range.location..range_end {
            if matches(env, idx) {
                return NSRange {
                    location: idx,
                    length: 1,
                };
            }
        }
    }
    NSRange {
        location: NSNotFound as NSUInteger,
        length: 0,
    }
}

fn range_of_string(
    env: &mut Environment,
    string: id,
    search_string: id,
    options: NSStringCompareOptions,
    range: NSRange,
) -> NSRange {
    log_dbg!(
        "[(NSString *){} rangeOfString:{} options:{} range:{:?}]",
        to_rust_string(env, string),
        to_rust_string(env, search_string),
        options,
        range
    );
    let len: NSUInteger = msg![env; string length];
    let range_end = range.location.checked_add(range.length).unwrap();
    assert!(range_end <= len, "rangeOfString range is out of bounds");
    let len_search: NSUInteger = msg![env; search_string length];
    if len_search == 0 || len_search > range.length {
        return NSRange {
            location: NSNotFound as NSUInteger,
            length: 0,
        };
    }

    let unsupported = options
        & !(NSCaseInsensitiveSearch | NSLiteralSearch | NSBackwardsSearch | NSAnchoredSearch);
    if unsupported != 0 {
        log!(
            "Warning: ignoring unsupported NSString search option bits {:#x}",
            unsupported
        );
    }
    let case_insensitive = options & NSCaseInsensitiveSearch != 0;
    let backwards = options & NSBackwardsSearch != 0;
    let anchored = options & NSAnchoredSearch != 0;
    let last_start = range_end - len_search;

    let matches = |env: &mut Environment, i| {
        is_match_at_position(
            env,
            string,
            search_string,
            i,
            range_end,
            len_search,
            |a, b| {
                if !case_insensitive {
                    return a == b;
                }
                let (Some(a), Some(b)) = (char::from_u32(a as u32), char::from_u32(b as u32))
                else {
                    return a == b;
                };
                a.to_lowercase().eq(b.to_lowercase())
            },
        )
    };

    if anchored {
        let i = if backwards {
            last_start
        } else {
            range.location
        };
        if matches(env, i) {
            return NSRange {
                location: i,
                length: len_search,
            };
        }
    } else if backwards {
        for i in (range.location..=last_start).rev() {
            if matches(env, i) {
                return NSRange {
                    location: i,
                    length: len_search,
                };
            }
        }
    } else {
        for i in range.location..=last_start {
            if matches(env, i) {
                return NSRange {
                    location: i,
                    length: len_search,
                };
            }
        }
    }
    NSRange {
        location: NSNotFound as NSUInteger,
        length: 0,
    }
}

/// Helper function for shared `doubleValue` and `floatValue` implementations.
fn float_value_common<F: std::str::FromStr + Default>(env: &mut Environment, string: id) -> F {
    let st = to_rust_string(env, string);
    let st = st.trim_start();
    let mut cutoff = st.len();
    for (i, c) in st.char_indices() {
        if !c.is_ascii_digit() && c != '.' && c != '+' && c != '-' {
            cutoff = i;
            break;
        }
    }
    // TODO: handle over/underflow properly
    st[..cutoff].parse().unwrap_or(Default::default())
}

/// Decimal-prefix parsing used by NSString's historical `longLongValue`.
/// Overflow follows the strtoll-style contract and clamps to the signed bound.
fn parse_signed_decimal_prefix_i64(input: &str) -> i64 {
    let input = input.trim_start();
    let mut bytes = input.as_bytes().iter().copied().peekable();
    let negative = match bytes.peek() {
        Some(b'-') => {
            bytes.next();
            true
        }
        Some(b'+') => {
            bytes.next();
            false
        }
        _ => false,
    };
    let mut saw_digit = false;
    let mut magnitude: u64 = 0;
    let limit = if negative {
        i64::MAX as u64 + 1
    } else {
        i64::MAX as u64
    };
    for byte in bytes {
        if !byte.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        magnitude = magnitude
            .saturating_mul(10)
            .saturating_add(u64::from(byte - b'0'))
            .min(limit);
    }
    if !saw_digit {
        return 0;
    }
    if negative {
        if magnitude == i64::MAX as u64 + 1 {
            i64::MIN
        } else {
            -(magnitude as i64)
        }
    } else {
        magnitude as i64
    }
}

/// Helper function for lineRangeForRange: and
/// getLineStart:end:contentsEnd:forRange:.
///
/// The two last arguments (get_[start/end]) correspond to the
/// start and end/contentsEnd returns. If false is specified for a given
/// argument, the corresponding return values will not be calculated and
/// set to 0.
fn line_range_helper(
    string: &Utf16String,
    range: NSRange,
    get_start: bool,
    get_end: bool,
) -> (NSUInteger, NSUInteger, NSUInteger) {
    let NSRange {
        location: r_start,
        length,
    } = range;
    let r_end: usize = r_start.checked_add(length).unwrap().try_into().unwrap();
    let r_start: usize = r_start.try_into().unwrap();
    // All the line range functions are "counting the posts, not the fences", so
    // it's ok if r_end = length.
    let str_len = string.len();
    assert!(r_end <= str_len, "Range out of bounds!");

    let mut start_pos: usize = 0;
    if get_start {
        start_pos = r_start;
        while start_pos > 0 {
            let c: u16 = string[start_pos - 1];
            // What counts as a line delimiter is noted here:
            // https://developer.apple.com/documentation/foundation/nsstring/1415111-getlinestart?language=objc
            // There's some special handling for if we start in the
            // middle of a CRLF.
            match c {
                // 'LINE FEED (LF)' (\n), 'NEXT LINE (NEL)', 'LINE SEPARATOR',
                // 'PARAGRAPH SEPARATOR'
                0x000A | 0x0085 | 0x2028 | 0x2029 => break,
                // 'CARRIAGE RETURN (CR)' (\r)
                0x000D => {
                    // If the first character is CR, and it is followed by an
                    // LF, then it's not counted as a line delimiter.
                    // (verified on simulator)
                    if start_pos == r_start && start_pos < str_len {
                        let after_cr: u16 = string[start_pos];
                        // 'LINE FEED (LF)' (\n)
                        if after_cr == 0x000A {
                            start_pos -= 1;
                            continue;
                        }
                    }
                    break;
                }
                _ => {}
            }
            start_pos -= 1;
        }
    }

    // There is very little extra cost for also getting contentsEnd if we're
    // getting end (or vice-versa), so they're combined into one argument.
    let mut end_pos = 0;
    let mut cend_pos = 0;
    if get_end {
        // We want to include the entire line that covers the last char
        // in [r_start, r_end).
        cend_pos = if length > 0 { r_end - 1 } else { r_start };
        while cend_pos < str_len {
            let c: u16 = string[cend_pos];
            // See above about what counts as a line delimiter.
            // There's more understandable handling for CRLF here as well.
            match c {
                //  'NEXT LINE (NEL)', 'LINE SEPARATOR', 'PARAGRAPH SEPARATOR'
                0x0085 | 0x2028 | 0x2029 => {
                    end_pos = cend_pos + 1;
                    break;
                }
                // 'LINE FEED (LF)' (\n),
                0x000A => {
                    // If this is the first character checked, then we also need
                    // to check back for a CR.
                    if cend_pos > 0 && string[cend_pos - 1] == 0x000D {
                        cend_pos -= 1;
                        end_pos = cend_pos + 2;
                    } else {
                        end_pos = cend_pos + 1;
                    }
                    break;
                }
                // 'CARRIAGE RETURN (CR)' (\r)
                0x000D => {
                    // Check if next character exists and is LF.
                    if cend_pos < str_len - 1 {
                        let after_cr: u16 = string[cend_pos + 1];
                        // 'LINE FEED (LF)' (\n)
                        if after_cr == 0x000A {
                            end_pos = cend_pos + 2;
                            break;
                        }
                    }
                    end_pos = cend_pos + 1;
                    break;
                }
                _ => {}
            }
            cend_pos += 1;
        }
        if cend_pos == str_len {
            end_pos = cend_pos
        }
    }

    (
        start_pos.try_into().unwrap(),
        end_pos.try_into().unwrap(),
        cend_pos.try_into().unwrap(),
    )
}

#[cfg(test)]
mod ns_string_tests {
    use super::*;

    #[test]
    fn strings_file_parser_handles_comments_escapes_and_bare_values() {
        let parsed = StringsFileParser::new(
            r#"/* header */ "hello" = "line\nvalue"; // comment
                bare = value;"#,
        )
        .parse()
        .unwrap();
        assert_eq!(
            parsed,
            vec![
                ("hello".to_owned(), "line\nvalue".to_owned()),
                ("bare".to_owned(), "value".to_owned()),
            ]
        );
    }

    #[test]
    fn invalid_utf8_is_a_failed_decode() {
        assert!(
            StringHostObject::try_decode(Cow::Borrowed(b"bplist00\xdf"), NSUTF8StringEncoding,)
                .is_none()
        );
    }

    #[test]
    fn dev_248_non_lossy_ascii_decodes_unicode_octal_and_escapes() {
        let decoded = StringHostObject::try_decode(
            Cow::Borrowed(br"caf\u00e9\\line\n\101"),
            NSNonLossyASCIIStringEncoding,
        )
        .unwrap();
        assert_eq!(decoded.to_utf8().unwrap(), "café\\line\nA");
        assert!(StringHostObject::try_decode(
            Cow::Borrowed(br"broken\u12"),
            NSNonLossyASCIIStringEncoding,
        )
        .is_none());
    }

    #[test]
    fn dos_korean_uses_cp949_and_round_trips() {
        let decoded =
            StringHostObject::try_decode(Cow::Borrowed(&[0xb0, 0xa1]), NSDOSKoreanStringEncoding)
                .unwrap();
        assert_eq!(decoded.to_utf8().unwrap(), "가");
        assert_eq!(
            encode_utf16_units(
                &"가".encode_utf16().collect::<Vec<_>>(),
                NSDOSKoreanStringEncoding,
                false,
                false,
            ),
            Some(vec![0xb0, 0xa1])
        );
    }

    #[test]
    fn latin1_non_ascii_is_encoded_without_an_ascii_only_panic() {
        assert_eq!(
            encode_utf16_units(
                &"café".encode_utf16().collect::<Vec<_>>(),
                NSISOLatin1StringEncoding,
                false,
                false,
            ),
            Some(vec![b'c', b'a', b'f', 0xe9])
        );
        assert_eq!(
            encode_utf16_units(
                &"snowman ☃".encode_utf16().collect::<Vec<_>>(),
                NSISOLatin1StringEncoding,
                false,
                false,
            ),
            None
        );
    }

    #[test]
    fn dev_239_cp1252_c_string_is_encoded_and_nul_terminated() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let string = from_rust_string(env, "cost €5".to_owned());
        let c_string: ConstPtr<u8> =
            msg![env; string cStringUsingEncoding:NSWindowsCP1252StringEncoding];
        assert!(!c_string.is_null());
        assert_eq!(env.mem.cstr_at(c_string), b"cost \x805");
        assert_eq!(env.mem.read(c_string + 7), 0u8);
        release(env, string);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn dev_239_detected_file_encoding_reports_the_encoding_used() {
        assert_eq!(
            detect_file_string_encoding(b"plain text"),
            NSUTF8StringEncoding
        );
        assert_eq!(
            detect_file_string_encoding(&[0xef, 0xbb, 0xbf, b'x']),
            NSUTF8StringEncoding
        );
        assert_eq!(
            detect_file_string_encoding(&[0xff, 0xfe, b'x', 0]),
            NSUTF16StringEncoding
        );

        assert_eq!(
            StringHostObject::try_decode(
                Cow::Borrowed(b"plain text"),
                detect_file_string_encoding(b"plain text"),
            )
            .unwrap()
            .to_utf8()
            .unwrap(),
            "plain text"
        );
    }

    #[test]
    fn signed_long_long_parses_prefix_and_clamps_overflow() {
        assert_eq!(parse_signed_decimal_prefix_i64("  -42px"), -42);
        assert_eq!(parse_signed_decimal_prefix_i64("+17"), 17);
        assert_eq!(parse_signed_decimal_prefix_i64("no digits"), 0);
        assert_eq!(
            parse_signed_decimal_prefix_i64("999999999999999999999"),
            i64::MAX
        );
        assert_eq!(
            parse_signed_decimal_prefix_i64("-999999999999999999999"),
            i64::MIN
        );
    }

    #[test]
    fn missing_file_initializer_populates_requested_error() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // The initializer and NSError factory use a fifth ARM argument, so
        // host-to-guest dispatch needs synthetic stack space in this fixture.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let path = from_rust_string(env, "/definitely-missing/idared-string.txt".to_owned());
        let out_error = env.mem.alloc_and_write(nil);
        let string: id = msg_class![env; NSString alloc];
        let result: id = msg![env; string initWithContentsOfFile:path
                                                  encoding:NSUTF8StringEncoding
                                                     error:out_error];
        assert_eq!(result, nil);

        let error = env.mem.read(out_error);
        assert_ne!(error, nil);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSFileReadNoSuchFileError);
        let domain: id = msg![env; error domain];
        assert_eq!(to_rust_string(env, domain), NSCocoaErrorDomain);
        let user_info: id = msg![env; error userInfo];
        let key = get_static_str(env, NSFilePathErrorKey);
        let stored_path: id = msg![env; user_info objectForKey:key];
        assert_eq!(
            to_rust_string(env, stored_path),
            "/definitely-missing/idared-string.txt"
        );

        env.mem.free(out_error.cast());
        release(env, path);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn unavailable_url_initializer_populates_requested_error() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let url_string = from_rust_string(env, "https://offline.invalid/ad.txt".to_owned());
        let url: id = msg_class![env; NSURL URLWithString:url_string];
        let out_error = env.mem.alloc_and_write(nil);
        let string: id = msg_class![env; NSString alloc];
        let result: id = msg![env; string initWithContentsOfURL:url
                                                  encoding:NSUTF8StringEncoding
                                                     error:out_error];
        assert_eq!(result, nil);

        let error = env.mem.read(out_error);
        assert_ne!(error, nil);
        let code: NSInteger = msg![env; error code];
        assert_eq!(code, NSURLErrorNotConnectedToInternet);
        let user_info: id = msg![env; error userInfo];
        let key = get_static_str(
            env,
            crate::frameworks::foundation::ns_url_connection::NSErrorFailingURLStringKey,
        );
        let failed_url: id = msg![env; user_info objectForKey:key];
        assert_eq!(
            to_rust_string(env, failed_url),
            "https://offline.invalid/ad.txt"
        );

        env.mem.free(out_error.cast());
        release(env, url_string);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn diagnostic_utf8_copy_is_bounded_and_non_mutating() {
        let utf8 = StringHostObject::Utf8(Cow::Borrowed("settings/items"));
        let utf16 = StringHostObject::Utf16("snowman: ☃".encode_utf16().collect());
        let utf32 = StringHostObject::Utf32(vec!['A' as u32, 0x1f680, 0x11_0000]);

        let mut full = [0; 32];
        let written = utf8.copy_diagnostic_utf8(&mut full);
        assert_eq!(&full[..written], b"settings/items");

        let mut bounded = [0; 10];
        let written = utf16.copy_diagnostic_utf8(&mut bounded);
        assert_eq!(&bounded[..written], "snowman: ".as_bytes());

        let mut scalar = [0; 16];
        let written = utf32.copy_diagnostic_utf8(&mut scalar);
        assert_eq!(&scalar[..written], "A🚀�".as_bytes());
    }

    #[test]
    fn mutable_string_insertion_and_replacement_use_utf16_indices() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // replaceCharactersInRange:withString: uses a fifth ARM argument, so
        // host-to-guest dispatch needs synthetic stack space in this fixture.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let string: id = msg_class![env; NSMutableString new];
        let initial = from_rust_string(env, "A🚀B".to_owned());
        () = msg![env; string setString:initial];
        release(env, initial);

        let snowman = from_rust_string(env, "☃".to_owned());
        let rocket_range = NSRange {
            location: 1,
            length: 2,
        };
        () = msg![env; string replaceCharactersInRange:rocket_range withString:snowman];
        let exclamation = from_rust_string(env, "!".to_owned());
        () = msg![env; string insertString:exclamation atIndex:3u32];
        assert_eq!(to_rust_string(env, string), "A☃B!");

        release(env, exclamation);
        release(env, snowman);
        release(env, string);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn mutable_string_replaces_non_overlapping_occurrences_in_utf16_range() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // options: and range: are passed on the synthetic guest stack.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;

        let string = from_rust_string(env, "🚀cat-CAT-cat".to_owned());
        let mutable: id = msg_class![env; NSMutableString alloc];
        let mutable: id = msg![env; mutable initWithString:string];
        let target = from_rust_string(env, "cat".to_owned());
        let replacement = from_rust_string(env, "dog".to_owned());
        let range = NSRange {
            location: 2,
            length: 11,
        };

        let count: NSUInteger = msg![env; mutable
            replaceOccurrencesOfString:target
            withString:replacement
            options:NSCaseInsensitiveSearch
            range:range];

        assert_eq!(count, 3);
        assert_eq!(to_rust_string(env, mutable), "🚀dog-dog-dog");

        release(env, replacement);
        release(env, target);
        release(env, mutable);
        release(env, string);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn backwards_replacement_chooses_the_last_overlapping_match() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;

        let string = from_rust_string(env, "aaa".to_owned());
        let mutable: id = msg_class![env; NSMutableString alloc];
        let mutable: id = msg![env; mutable initWithString:string];
        let target = from_rust_string(env, "aa".to_owned());
        let replacement = from_rust_string(env, "X".to_owned());
        let count: NSUInteger = msg![env; mutable
            replaceOccurrencesOfString:target
            withString:replacement
            options:NSBackwardsSearch
            range:(NSRange { location: 0, length: 3 })];

        assert_eq!(count, 1);
        assert_eq!(to_rust_string(env, mutable), "aX");

        release(env, replacement);
        release(env, target);
        release(env, mutable);
        release(env, string);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn linerange_tests() {
        let range = |x, y| NSRange {
            location: x,
            length: y,
        };
        let str1: Utf16String = "abcd\nab".encode_utf16().collect();
        assert!(line_range_helper(&str1, range(5, 1), true, true) == (5, 7, 7));
        assert!(line_range_helper(&str1, range(4, 1), true, true) == (0, 5, 4));

        let str2: Utf16String = "abc\r".encode_utf16().collect();
        assert!(line_range_helper(&str2, range(4, 0), true, true) == (4, 4, 4));
        assert!(line_range_helper(&str2, range(3, 1), true, true) == (0, 4, 3));

        let str3: Utf16String = "abc\r\nab".encode_utf16().collect();
        assert!(line_range_helper(&str3, range(4, 0), true, true) == (0, 5, 3));
        assert!(line_range_helper(&str3, range(4, 1), true, true) == (0, 5, 3));
        assert!(line_range_helper(&str3, range(6, 1), true, true) == (5, 7, 7));
        assert!(line_range_helper(&str3, range(4, 2), true, true) == (0, 7, 7));

        let str4: Utf16String = "\r\n".encode_utf16().collect();
        assert!(line_range_helper(&str4, range(1, 0), true, true) == (0, 2, 0));
        assert!(line_range_helper(&str4, range(1, 1), true, true) == (0, 2, 0));
        assert!(line_range_helper(&str4, range(0, 0), true, true) == (0, 2, 0));

        let str5: Utf16String = "abcd\na\n".encode_utf16().collect();
        assert!(line_range_helper(&str5, range(6, 1), true, true) == (5, 7, 6));
        assert!(line_range_helper(&str5, range(4, 1), true, true) == (0, 5, 4));
    }
}

/// Helper function to get bytes of a string in the specified NSStringEncoding.
///
/// `include_null_terminator` flag controls if NULL-terminator should be
/// included or not.
/// Return value specify if provided buffer was ok or too small.
/// (TODO: indicate error on conversion too)
/// In case of small buffer no data is written.
///
/// Right now this helper is used for `NSString getCString:maxLength:encoding:`
/// method, `CFStringGetPascalString` and `CFStringGetBytes` functions.
pub fn get_bytes_buffer_inner(
    env: &mut Environment,
    str: id, // NSString *
    buffer: MutPtr<u8>,
    buffer_size: NSUInteger,
    encoding: NSStringEncoding,
    include_null_terminator: bool,
) -> bool {
    // TODO: other encodings
    assert!(
        C_STRING_FRIENDLY_ENCODINGS.contains(&encoding),
        "unsupported C-string encoding {encoding:#x}"
    );

    let src = to_rust_string(env, str);
    if encoding == NSDOSKoreanStringEncoding {
        let units: Vec<u16> = src.encode_utf16().collect();
        let Some(mut encoded) = encode_utf16_units(&units, encoding, false, false) else {
            return false;
        };
        if include_null_terminator {
            encoded.push(0);
        }
        let dest = env.mem.bytes_at_mut(buffer, buffer_size);
        if dest.len() < encoded.len() {
            return false;
        }
        dest[..encoded.len()].copy_from_slice(&encoded);
        return true;
    }
    if encoding == NSASCIIStringEncoding
        || encoding == NSMacOSRomanStringEncoding
        || encoding == NSISOLatin1StringEncoding
    {
        // TODO: properly support Mac OS Roman and ISO Latin 1 encoding.
        // The first 128 characters are identical to the ASCII
        assert!(src.as_bytes().iter().all(|byte| byte.is_ascii()));
    }
    let dest = env.mem.bytes_at_mut(buffer, buffer_size);
    let src_len = if include_null_terminator {
        src.len() + 1
    } else {
        src.len()
    };
    if dest.len() < src_len {
        return false;
    }

    let iter: Box<dyn Iterator<Item = &u8>> = if include_null_terminator {
        Box::new(src.as_bytes().iter().chain(b"\0".iter()))
    } else {
        Box::new(src.as_bytes().iter())
    };
    for (i, &byte) in iter.enumerate() {
        dest[i] = byte;
    }

    true
}

fn encode_utf16_units(
    units: &[u16],
    encoding: NSStringEncoding,
    lossy: bool,
    external: bool,
) -> Option<Vec<u8>> {
    let string = if lossy {
        String::from_utf16_lossy(units)
    } else {
        String::from_utf16(units).ok()?
    };

    let mut bytes = match encoding {
        NSUTF8StringEncoding => string.into_bytes(),
        NSASCIIStringEncoding => {
            if !lossy && !string.is_ascii() {
                return None;
            }
            string
                .chars()
                .map(|character| {
                    if character.is_ascii() {
                        character as u8
                    } else {
                        b'?'
                    }
                })
                .collect()
        }
        NSMacOSRomanStringEncoding => {
            let (encoded, _, had_errors) = MACINTOSH.encode(&string);
            if had_errors && !lossy {
                return None;
            }
            encoded.into_owned()
        }
        NSISOLatin1StringEncoding => {
            if !lossy && string.chars().any(|character| character as u32 > 0xff) {
                return None;
            }
            string
                .chars()
                .map(|character| u8::try_from(character as u32).unwrap_or(b'?'))
                .collect()
        }
        NSWindowsCP1252StringEncoding => {
            let (encoded, _, had_errors) = WINDOWS_1252.encode(&string);
            if had_errors && !lossy {
                return None;
            }
            encoded.into_owned()
        }
        NSShiftJISStringEncoding => {
            let (encoded, _, had_errors) = SHIFT_JIS.encode(&string);
            if had_errors && !lossy {
                return None;
            }
            encoded.into_owned()
        }
        NSDOSKoreanStringEncoding => {
            let (encoded, _, had_errors) = EUC_KR.encode(&string);
            if had_errors && !lossy {
                return None;
            }
            encoded.into_owned()
        }
        NSUTF16StringEncoding | NSUTF16LittleEndianStringEncoding => {
            units.iter().flat_map(|unit| unit.to_le_bytes()).collect()
        }
        NSUTF16BigEndianStringEncoding => {
            units.iter().flat_map(|unit| unit.to_be_bytes()).collect()
        }
        NSUTF32StringEncoding | NSUTF32LittleEndianStringEncoding => string
            .chars()
            .flat_map(|character| (character as u32).to_le_bytes())
            .collect(),
        NSUTF32BigEndianStringEncoding => string
            .chars()
            .flat_map(|character| (character as u32).to_be_bytes())
            .collect(),
        _ => return None,
    };

    if external && !units.is_empty() {
        let prefix: &[u8] = match encoding {
            NSUTF16StringEncoding => &[0xff, 0xfe],
            NSUTF32StringEncoding => &[0xff, 0xfe, 0x00, 0x00],
            _ => &[],
        };
        if !prefix.is_empty() {
            let mut external_bytes = Vec::with_capacity(prefix.len() + bytes.len());
            external_bytes.extend_from_slice(prefix);
            external_bytes.append(&mut bytes);
            bytes = external_bytes;
        }
    }

    Some(bytes)
}

#[allow(clippy::too_many_arguments)]
fn get_bytes_range_inner(
    env: &mut Environment,
    string: id,
    buffer: MutPtr<u8>,
    max_length: NSUInteger,
    used_length: MutPtr<NSUInteger>,
    encoding: NSStringEncoding,
    options: NSStringEncodingConversionOptions,
    range: NSRange,
    remaining_range: MutPtr<NSRange>,
) -> bool {
    let all_units: Vec<u16> = env
        .objc
        .borrow::<StringHostObject>(string)
        .iter_code_units()
        .collect();
    let start = range.location as usize;
    let requested = range.length as usize;
    let end = start
        .checked_add(requested)
        .expect("NSString byte-conversion range overflow");
    assert!(
        end <= all_units.len(),
        "NSString byte-conversion range out of bounds"
    );
    let units = &all_units[start..end];
    let lossy = options & NSStringEncodingConversionAllowLossy != 0;
    let external = options & NSStringEncodingConversionExternalRepresentation != 0;
    let max_length = max_length as usize;

    // Find the longest prefix that is representable in the requested encoding
    // and fits. Never split a UTF-16 surrogate pair at the buffer boundary.
    let mut converted_units = units.len();
    let bytes = loop {
        if converted_units > 0
            && converted_units < units.len()
            && (0xd800..=0xdbff).contains(&units[converted_units - 1])
            && (0xdc00..=0xdfff).contains(&units[converted_units])
        {
            converted_units -= 1;
        }
        if let Some(bytes) =
            encode_utf16_units(&units[..converted_units], encoding, lossy, external)
        {
            if bytes.len() <= max_length {
                break bytes;
            }
        }
        if converted_units == 0 {
            break Vec::new();
        }
        converted_units -= 1;
    };

    if !buffer.is_null() && !bytes.is_empty() {
        env.mem
            .bytes_at_mut(buffer, bytes.len().try_into().unwrap())
            .copy_from_slice(&bytes);
    }
    if !used_length.is_null() {
        env.mem.write(used_length, bytes.len() as NSUInteger);
    }
    if !remaining_range.is_null() {
        env.mem.write(
            remaining_range,
            NSRange {
                location: range.location + converted_units as NSUInteger,
                length: range.length - converted_units as NSUInteger,
            },
        );
    }

    converted_units == units.len()
}

/// Helper function used by
/// `[NSString stringByReplacingOccurrencesOfString:withString:options:range:]`
/// method.
fn string_by_replacing_occurrences_inner(
    env: &mut Environment,
    source: id,      // NSString *
    target: id,      // NSString *
    replacement: id, // NSString *
    options: NSStringCompareOptions,
) -> id {
    // TODO: support foreign subclasses (perhaps via a helper function that
    // copies the string first)
    let mut main_iter = env
        .objc
        .borrow::<StringHostObject>(source)
        .iter_code_units();
    let target_iter = env
        .objc
        .borrow::<StringHostObject>(target)
        .iter_code_units();
    let replacement_iter = env
        .objc
        .borrow::<StringHostObject>(replacement)
        .iter_code_units();

    // Zero-length target case
    if target_iter.clone().next().is_none() {
        let res = msg![env; source copy];
        return autorelease(env, res);
    }

    let case_insensitive = match options {
        0 => false, // No options mean literal match
        NSCaseInsensitiveSearch => true,
        _ => unimplemented!(),
    };

    let mut result: Utf16String = Vec::new();
    loop {
        if let Some(new_main_iter) = main_iter.strip_prefix(&target_iter, case_insensitive) {
            // matched target, replace it
            result.extend(replacement_iter.clone());
            main_iter = new_main_iter;
        } else {
            // no match, copy as normal
            match main_iter.next() {
                Some(cur) => result.push(cur),
                None => break,
            }
        }
    }

    // TODO: For a foreign subclass of NSString, do we have to return that
    // subclass? The signature implies this isn't the case and it's probably not
    // worth the effort, but it's an interesting question.
    let result_ns_string = msg_class![env; _touchHLE_NSString alloc];
    *env.objc.borrow_mut(result_ns_string) = StringHostObject::Utf16(result);
    autorelease(env, result_ns_string)
}
