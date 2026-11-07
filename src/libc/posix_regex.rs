/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Darwin/POSIX `regex.h` for 32-bit guests.
//!
//! The guest-visible structs follow Apple's historical ABI. Compiled host
//! regexes live in emulator state and are referenced by an opaque guest handle,
//! so no host pointer or Rust layout leaks into application memory.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, GuestUSize, MutPtr, SafeRead};
use crate::Environment;
use regex_lite::{Regex, RegexBuilder};
use std::collections::HashMap;

const REG_MAGIC: i32 = 0x7265_6765;

const REG_EXTENDED: i32 = 0o001;
const REG_ICASE: i32 = 0o002;
const REG_NOSUB: i32 = 0o004;
const REG_NEWLINE: i32 = 0o010;
const REG_PEND: i32 = 0o040;

const REG_NOTBOL: i32 = 0o001;
const REG_NOTEOL: i32 = 0o002;
const REG_STARTEND: i32 = 0o004;

const REG_NOMATCH: i32 = 1;
const REG_BADPAT: i32 = 2;
const REG_EESCAPE: i32 = 5;
const REG_INVARG: i32 = 16;
const REG_ILLSEQ: i32 = 17;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
struct regex_t {
    re_magic: i32,
    re_nsub: GuestUSize,
    re_endp: ConstPtr<u8>,
    re_g: MutPtr<u8>,
}
unsafe impl SafeRead for regex_t {}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C, packed)]
struct regmatch_t {
    rm_so: i64,
    rm_eo: i64,
}
unsafe impl SafeRead for regmatch_t {}

struct CompiledRegex {
    regex: Regex,
    no_subexpressions: bool,
}

#[derive(Default)]
pub struct State {
    compiled: HashMap<u32, CompiledRegex>,
}

fn basic_to_extended(pattern: &str) -> Result<String, i32> {
    let mut translated = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    let mut in_brackets = false;
    while let Some(character) = chars.next() {
        if character == '[' && !in_brackets {
            in_brackets = true;
            translated.push(character);
            continue;
        }
        if character == ']' && in_brackets {
            in_brackets = false;
            translated.push(character);
            continue;
        }
        if character == '\\' {
            let Some(escaped) = chars.next() else {
                return Err(REG_EESCAPE);
            };
            if !in_brackets && matches!(escaped, '(' | ')' | '{' | '}' | '+' | '?' | '|') {
                translated.push(escaped);
            } else {
                translated.push('\\');
                translated.push(escaped);
            }
            continue;
        }
        if !in_brackets && matches!(character, '(' | ')' | '{' | '}' | '+' | '?' | '|') {
            translated.push('\\');
        }
        translated.push(character);
    }
    Ok(translated)
}

fn pattern_bytes(
    env: &Environment,
    preg: MutPtr<regex_t>,
    pattern: ConstPtr<u8>,
    flags: i32,
) -> Result<Vec<u8>, i32> {
    if preg.is_null() || pattern.is_null() {
        return Err(REG_INVARG);
    }
    if flags & REG_PEND != 0 {
        let end = env.mem.read(preg).re_endp;
        if end.is_null() || end.to_bits() < pattern.to_bits() {
            return Err(REG_INVARG);
        }
        Ok(env
            .mem
            .bytes_at(pattern, end.to_bits() - pattern.to_bits())
            .to_vec())
    } else {
        Ok(env.mem.cstr_at(pattern).to_vec())
    }
}

fn regcomp(env: &mut Environment, preg: MutPtr<regex_t>, pattern: ConstPtr<u8>, flags: i32) -> i32 {
    const SUPPORTED: i32 = REG_EXTENDED | REG_ICASE | REG_NOSUB | REG_NEWLINE | REG_PEND;
    if flags & !SUPPORTED != 0 {
        return REG_INVARG;
    }
    let bytes = match pattern_bytes(env, preg, pattern, flags) {
        Ok(bytes) => bytes,
        Err(error) => return error,
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(_) => return REG_ILLSEQ,
    };
    let translated = if flags & REG_EXTENDED != 0 {
        text.to_owned()
    } else {
        match basic_to_extended(text) {
            Ok(pattern) => pattern,
            Err(error) => return error,
        }
    };

    let mut builder = RegexBuilder::new(&translated);
    builder.case_insensitive(flags & REG_ICASE != 0);
    builder.multi_line(flags & REG_NEWLINE != 0);
    // POSIX treats newline as an ordinary character unless REG_NEWLINE is set.
    builder.dot_matches_new_line(flags & REG_NEWLINE == 0);
    let regex = match builder.build() {
        Ok(regex) => regex,
        Err(_) => return REG_BADPAT,
    };

    let old = env.mem.read(preg);
    if old.re_magic == REG_MAGIC && !old.re_g.is_null() {
        if env
            .libc_state
            .posix_regex
            .compiled
            .remove(&old.re_g.to_bits())
            .is_some()
        {
            env.mem.free(old.re_g.cast());
        }
    }
    let handle: MutPtr<u8> = env.mem.alloc_and_write(0);
    let subexpressions = regex.captures_len().saturating_sub(1);
    env.libc_state.posix_regex.compiled.insert(
        handle.to_bits(),
        CompiledRegex {
            regex,
            no_subexpressions: flags & REG_NOSUB != 0,
        },
    );
    env.mem.write(
        preg,
        regex_t {
            re_magic: REG_MAGIC,
            re_nsub: subexpressions.try_into().unwrap(),
            re_endp: old.re_endp,
            re_g: handle,
        },
    );
    0
}

fn regexec(
    env: &mut Environment,
    preg: ConstPtr<regex_t>,
    string: ConstPtr<u8>,
    nmatch: GuestUSize,
    pmatch: MutPtr<regmatch_t>,
    flags: i32,
) -> i32 {
    if preg.is_null() || string.is_null() || flags & !(REG_NOTBOL | REG_NOTEOL | REG_STARTEND) != 0
    {
        return REG_INVARG;
    }
    let header = env.mem.read(preg);
    if header.re_magic != REG_MAGIC || header.re_g.is_null() {
        return REG_INVARG;
    }
    let Some(compiled) = env
        .libc_state
        .posix_regex
        .compiled
        .get(&header.re_g.to_bits())
    else {
        return REG_INVARG;
    };

    let (bytes, range_start, range_end) = if flags & REG_STARTEND != 0 {
        if nmatch == 0 || pmatch.is_null() {
            return REG_INVARG;
        }
        let requested = env.mem.read(pmatch);
        if requested.rm_so < 0 || requested.rm_eo < requested.rm_so {
            return REG_INVARG;
        }
        let start: usize = requested.rm_so.try_into().unwrap();
        let end: usize = requested.rm_eo.try_into().unwrap();
        (
            env.mem.bytes_at(string, end.try_into().unwrap()).to_vec(),
            start,
            end,
        )
    } else {
        let bytes = env.mem.cstr_at(string).to_vec();
        let end = bytes.len();
        (bytes, 0, end)
    };
    let slice = &bytes[range_start..range_end];
    let text = match std::str::from_utf8(slice) {
        Ok(text) => text,
        Err(_) => return REG_ILLSEQ,
    };

    // A non-newline sentinel prevents ^/$ from treating the selected range as
    // a boundary when REG_NOTBOL/REG_NOTEOL asks otherwise. Capture offsets are
    // translated back to the caller's original byte string below.
    let prefix = usize::from(flags & REG_NOTBOL != 0);
    let mut adjusted = String::with_capacity(text.len() + 2);
    if prefix != 0 {
        adjusted.push('x');
    }
    adjusted.push_str(text);
    if flags & REG_NOTEOL != 0 {
        adjusted.push('x');
    }
    let Some(captures) = compiled.regex.captures_at(&adjusted, prefix) else {
        return REG_NOMATCH;
    };
    let Some(full_match) = captures.get(0) else {
        return REG_NOMATCH;
    };
    if full_match.start() < prefix || full_match.end() > prefix + text.len() {
        return REG_NOMATCH;
    }

    if !compiled.no_subexpressions && nmatch != 0 && !pmatch.is_null() {
        for index in 0..nmatch {
            let value = captures
                .get(index as usize)
                .and_then(|matched| {
                    let start = matched.start().checked_sub(prefix)?;
                    let end = matched.end().checked_sub(prefix)?;
                    if end > text.len() {
                        return None;
                    }
                    Some(regmatch_t {
                        rm_so: (range_start + start).try_into().unwrap(),
                        rm_eo: (range_start + end).try_into().unwrap(),
                    })
                })
                .unwrap_or(regmatch_t {
                    rm_so: -1,
                    rm_eo: -1,
                });
            env.mem.write(pmatch + index, value);
        }
    }
    0
}

fn error_text(error: i32) -> &'static str {
    match error {
        0 => "No error",
        REG_NOMATCH => "regexec() failed to match",
        REG_BADPAT => "invalid regular expression",
        REG_EESCAPE => "trailing backslash (\\)",
        REG_INVARG => "invalid argument to regex routine",
        REG_ILLSEQ => "illegal byte sequence",
        _ => "regular expression error",
    }
}

fn regerror(
    env: &mut Environment,
    error: i32,
    _preg: ConstPtr<regex_t>,
    buffer: MutPtr<u8>,
    size: GuestUSize,
) -> GuestUSize {
    let message = error_text(error).as_bytes();
    let required: GuestUSize = (message.len() + 1).try_into().unwrap();
    if size != 0 && !buffer.is_null() {
        let copied = message.len().min(size.saturating_sub(1) as usize);
        env.mem
            .bytes_at_mut(buffer, copied.try_into().unwrap())
            .copy_from_slice(&message[..copied]);
        env.mem.write(buffer + copied as u32, 0);
    }
    required
}

fn regfree(env: &mut Environment, preg: MutPtr<regex_t>) {
    if preg.is_null() {
        return;
    }
    let header = env.mem.read(preg);
    if header.re_magic == REG_MAGIC && !header.re_g.is_null() {
        if env
            .libc_state
            .posix_regex
            .compiled
            .remove(&header.re_g.to_bits())
            .is_some()
        {
            env.mem.free(header.re_g.cast());
        }
    }
    env.mem.write(preg, regex_t::default());
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(regcomp(_, _, _)),
    export_c_func!(regexec(_, _, _, _, _)),
    export_c_func!(regerror(_, _, _, _)),
    export_c_func!(regfree(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::{guest_size_of, Ptr};

    #[test]
    fn darwin_32_bit_layout_and_extended_matching_are_preserved() {
        assert_eq!(guest_size_of::<regex_t>(), 16);
        assert_eq!(guest_size_of::<regmatch_t>(), 16);

        let mut env = Environment::new_fake_for_test();
        let pattern = env.mem.alloc_and_write_cstr(br"^[0-9A-F]{8}-[0-9A-F]{4}$");
        let value = env.mem.alloc_and_write_cstr(b"12ab34CD-5678");
        let regex = env.mem.alloc_and_write(regex_t::default());
        let matches = env.mem.alloc(guest_size_of::<regmatch_t>()).cast();

        assert_eq!(
            regcomp(
                &mut env,
                regex,
                pattern.cast_const(),
                REG_EXTENDED | REG_ICASE
            ),
            0
        );
        let compiled_header = env.mem.read(regex);
        let subexpressions = compiled_header.re_nsub;
        assert_eq!(subexpressions, 0);
        assert_eq!(
            regexec(
                &mut env,
                regex.cast_const(),
                value.cast_const(),
                1,
                matches,
                0
            ),
            0
        );
        assert_eq!(
            env.mem.read(matches),
            regmatch_t {
                rm_so: 0,
                rm_eo: 13
            }
        );
        regfree(&mut env, regex);
        let freed_header = env.mem.read(regex);
        let magic = freed_header.re_magic;
        assert_eq!(magic, 0);
    }

    #[test]
    fn regex_errors_report_required_size_and_terminate_truncated_output() {
        let mut env = Environment::new_fake_for_test();
        let buffer: MutPtr<u8> = env.mem.alloc(5).cast();
        let required = regerror(&mut env, REG_NOMATCH, Ptr::null(), buffer, 5);
        assert!(required > 5);
        assert_eq!(env.mem.bytes_at(buffer.cast_const(), 5), b"rege\0");
    }

    #[test]
    fn basic_syntax_and_startend_preserve_guest_byte_offsets() {
        let mut env = Environment::new_fake_for_test();
        let pattern = env.mem.alloc_and_write_cstr(br"^\([a-z][a-z]*\)$");
        let value: MutPtr<u8> = env.mem.alloc(11).cast();
        env.mem
            .bytes_at_mut(value, 11)
            .copy_from_slice(b"xx\0alpha\0yy");
        let regex = env.mem.alloc_and_write(regex_t::default());
        let matches = env.mem.alloc(guest_size_of::<regmatch_t>() * 2).cast();
        env.mem.write(matches, regmatch_t { rm_so: 3, rm_eo: 8 });

        assert_eq!(regcomp(&mut env, regex, pattern.cast_const(), 0), 0);
        assert_eq!(
            regexec(
                &mut env,
                regex.cast_const(),
                value.cast_const(),
                2,
                matches,
                REG_STARTEND,
            ),
            0
        );
        assert_eq!(env.mem.read(matches), regmatch_t { rm_so: 3, rm_eo: 8 });
        assert_eq!(env.mem.read(matches + 1), regmatch_t { rm_so: 3, rm_eo: 8 });
        regfree(&mut env, regex);
    }
}
