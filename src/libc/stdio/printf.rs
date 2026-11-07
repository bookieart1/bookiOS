/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `printf` function family. The implementation is also used by `NSLog` etc.

use crate::abi::{DotDotDot, VaList};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{ns_string, unichar};
use crate::libc::clocale::{setlocale, LC_CTYPE};
use crate::libc::errno::set_errno;
use crate::libc::posix_io::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::libc::stdio::{fwrite, getc, ungetc, EOF, FILE};
use crate::libc::stdlib::{atof_inner_generic, str_to_int_inner_generic};
use crate::libc::string::strlen;
use crate::libc::wchar::wchar_t;
use crate::mem::{guest_size_of, ConstPtr, GuestUSize, Mem, MutPtr, MutVoidPtr, Ptr};
use crate::objc::{id, msg, nil};
use crate::Environment;
use std::collections::HashSet;
use std::io::Write;

const ALL_SPECIFIERS: [u8; 25] = [
    // IEEE printf specification
    b'd', b'i', b'o', b'u', b'x', b'X', b'f', b'F', b'e', b'E', b'g', b'G', b'a', b'A', b'c', b's',
    b'p', b'n', b'C', b'S', b'%', // NSString formatting
    b'@', b'D', b'U', b'O',
];

const INTEGER_SPECIFIERS: [u8; 6] = *b"diouxX";
const FLOAT_SPECIFIERS: [u8; 3] = *b"feg";

/// String formatting implementation for `printf` and `NSLog` function families.
///
/// `NS_LOG` is [true] for the `NSLog` format string type, or [false] for the
/// `printf` format string type.
///
/// `get_format_char` is a callback that returns the byte at a given index in
/// the format string, or `'\0'` if the index is one past the last byte.
pub fn printf_inner<const NS_LOG: bool, F: Fn(&Mem, GuestUSize) -> u8>(
    env: &mut Environment,
    get_format_char: F,
    mut args: VaList,
) -> Vec<u8> {
    let mut res = Vec::<u8>::new();

    let mut format_char_idx = 0;

    loop {
        let c = get_format_char(&env.mem, format_char_idx);
        format_char_idx += 1;

        if c == b'\0' {
            break;
        }
        if c != b'%' {
            res.push(c);
            continue;
        }

        // Flags can come in any order.
        let mut prepend_sign = false;
        let mut left_justified = false;
        let mut pad_char = ' ';
        let mut alternative_form = false;
        loop {
            match get_format_char(&env.mem, format_char_idx) {
                b'+' => prepend_sign = true,
                b'-' => left_justified = true,
                b'0' => pad_char = '0',
                b'#' => alternative_form = true,
                _ => break,
            }
            format_char_idx += 1;
        }

        let mut pad_width = if get_format_char(&env.mem, format_char_idx) == b'*' {
            let pad_width = args.next::<i32>(env);
            format_char_idx += 1;
            pad_width
        } else {
            let mut pad_width: i32 = 0;
            while let c @ b'0'..=b'9' = get_format_char(&env.mem, format_char_idx) {
                pad_width = pad_width * 10 + (c - b'0') as i32;
                format_char_idx += 1;
            }
            pad_width
        };
        // A negative width from '*' means left-justified.
        if pad_width < 0 {
            left_justified = true;
            pad_width = -pad_width;
        }
        // Left-justified output is formatted without a width and padded with
        // spaces on the right afterwards. '-' overrides '0'.
        let right_pad_width = if left_justified {
            pad_char = ' ';
            std::mem::take(&mut pad_width) as usize
        } else {
            0
        };

        let precision = if get_format_char(&env.mem, format_char_idx) == b'.' {
            format_char_idx += 1;
            let precision = if get_format_char(&env.mem, format_char_idx) == b'*' {
                let precision = args.next::<i32>(env);
                assert!(precision >= 0); // TODO: ignore negative
                format_char_idx += 1;
                precision as usize
            } else {
                let mut precision = 0;
                while let c @ b'0'..=b'9' = get_format_char(&env.mem, format_char_idx) {
                    precision = precision * 10 + (c - b'0') as usize;
                    format_char_idx += 1;
                }
                precision
            };
            Some(precision)
        } else {
            None
        };

        let length_modifier = match get_format_char(&env.mem, format_char_idx) {
            b'h' => {
                format_char_idx += 1;
                if get_format_char(&env.mem, format_char_idx) == b'h' {
                    format_char_idx += 1;
                    Some("hh")
                } else {
                    Some("h")
                }
            }
            b'l' => {
                format_char_idx += 1;
                if get_format_char(&env.mem, format_char_idx) == b'l' {
                    format_char_idx += 1;
                    Some("ll")
                } else {
                    Some("l")
                }
            }
            // q seems to be an equivalent of 'll'
            // https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Strings/Articles/formatSpecifiers.html#//apple_ref/doc/uid/TP40004265-SW1
            b'q' => {
                format_char_idx += 1;
                Some("ll")
            }
            b'j' | b'z' | b't' | b'L' => unimplemented!(),
            _ => None,
        };

        let specifier = get_format_char(&env.mem, format_char_idx);
        format_char_idx += 1;

        if !ALL_SPECIFIERS.contains(&specifier) {
            // According to `printf` specs, this behaviour is undefined.
            // But as seen on both macOS and iOS, the '%' just got skipped.
            // Also, we need to back-track 1 position
            format_char_idx -= 1;
            continue;
        }

        if specifier == b'\0' {
            // Apparently, errno is not set in this case (tested on macOS),
            // thus we treat this situation as a normal
            // and just stop the formatting.
            assert_eq!(b'%', get_format_char(&env.mem, format_char_idx - 2));
            log!("printf_inner encountered '%' at the end of format string, ignoring.");
            break;
        }
        if specifier == b'%' {
            res.push(b'%');
            continue;
        }

        if precision.is_some() {
            assert!(
                INTEGER_SPECIFIERS.contains(&specifier)
                    || FLOAT_SPECIFIERS.contains(&specifier)
                    || specifier == b's'
            )
        }

        // The alternative form makes no difference to %d and %i. For the
        // implemented floating-point conversions it retains a decimal point
        // even when the requested precision has no fractional digits.
        assert!(
            !alternative_form || matches!(specifier, b'd' | b'i' | b'e' | b'f' | b'g'),
            "TODO: '#' with %{}",
            specifier as char,
        );

        let formatted_start = res.len();
        match specifier {
            // Integer specifiers
            b'c' => {
                assert!(!prepend_sign);
                // TODO: support length modifier
                assert!(length_modifier.is_none());
                let c: u8 = args.next(env);
                assert!(pad_char == ' ' && pad_width == 0); // TODO
                res.push(c);
            }
            // Apple extension? Seemingly works in both NSLog and printf.
            b'C' => {
                assert!(!prepend_sign);
                assert!(length_modifier.is_none());
                let c: unichar = args.next(env);
                // TODO
                assert!(pad_char == ' ' && pad_width == 0);
                // This will panic if it's a surrogate! This isn't good if
                // targeting UTF-16 ([NSString stringWithFormat:] etc).
                let c = char::from_u32(c.into()).unwrap();
                write!(&mut res, "{c}").unwrap();
            }
            b's' | b'S' => {
                assert!(!prepend_sign);
                if length_modifier == Some("l") || specifier == b'S' {
                    if specifier == b'S' {
                        assert!(length_modifier.is_none());
                    }
                    assert!(precision.is_none());
                    let ctype_locale = setlocale(env, LC_CTYPE, Ptr::null());
                    assert_eq!(env.mem.read(ctype_locale), b'C');
                    let w_string: ConstPtr<wchar_t> = args.next(env);
                    assert!(pad_char == ' ' && pad_width == 0); // TODO
                    if !w_string.is_null() {
                        res.extend_from_slice(env.mem.wcstr_at(w_string).as_bytes());
                    } else {
                        res.extend_from_slice("(null)".as_bytes());
                    }
                } else {
                    assert!(length_modifier.is_none()); // TODO
                    let c_string: ConstPtr<u8> = args.next(env);
                    // Darwin accepts the otherwise-meaningless `0` flag for
                    // strings and pads with spaces. Some older games generate
                    // formats such as "%05s" dynamically.
                    if !c_string.is_null() {
                        if let Some(precision) = precision {
                            let str_len = strlen(env, c_string);
                            res.extend_from_slice(
                                env.mem.bytes_at(c_string, str_len.min(precision as _)),
                            )
                        } else if pad_width > 0 {
                            let pad_width = pad_width as usize;
                            let str = env.mem.cstr_at_utf8(c_string).unwrap();
                            write!(&mut res, "{str:>pad_width$}").unwrap();
                        } else {
                            res.extend_from_slice(env.mem.cstr_at(c_string));
                        }
                    } else {
                        assert!(precision.is_none());
                        res.extend_from_slice("(null)".as_bytes());
                    }
                }
            }
            b'd' | b'i' | b'u' => {
                // Note: on 32-bit system int and long are i32,
                // so single length_modifier is ignored (but not double one!)
                let int: i64 = if specifier == b'u' {
                    if length_modifier == Some("ll") {
                        let uint: u64 = args.next(env);
                        uint.try_into().unwrap()
                    } else if length_modifier == Some("hh") {
                        let uint: u8 = args.next(env);
                        uint.into()
                    } else if length_modifier == Some("h") {
                        let uint: u16 = args.next(env);
                        uint.into()
                    } else {
                        assert!(length_modifier.is_none() || length_modifier == Some("l"));
                        let uint: u32 = args.next(env);
                        uint.into()
                    }
                } else if length_modifier == Some("ll") {
                    args.next(env)
                } else if length_modifier == Some("hh") {
                    let int: i8 = args.next(env);
                    int.into()
                } else if length_modifier == Some("h") {
                    let int: i16 = args.next(env);
                    int.into()
                } else {
                    assert!(length_modifier.is_none() || length_modifier == Some("l"));
                    let int: i32 = args.next(env);
                    int.into()
                };

                let int_with_precision = if precision.is_some_and(|value| value > 0) {
                    format!("{:01$}", int, precision.unwrap())
                } else {
                    format!("{int}")
                };

                if pad_width > 0 {
                    let pad_width = pad_width as usize;
                    if pad_char == '0' && precision.is_none() {
                        if prepend_sign {
                            assert!(pad_width > 0);
                            if int >= 0 {
                                write!(&mut res, "+{:0>1$}", int, pad_width - 1).unwrap();
                            } else {
                                write!(&mut res, "-{:0>1$}", int.abs(), pad_width - 1).unwrap();
                            }
                        } else {
                            write!(&mut res, "{int:0>pad_width$}").unwrap();
                        }
                    } else {
                        assert!(!prepend_sign);
                        write!(&mut res, "{int_with_precision:>pad_width$}").unwrap();
                    }
                } else {
                    assert!(!prepend_sign);
                    res.extend_from_slice(int_with_precision.as_bytes());
                }
            }
            b'@' if NS_LOG => {
                assert!(!prepend_sign);
                assert!(length_modifier.is_none());
                let object: id = args.next(env);
                // TODO: use localized description if available?
                let description: id = msg![env; object description];
                if description != nil {
                    // TODO: avoid copy
                    // TODO: what if the description isn't valid UTF-16?
                    let description = ns_string::to_rust_string(env, description);
                    write!(&mut res, "{description}").unwrap();
                } else {
                    write!(&mut res, "(null)").unwrap();
                }
            }
            b'x' => {
                assert!(!prepend_sign);
                // Note: on 32-bit system unsigned int and unsigned long
                // are u32, so length_modifier is ignored
                let uint: u32 = if length_modifier == Some("ll") {
                    let uint: u64 = args.next(env);
                    uint.try_into().unwrap()
                } else if length_modifier == Some("hh") {
                    let uint: u8 = args.next(env);
                    uint.into()
                } else if length_modifier == Some("h") {
                    let uint: u16 = args.next(env);
                    uint.into()
                } else {
                    assert!(length_modifier.is_none() || length_modifier == Some("l"));
                    let uint: u32 = args.next(env);
                    uint
                };
                if pad_width > 0 {
                    assert!(precision.is_none()); // TODO
                    let pad_width = pad_width as usize;
                    if pad_char == '0' && precision.is_none() {
                        write!(&mut res, "{uint:0>pad_width$x}").unwrap();
                    } else {
                        write!(&mut res, "{uint:>pad_width$x}").unwrap();
                    }
                } else {
                    let tmp = if precision.is_some_and(|value| value > 0) {
                        format!("{:01$x}", uint, precision.unwrap())
                    } else {
                        if let Some(precision) = precision {
                            assert!(precision == 0 && uint != 0); // TODO
                        }
                        format!("{uint:x}")
                    };
                    res.extend_from_slice(tmp.as_bytes());
                }
            }
            b'X' => {
                assert!(!prepend_sign);
                assert!(precision.is_none());
                // Note: on 32-bit system unsigned int and unsigned long
                // are u32, so length_modifier is ignored
                let uint: u32 = if length_modifier == Some("ll") {
                    let uint: u64 = args.next(env);
                    uint.try_into().unwrap()
                } else if length_modifier == Some("hh") {
                    let uint: u8 = args.next(env);
                    uint.into()
                } else if length_modifier == Some("h") {
                    let uint: u16 = args.next(env);
                    uint.into()
                } else {
                    assert!(length_modifier.is_none() || length_modifier == Some("l"));
                    let uint: u32 = args.next(env);
                    uint
                };
                if pad_width > 0 {
                    let pad_width = pad_width as usize;
                    if pad_char == '0' && precision.is_none() {
                        write!(&mut res, "{uint:0>pad_width$X}").unwrap();
                    } else {
                        assert!(pad_char == ' '); // TODO
                        write!(&mut res, "{uint:>pad_width$X}").unwrap();
                    }
                } else {
                    res.extend_from_slice(format!("{uint:X}").as_bytes());
                }
            }
            b'p' => {
                assert!(!prepend_sign);
                assert!(length_modifier.is_none());
                let ptr: MutVoidPtr = args.next(env);
                // '%p' is implementation defined,
                // but this matches iOS simulator output
                let tmp = format!("{:#x}", ptr.to_bits());
                if pad_width > 0 {
                    let pad_width = pad_width as usize;
                    assert!(pad_char == ' '); // TODO
                    write!(&mut res, "{tmp:>pad_width$}").unwrap();
                } else {
                    res.extend_from_slice(tmp.as_bytes());
                }
            }
            // Float specifiers
            b'f' => {
                let float: f64 = args.next(env);
                let pad_width = pad_width as usize;
                let precision = precision.unwrap_or(6);

                let formatted = f_format(
                    float,
                    pad_width,
                    pad_char,
                    precision,
                    prepend_sign,
                    alternative_form,
                );
                res.extend_from_slice(formatted.as_bytes());
            }
            b'e' => {
                let float: f64 = args.next(env);
                let pad_width = pad_width as usize;
                let precision = precision.unwrap_or(6);

                let formatted = e_format(
                    float,
                    pad_width,
                    pad_char,
                    precision,
                    prepend_sign,
                    alternative_form,
                );
                res.extend_from_slice(formatted.as_bytes());
            }
            b'g' => {
                let float: f64 = args.next(env);
                let pad_width = pad_width as usize;

                // Reference https://en.cppreference.com/w/c/io/vfprintf
                let P: i32 = if let Some(precision) = precision {
                    if precision == 0 {
                        1
                    } else {
                        precision.try_into().unwrap()
                    }
                } else {
                    6
                };
                let X: i32 = if float == 0.0 {
                    0
                } else {
                    float.abs().log10().floor() as i32
                };
                log_dbg!(
                    "float {}, pad_width {}, pad_char '{}', P {}, X {}",
                    float,
                    pad_width,
                    pad_char,
                    P,
                    X
                );
                if P > X && X >= -4 {
                    let precision: usize = (P - X - 1).try_into().unwrap();

                    let result = f_format(
                        float,
                        pad_width,
                        pad_char,
                        precision,
                        prepend_sign,
                        alternative_form,
                    );
                    let trimmed_result = if result.contains('.') && !alternative_form {
                        result.trim_end_matches('0').trim_end_matches('.')
                    } else {
                        &result
                    };

                    let trimmed_result = repad_float(trimmed_result, pad_width, pad_char);

                    res.extend_from_slice(trimmed_result.as_bytes());
                } else {
                    let precision: usize = (P - 1).try_into().unwrap();

                    let formatted = e_format(
                        float,
                        pad_width,
                        pad_char,
                        precision,
                        prepend_sign,
                        alternative_form,
                    );
                    res.extend_from_slice(formatted.as_bytes());
                }
            }
            // TODO: more specifiers
            _ => unimplemented!(
                "Format character '{}'. Formatted up to index {}",
                specifier as char,
                format_char_idx
            ),
        }
        let formatted_len = res.len() - formatted_start;
        if formatted_len < right_pad_width {
            res.resize(formatted_start + right_pad_width, b' ');
        }
    }

    log_dbg!("=> {:?}", std::str::from_utf8(&res));

    res
}

fn pad_float(
    magnitude: String,
    negative: bool,
    prepend_sign: bool,
    pad_width: usize,
    pad_char: char,
) -> String {
    let sign = if negative {
        "-"
    } else if prepend_sign {
        "+"
    } else {
        ""
    };
    let padding = pad_width.saturating_sub(sign.len() + magnitude.len());

    match pad_char {
        '0' => format!("{sign}{}{magnitude}", "0".repeat(padding)),
        ' ' => format!("{}{sign}{magnitude}", " ".repeat(padding)),
        _ => unreachable!("unsupported printf padding character"),
    }
}

fn repad_float(formatted: &str, pad_width: usize, pad_char: char) -> String {
    let padding = pad_width.saturating_sub(formatted.len());
    if padding == 0 {
        return formatted.to_string();
    }

    match pad_char {
        '0' if formatted.starts_with('+') || formatted.starts_with('-') => format!(
            "{}{}{}",
            &formatted[..1],
            "0".repeat(padding),
            &formatted[1..]
        ),
        '0' => format!("{}{formatted}", "0".repeat(padding)),
        ' ' => format!("{}{formatted}", " ".repeat(padding)),
        _ => unreachable!("unsupported printf padding character"),
    }
}

fn f_format(
    float: f64,
    pad_width: usize,
    pad_char: char,
    precision: usize,
    prepend_sign: bool,
    alternative_form: bool,
) -> String {
    let mut magnitude = format!("{:.*}", precision, float.abs());
    if alternative_form && !magnitude.contains('.') {
        magnitude.push('.');
    }
    pad_float(
        magnitude,
        float.is_sign_negative(),
        prepend_sign,
        pad_width,
        pad_char,
    )
}

fn e_format(
    float: f64,
    pad_width: usize,
    pad_char: char,
    precision: usize,
    prepend_sign: bool,
    alternative_form: bool,
) -> String {
    let exponent = if float == 0.0 {
        0.0
    } else {
        float.abs().log10().floor()
    };
    let mantissa = float.abs() / 10f64.powf(exponent);
    let mut mantissa = format!("{mantissa:.precision$}");
    if alternative_form && !mantissa.contains('.') {
        mantissa.push('.');
    }
    let magnitude = format!("{mantissa}e{exponent:+03}");
    pad_float(
        magnitude,
        float.is_sign_negative(),
        prepend_sign,
        pad_width,
        pad_char,
    )
}

#[cfg(test)]
mod tests {
    use super::{e_format, f_format, format_wide, repad_float, sscanf_common};
    use crate::abi::VaList;
    use crate::mem::ConstVoidPtr;
    use crate::Environment;

    #[test]
    fn float_plus_flag_preserves_sign_and_width() {
        assert_eq!(f_format(1.25, 0, ' ', 2, true, false), "+1.25");
        assert_eq!(f_format(-1.25, 0, ' ', 2, true, false), "-1.25");
        assert_eq!(f_format(1.25, 8, ' ', 2, true, false), "   +1.25");
        assert_eq!(f_format(1.25, 8, '0', 2, true, false), "+0001.25");
        assert_eq!(f_format(-1.25, 8, '0', 2, true, false), "-0001.25");
        assert_eq!(f_format(1.25, 8, '0', 2, false, false), "00001.25");
        assert_eq!(f_format(10.0, 0, ' ', 0, false, true), "10.");
        assert_eq!(repad_float("+1.25", 8, '0'), "+0001.25");
        assert_eq!(repad_float("-1.25", 8, '0'), "-0001.25");
    }

    #[test]
    fn exponential_plus_flag_preserves_sign_and_width() {
        assert_eq!(e_format(12.5, 0, ' ', 1, true, false), "+1.2e+01");
        assert_eq!(e_format(-12.5, 0, ' ', 1, true, false), "-1.2e+01");
        assert_eq!(e_format(12.5, 11, ' ', 1, true, false), "   +1.2e+01");
        assert_eq!(e_format(12.5, 11, '0', 1, true, false), "+0001.2e+01");
        assert_eq!(e_format(12.5, 0, ' ', 0, false, true), "1.e+01");
        assert_eq!(e_format(12.5, 9, '0', 0, true, true), "+001.e+01");
    }

    #[test]
    fn dev_244_wide_printf_uses_guest_arguments() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let wide = env.mem.alloc(4 * 9).cast();
        for (index, value) in "score=%d\0".encode_utf16().enumerate() {
            env.mem
                .write(wide + index.try_into().unwrap(), i32::from(value));
        }
        env.cpu.regs_mut()[0] = 42;
        let args = VaList::for_test(0, ConstVoidPtr::from_bits(0));
        assert_eq!(format_wide(&mut env, wide.cast_const(), args), b"score=42");
    }

    #[test]
    fn dev_245_sscanf_ll_writes_guest_64_bit_integer() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let source = env.mem.alloc_and_write_cstr(b"-4294967297");
        let format = env.mem.alloc_and_write_cstr(b"%lld");
        let destination = env.mem.alloc_and_write(0_i64);
        env.cpu.regs_mut()[0] = destination.to_bits();
        let args = VaList::for_test(0, ConstVoidPtr::from_bits(0));
        assert_eq!(
            sscanf_common(&mut env, source.cast_const(), format.cast_const(), args),
            1
        );
        assert_eq!(env.mem.read(destination), -4_294_967_297_i64);
        env.mem.free(destination.cast());
        env.mem.free(format.cast());
        env.mem.free(source.cast());
    }
}

fn snprintf(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!("snprintf() implemented as a wrapper of vsnprintf()");

    vsnprintf(env, dest, n, format, args.start())
}

fn vasprintf(
    env: &mut Environment,
    ret: MutPtr<MutPtr<u8>>,
    format: ConstPtr<u8>,
    arg: VaList,
) -> i32 {
    log_dbg!(
        "vasprintf({:?}, {:?} ({:?}), ...)",
        ret,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    let count: GuestUSize = (res.len() + 1).try_into().unwrap();

    let dest: MutPtr<u8> = env.mem.alloc(count * guest_size_of::<u8>()).cast();

    let dest_slice = env.mem.bytes_at_mut(dest, count);
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    env.mem.write(ret, dest);

    res.len().try_into().unwrap()
}

fn vprintf(env: &mut Environment, format: ConstPtr<u8>, arg: VaList) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "vprintf({:?} ({:?}), ...)",
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    // TODO: I/O error handling
    let _ = std::io::stdout().write_all(&res);
    res.len().try_into().unwrap()
}

fn format_wide(env: &mut Environment, format: ConstPtr<wchar_t>, args: VaList) -> Vec<u8> {
    let ctype_locale = setlocale(env, LC_CTYPE, Ptr::null());
    assert_eq!(env.mem.read(ctype_locale), b'C');
    let format = env.mem.wcstr_at(format);
    let bytes = format.as_bytes();
    let len: GuestUSize = bytes.len().try_into().unwrap();
    printf_inner::<false, _>(
        env,
        |_mem, idx| {
            if idx == len {
                b'\0'
            } else {
                bytes[idx as usize]
            }
        },
        args,
    )
}

fn vwprintf(env: &mut Environment, format: ConstPtr<wchar_t>, args: VaList) -> i32 {
    set_errno(env, 0);
    log_dbg!(
        "vwprintf({:?} ({:?}), ...)",
        format,
        env.mem.wcstr_at(format)
    );
    let result = format_wide(env, format, args);
    let _ = std::io::stdout().write_all(&result);
    result.len().try_into().unwrap()
}

fn vsnprintf(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    format: ConstPtr<u8>,
    arg: VaList,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "vsnprintf({:?} {:?} {:?})",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    if n == 0 {
        return res.len().try_into().unwrap();
    }
    let middle = if ((n - 1) as usize) < res.len() {
        &res[..(n - 1) as usize]
    } else {
        &res[..]
    };

    let dest_slice = env.mem.bytes_at_mut(dest, n);
    for (i, &byte) in middle.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn vsnprintf_l(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    _locale: MutVoidPtr,
    format: ConstPtr<u8>,
    arg: VaList,
) -> i32 {
    // Formatting is currently locale-independent, so this has the same
    // behavior as vsnprintf while still providing the Darwin API.
    vsnprintf(env, dest, n, format, arg)
}

fn vsprintf(env: &mut Environment, dest: MutPtr<u8>, format: ConstPtr<u8>, arg: VaList) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "vsprintf({:?}, {:?} ({:?}), ...)",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);

    let dest_slice = env
        .mem
        .bytes_at_mut(dest, (res.len() + 1).try_into().unwrap());
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn __sprintf_chk(
    env: &mut Environment,
    dest: MutPtr<u8>,
    _flags: i32,
    strlen: GuestUSize,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    if strlen == 0 {
        panic!();
    }
    // TODO: respect flags level
    // TODO: full overflow check
    sprintf(env, dest, format, args)
}

fn __vsnprintf_chk(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    _flags: i32,
    dest_len: GuestUSize,
    format: ConstPtr<u8>,
    args: VaList,
) -> i32 {
    // Darwin's checked variant guarantees that the caller cannot write beyond
    // the compiler-known destination size. Preserve that bound even for old
    // applications whose requested length is larger than the object itself.
    let bounded_n = n.min(dest_len);
    vsnprintf(env, dest, bounded_n, format, args)
}

fn __snprintf_chk(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    flags: i32,
    dest_len: GuestUSize,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    __vsnprintf_chk(env, dest, n, flags, dest_len, format, args.start())
}

fn sprintf(env: &mut Environment, dest: MutPtr<u8>, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "sprintf({:?}, {:?} ({:?}), ...)",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), args.start());

    let dest_slice = env
        .mem
        .bytes_at_mut(dest, (res.len() + 1).try_into().unwrap());
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn swprintf(
    env: &mut Environment,
    ws: MutPtr<wchar_t>,
    n: GuestUSize,
    format: ConstPtr<wchar_t>,
    args: DotDotDot,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!("swprintf() implemented as a wrapper of vswprintf()");

    vswprintf(env, ws, n, format, args.start())
}

fn vswprintf(
    env: &mut Environment,
    ws: MutPtr<wchar_t>,
    n: GuestUSize,
    format: ConstPtr<wchar_t>,
    args: VaList,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    let wcstr_format = env.mem.wcstr_at(format);
    log_dbg!(
        "vswprintf({:?}, {}, {:?} ({:?}), ...)",
        ws,
        n,
        format,
        wcstr_format
    );

    let res = format_wide(env, format, args);

    let to_write = n.min(res.len() as GuestUSize);
    for i in 0..to_write {
        env.mem.write(ws + i, res[i as usize] as wchar_t);
    }
    if to_write >= n {
        // TODO: set errno
        return -1;
    }
    env.mem.write(ws + to_write, wchar_t::default());
    to_write as i32
}

fn printf(env: &mut Environment, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "printf({:?} ({:?}), ...)",
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), args.start());
    // TODO: I/O error handling
    let _ = std::io::stdout().write_all(&res);
    res.len().try_into().unwrap()
}

// TODO: more printf variants

/// A simple wrapper around [sscanf_common_generic] for the case of C string.
fn sscanf_common(
    env: &mut Environment,
    src: ConstPtr<u8>,
    format: ConstPtr<u8>,
    args: VaList,
) -> i32 {
    sscanf_common_generic(
        env,
        |env, s, idx| match env.mem.read(s + idx) {
            b'\0' => Err(()),
            c => Ok(c),
        },
        |_, _, _| (),
        src.cast_mut(),
        format,
        args,
    )
}

/// Formatted scan implementation for `sscanf` family.
///
/// `getc_fn` is a callback to get next character from `subject`.
/// 3rd parameter in this callback is a index which is safe to ignore
/// (for example, in case of a file stream).
/// Error signifies an abnormal stop of input,
/// such as [crate::libc::stdio::EOF] in the file stream.
/// Note: `'\0'` does not necessary expect to produce an error!
///
/// `ungetc_fn` is a callback to un-get character from `subject`.
/// Could be ignored entirely (for example, in case of a string).
///
/// `subject` is either C string or file stream (for now).
///
/// `args` is the list of arguments to store produced outputs.
///
/// TODO: instead of `u8: From<T>` constraint, implement a conversion callback
fn sscanf_common_generic<
    T,
    U,
    F1: Fn(&mut Environment, MutPtr<U>, GuestUSize) -> Result<T, ()>,
    F2: Fn(&mut Environment, MutPtr<U>, u8), // TODO: make last param generic too?
>(
    env: &mut Environment,
    getc_fn: F1,
    ungetc_fn: F2,
    subject: MutPtr<U>,
    format: ConstPtr<u8>,
    mut args: VaList,
) -> i32
where
    u8: From<T>,
{
    let mut src_char_idx = 0;
    let mut format_char_idx = 0;

    let mut matched_args = 0;
    let mut matched_conversion = false;

    'outer: loop {
        let c = env.mem.read(format + format_char_idx);
        format_char_idx += 1;

        if c == b'\0' {
            break;
        }
        if c != b'%' {
            let mut cc: u8 = match getc_fn(env, subject, src_char_idx) {
                Ok(c) => c.into(),
                Err(_) => break 'outer,
            };
            if isspace(env, format + format_char_idx - 1) {
                // "any single whitespace character in the format string
                // consumes all available consecutive whitespace characters
                // from the input"
                while isspace_inner(cc) {
                    src_char_idx += 1;
                    cc = match getc_fn(env, subject, src_char_idx) {
                        Ok(c) => c.into(),
                        Err(_) => break 'outer,
                    };
                }
                // backtrack one
                ungetc_fn(env, subject, cc);
                continue;
            }
            if c != cc {
                // matching failure, backtrack one
                ungetc_fn(env, subject, cc);
                return matched_args;
            }
            src_char_idx += 1;
            continue;
        }

        let assignment_suppressed = if env.mem.read(format + format_char_idx) == b'*' {
            format_char_idx += 1;
            true
        } else {
            false
        };
        let mut max_width: u32 = 0;
        while let c @ b'0'..=b'9' = env.mem.read(format + format_char_idx) {
            max_width = max_width * 10 + (c - b'0') as u32;
            format_char_idx += 1;
        }

        let length_modifier = match env.mem.read(format + format_char_idx) {
            b'h' => {
                format_char_idx += 1;
                if env.mem.read(format + format_char_idx) == b'h' {
                    format_char_idx += 1;
                    Some("hh")
                } else {
                    Some("h")
                }
            }
            b'l' => {
                format_char_idx += 1;
                if env.mem.read(format + format_char_idx) == b'l' {
                    format_char_idx += 1;
                    Some("ll")
                } else {
                    Some("l")
                }
            }
            // q seems to be an equivalent of 'll'
            // https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Strings/Articles/formatSpecifiers.html#//apple_ref/doc/uid/TP40004265-SW1
            b'q' => {
                format_char_idx += 1;
                Some("ll")
            }

            _ => None,
        };

        let specifier = env.mem.read(format + format_char_idx);
        format_char_idx += 1;

        if !b"[cn".contains(&specifier) {
            // skip whitespaces
            let mut cc: u8 = match getc_fn(env, subject, src_char_idx) {
                Ok(c) => c.into(),
                Err(_) => break 'outer,
            };
            while isspace_inner(cc) {
                src_char_idx += 1;
                cc = match getc_fn(env, subject, src_char_idx) {
                    Ok(c) => c.into(),
                    Err(_) => break 'outer,
                };
            }
            // backtrack one
            ungetc_fn(env, subject, cc);
        }

        match specifier {
            b'd' | b'i' => {
                let base: u32 = if specifier == b'd' {
                    10
                } else {
                    // automatic base detection in strtol
                    0
                };

                let res = str_to_int_inner_generic(
                    env,
                    &getc_fn,
                    &ungetc_fn,
                    subject,
                    src_char_idx,
                    base,
                    if max_width > 0 { max_width } else { u32::MAX },
                    |s, base| {
                        // We parse as i64 to accommodate for
                        // `signed long long` case (reserved for future use).
                        // Later, the actual conversion to target type is
                        // happening when we actually write value back.
                        i64::from_str_radix(s, base).unwrap()
                    },
                    |num| num.checked_mul(-1).unwrap(),
                );
                match res {
                    Ok((val, len)) => {
                        src_char_idx += len;
                        matched_conversion = true;
                        if !assignment_suppressed {
                            match length_modifier {
                                Some("hh") => {
                                    let c_char_ptr: ConstPtr<i8> = args.next(env);
                                    env.mem.write(c_char_ptr.cast_mut(), val as i8);
                                }
                                Some("h") => {
                                    let c_short_ptr: ConstPtr<i16> = args.next(env);
                                    env.mem.write(c_short_ptr.cast_mut(), val as i16);
                                }
                                None => {
                                    let c_int_ptr: ConstPtr<i32> = args.next(env);
                                    env.mem.write(c_int_ptr.cast_mut(), val as i32);
                                }
                                Some("l") => {
                                    // `long` remains 32-bit in the guest ABI.
                                    let c_long_ptr: ConstPtr<i32> = args.next(env);
                                    env.mem.write(c_long_ptr.cast_mut(), val as i32);
                                }
                                Some("ll") => {
                                    let c_long_long_ptr: ConstPtr<i64> = args.next(env);
                                    env.mem.write(c_long_long_ptr.cast_mut(), val);
                                }
                                _ => unimplemented!("length_modifier {:?}", length_modifier),
                            }
                            matched_args += 1;
                        }
                    }
                    Err(_) => break 'outer,
                }
            }
            b'f' | b'g' => {
                assert_eq!(max_width, 0); // TODO
                let res = atof_inner_generic(env, &getc_fn, &ungetc_fn, subject, src_char_idx);
                let val = match res {
                    Ok((val, len)) => {
                        src_char_idx += len;
                        matched_conversion = true;
                        val
                    }
                    Err(_) => break 'outer,
                };
                if !assignment_suppressed {
                    match length_modifier {
                        None => {
                            let c_int_ptr: ConstPtr<f32> = args.next(env);
                            env.mem.write(c_int_ptr.cast_mut(), val as f32);
                        }
                        Some("l") => {
                            let c_int_ptr: ConstPtr<f64> = args.next(env);
                            env.mem.write(c_int_ptr.cast_mut(), val);
                        }
                        Some(modifier) => {
                            unimplemented!("Length formater '{}' for f", modifier)
                        }
                    }
                    matched_args += 1;
                }
            }
            b'x' | b'X' | b'u' => {
                let base: u32 = match specifier {
                    b'x' | b'X' => 16,
                    b'u' => 10,
                    _ => unreachable!(),
                };
                let res = str_to_int_inner_generic(
                    env,
                    &getc_fn,
                    &ungetc_fn,
                    subject,
                    src_char_idx,
                    base,
                    if max_width > 0 { max_width } else { u32::MAX },
                    |s, base| {
                        // We parse as u64 to accommodate for
                        // `unsigned long long` case (reserved for future use).
                        // Later, the actual conversion to target type is
                        // happening when we actually write value back.
                        u64::from_str_radix(s, base).unwrap()
                    },
                    |num| num.wrapping_neg(),
                );
                match res {
                    Ok((val, len)) => {
                        src_char_idx += len;
                        matched_conversion = true;
                        if !assignment_suppressed {
                            match length_modifier {
                                Some("hh") => {
                                    let c_char_ptr: ConstPtr<u8> = args.next(env);
                                    env.mem.write(c_char_ptr.cast_mut(), val as u8);
                                }
                                Some("h") => {
                                    let c_short_ptr: ConstPtr<u16> = args.next(env);
                                    env.mem.write(c_short_ptr.cast_mut(), val as u16);
                                }
                                None => {
                                    let c_u32_ptr: ConstPtr<u32> = args.next(env);
                                    env.mem.write(c_u32_ptr.cast_mut(), val as u32);
                                }
                                Some("l") => {
                                    let c_ulong_ptr: ConstPtr<u32> = args.next(env);
                                    env.mem.write(c_ulong_ptr.cast_mut(), val as u32);
                                }
                                Some("ll") => {
                                    let c_u64_ptr: ConstPtr<u64> = args.next(env);
                                    env.mem.write(c_u64_ptr.cast_mut(), val);
                                }
                                _ => unimplemented!("length_modifier {:?}", length_modifier),
                            }
                            matched_args += 1;
                        }
                    }
                    Err(_) => break 'outer,
                }
            }
            b'[' => {
                assert!(length_modifier.is_none());
                // [set] case
                assert_ne!(env.mem.read(format + format_char_idx), b']');
                let mut c: u8;
                let inverted = if env.mem.read(format + format_char_idx) == b'^' {
                    format_char_idx += 1;
                    assert_ne!(env.mem.read(format + format_char_idx), b']');
                    true
                } else {
                    false
                };
                // Build set
                let mut set: HashSet<u8> = HashSet::new();
                c = env.mem.read(format + format_char_idx);
                format_char_idx += 1;
                while c != b']' {
                    if env.mem.read(format + format_char_idx) == b'-' {
                        assert_ne!(env.mem.read(format + format_char_idx + 1), b']');
                        let cc = env.mem.read(format + format_char_idx + 1);
                        for x in c..=cc {
                            set.insert(x);
                        }
                        format_char_idx += 2;
                    } else {
                        set.insert(c);
                    }
                    c = env.mem.read(format + format_char_idx);
                    format_char_idx += 1;
                }
                let mut dst_ptr: MutPtr<u8> = if assignment_suppressed {
                    MutPtr::null()
                } else {
                    args.next(env)
                };
                let mut matched = false;
                // Consume `src` while chars are in the set
                // (or not in the set if inverted)
                let mut cc: u8 = match getc_fn(env, subject, src_char_idx) {
                    Ok(c) => c.into(),
                    Err(_) => break 'outer,
                };
                src_char_idx += 1;
                let mut match_count = 0;
                while set.contains(&cc) ^ inverted && cc != b'\0' {
                    matched = true;
                    if !assignment_suppressed {
                        env.mem.write(dst_ptr, cc);
                        dst_ptr += 1;
                    }
                    match_count += 1;
                    if max_width > 0 && match_count == max_width {
                        break;
                    }
                    cc = match getc_fn(env, subject, src_char_idx) {
                        Ok(c) => c.into(),
                        Err(_) => break,
                    };
                    src_char_idx += 1;
                }
                if !(set.contains(&cc) ^ inverted && cc != b'\0') {
                    // We need to backtrack one position
                    ungetc_fn(env, subject, cc);
                    src_char_idx -= 1;
                }
                if matched {
                    matched_conversion = true;
                    if !assignment_suppressed {
                        env.mem.write(dst_ptr, b'\0');
                        matched_args += 1;
                    }
                } else {
                    break 'outer;
                }
            }
            b's' => {
                assert!(length_modifier.is_none());
                let orig_dst_ptr: MutPtr<u8> = if assignment_suppressed {
                    MutPtr::null()
                } else {
                    args.next(env)
                };
                let mut dst_ptr: MutPtr<u8> = orig_dst_ptr;
                let mut written = 0;
                let mut matched = false;
                loop {
                    let cc: u8 = match getc_fn(env, subject, src_char_idx) {
                        Ok(c) => c.into(),
                        Err(_) => break,
                    };
                    if !isspace_inner(cc) {
                        if cc == b'\0' {
                            break;
                        }
                        if max_width > 0 && written >= max_width {
                            ungetc_fn(env, subject, cc);
                            break;
                        }
                        matched = true;
                        if !assignment_suppressed {
                            env.mem.write(dst_ptr, cc);
                        }
                        src_char_idx += 1;
                        if !assignment_suppressed {
                            dst_ptr += 1;
                        }
                        written += 1;
                    } else {
                        ungetc_fn(env, subject, cc);
                        break;
                    }
                }
                if !matched {
                    break 'outer;
                }
                matched_conversion = true;
                if !assignment_suppressed {
                    env.mem.write(dst_ptr, b'\0');
                    matched_args += 1;
                    log_dbg!(
                        "sscanf_common_generic read %s '{:?}'",
                        env.mem.cstr_at_utf8(orig_dst_ptr)
                    );
                }
            }
            // TODO: more specifiers
            _ => unimplemented!("Format character '{}'", specifier as char),
        }
    }

    if matched_args == 0 && !matched_conversion {
        if let Ok(cc) = getc_fn(env, subject, src_char_idx) {
            // nothing matched, backtrack one
            ungetc_fn(env, subject, cc.into());
        } else {
            return EOF;
        }
    }
    matched_args
}

fn sscanf(env: &mut Environment, src: ConstPtr<u8>, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "sscanf({:?} ({:?}), {:?} ({:?}), ...)",
        src,
        env.mem.cstr_at_utf8(src),
        format,
        env.mem.cstr_at_utf8(format)
    );

    sscanf_common(env, src, format, args.start())
}

fn swscanf(
    env: &mut Environment,
    ws: ConstPtr<wchar_t>,
    format: ConstPtr<wchar_t>,
    args: DotDotDot,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    // TODO: support other locales
    let ctype_locale = setlocale(env, LC_CTYPE, Ptr::null());
    assert_eq!(env.mem.read(ctype_locale), b'C');

    let w_string = env.mem.wcstr_at(ws);
    let w_format = env.mem.wcstr_at(format);
    log_dbg!(
        "swscanf({:?} ({:?}), {:?} ({:?}), ...)",
        ws,
        w_string,
        format,
        w_format
    );
    // TODO: refactor code to parametrise sscanf_common()
    // for normal and wide strings instead
    let c_string = env.mem.alloc_and_write_cstr(w_string.as_bytes());
    let c_format = env.mem.alloc_and_write_cstr(w_format.as_bytes());
    let res = sscanf(env, c_string.cast_const(), c_format.cast_const(), args);
    env.mem.free(c_string.cast());
    env.mem.free(c_format.cast());
    res
}

fn vsscanf(env: &mut Environment, src: ConstPtr<u8>, format: ConstPtr<u8>, arg: VaList) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "vsscanf({:?}, {:?} ({:?}), ...)",
        src,
        format,
        env.mem.cstr_at_utf8(format)
    );

    sscanf_common(env, src, format, arg)
}

fn fscanf(
    env: &mut Environment,
    stream: MutPtr<FILE>,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "fscanf({:?}, {:?} ({:?}), ...)",
        stream,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let cc = getc(env, stream);
    if cc == EOF {
        return EOF;
    } else {
        assert_eq!(cc, ungetc(env, cc, stream));
    }

    sscanf_common_generic(
        env,
        |env, file, _idx| {
            let c = getc(env, file);
            if c == EOF {
                Err::<u8, ()>(())
            } else {
                Ok(<i32 as TryInto<u8>>::try_into(c).unwrap())
            }
        },
        |env, file, c| {
            assert_eq!(c as i32, ungetc(env, c as i32, file));
        },
        stream,
        format,
        args.start(),
    )
}

fn fprintf(
    env: &mut Environment,
    stream: MutPtr<FILE>,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!("fprintf() implemented as a wrapper of vfprintf()");

    vfprintf(env, stream, format, args.start())
}

fn vfprintf(env: &mut Environment, stream: MutPtr<FILE>, format: ConstPtr<u8>, arg: VaList) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "vfprintf({:?}, {:?} ({:?}), ...)",
        stream,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    // TODO: I/O error handling
    match env.mem.read(stream).fd {
        STDIN_FILENO => panic!("Unexpected file descriptor"),
        STDOUT_FILENO => _ = std::io::stdout().write_all(&res),
        STDERR_FILENO => _ = std::io::stderr().write_all(&res),
        _ => {
            let buf = env.mem.alloc_and_write_cstr(res.as_slice());
            let result = fwrite(
                env,
                buf.cast_const().cast(),
                1,
                res.len() as GuestUSize,
                stream,
            );
            assert_eq!(result, res.len() as GuestUSize);
            env.mem.free(buf.cast());
        }
    }
    res.len().try_into().unwrap()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sscanf(_, _, _)),
    export_c_func!(swscanf(_, _, _)),
    export_c_func!(vsscanf(_, _, _)),
    export_c_func!(fscanf(_, _, _)),
    export_c_func!(snprintf(_, _, _, _)),
    export_c_func!(__snprintf_chk(_, _, _, _, _, _)),
    export_c_func!(vasprintf(_, _, _)),
    export_c_func!(vprintf(_, _)),
    export_c_func!(vwprintf(_, _)),
    export_c_func!(vsnprintf(_, _, _, _)),
    export_c_func!(vsnprintf_l(_, _, _, _, _)),
    export_c_func!(vsprintf(_, _, _)),
    export_c_func!(__sprintf_chk(_, _, _, _, _)),
    export_c_func!(__vsnprintf_chk(_, _, _, _, _, _)),
    export_c_func!(sprintf(_, _, _)),
    export_c_func!(swprintf(_, _, _, _)),
    export_c_func!(vswprintf(_, _, _, _)),
    export_c_func!(printf(_, _)),
    export_c_func!(fprintf(_, _, _)),
    export_c_func!(vfprintf(_, _, _)),
];

// Helper function, not a part of printf family
// TODO: write proper libc's isspace()
pub fn isspace(env: &mut Environment, src: ConstPtr<u8>) -> bool {
    let c = env.mem.read(src);
    isspace_inner(c)
}
pub fn isspace_inner(c: u8) -> bool {
    // Rust's definition of whitespace excludes vertical tab, unlike C's
    c.is_ascii_whitespace() || c == b'\x0b'
}
