/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The bounded historical `iconv` surface used by 1 Minute RPG.

use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::errno::{set_errno, E2BIG, EBADF, EILSEQ, EINVAL};
use crate::mem::{ConstPtr, GuestUSize, MutPtr, MutVoidPtr};
use crate::Environment;
use encoding_rs::{DecoderResult, EUC_KR};
use std::collections::HashSet;

const INVALID_ICONV: u32 = u32::MAX;

#[derive(Default)]
pub struct State {
    cp949_to_utf8: HashSet<u32>,
}

fn normalized_encoding_name(env: &Environment, name: ConstPtr<u8>) -> Option<String> {
    env.mem
        .cstr_at_utf8(name)
        .ok()
        .map(|name| name.replace(['-', '_'], "").to_ascii_uppercase())
}

fn iconv_open(env: &mut Environment, to_code: ConstPtr<u8>, from_code: ConstPtr<u8>) -> MutVoidPtr {
    let to_code = normalized_encoding_name(env, to_code);
    let from_code = normalized_encoding_name(env, from_code);
    if to_code.as_deref() != Some("UTF8")
        || !matches!(from_code.as_deref(), Some("CP949" | "WINDOWS949" | "EUCKR"))
    {
        set_errno(env, EINVAL);
        return MutVoidPtr::from_bits(INVALID_ICONV);
    }

    let handle = env.mem.alloc(1);
    env.libc_state.iconv.cp949_to_utf8.insert(handle.to_bits());
    handle
}

fn first_incomplete_cp949_byte(input: &[u8]) -> Option<usize> {
    let mut index = 0;
    while index < input.len() {
        if input[index] <= 0x7f {
            index += 1;
        } else if index + 1 == input.len() {
            return Some(index);
        } else {
            index += 2;
        }
    }
    None
}

fn iconv(
    env: &mut Environment,
    converter: MutVoidPtr,
    in_buffer: MutPtr<ConstPtr<u8>>,
    in_bytes_left: MutPtr<GuestUSize>,
    out_buffer: MutPtr<MutPtr<u8>>,
    out_bytes_left: MutPtr<GuestUSize>,
) -> GuestUSize {
    if !env
        .libc_state
        .iconv
        .cp949_to_utf8
        .contains(&converter.to_bits())
    {
        set_errno(env, EBADF);
        return GuestUSize::MAX;
    }
    if in_buffer.is_null() {
        return 0;
    }
    if in_bytes_left.is_null() || out_buffer.is_null() || out_bytes_left.is_null() {
        set_errno(env, EINVAL);
        return GuestUSize::MAX;
    }

    let input_pointer = env.mem.read(in_buffer);
    let input_len = env.mem.read(in_bytes_left);
    let output_pointer = env.mem.read(out_buffer);
    let output_len = env.mem.read(out_bytes_left);
    let input = env
        .mem
        .bytes_at(input_pointer, input_len)
        .to_vec();
    let incomplete = first_incomplete_cp949_byte(&input);
    let convertible_len = incomplete.unwrap_or(input.len());
    let mut output = vec![0; output_len as usize];
    let mut decoder = EUC_KR.new_decoder_without_bom_handling();
    let (result, read, written) = decoder.decode_to_utf8_without_replacement(
        &input[..convertible_len],
        &mut output,
        true,
    );
    env.mem
        .bytes_at_mut(output_pointer, written as GuestUSize)
        .copy_from_slice(&output[..written]);
    env.mem.write(in_buffer, input_pointer + read as u32);
    env.mem
        .write(in_bytes_left, input_len - read as GuestUSize);
    env.mem.write(out_buffer, output_pointer + written as u32);
    env.mem
        .write(out_bytes_left, output_len - written as GuestUSize);

    let error = match result {
        DecoderResult::OutputFull => Some(E2BIG),
        DecoderResult::Malformed(_, _) => Some(EILSEQ),
        DecoderResult::InputEmpty if incomplete.is_some() => Some(EINVAL),
        DecoderResult::InputEmpty => None,
    };
    if let Some(error) = error {
        set_errno(env, error);
        GuestUSize::MAX
    } else {
        0
    }
}

fn iconv_close(env: &mut Environment, converter: MutVoidPtr) -> i32 {
    if !env
        .libc_state
        .iconv
        .cp949_to_utf8
        .remove(&converter.to_bits())
    {
        set_errno(env, EBADF);
        return -1;
    }
    env.mem.free(converter);
    0
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(iconv_open(_, _)),
    export_c_func!(iconv(_, _, _, _, _)),
    export_c_func!(iconv_close(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libc::errno::get_errno;

    fn open_cp949_to_utf8(env: &mut Environment) -> MutVoidPtr {
        let to = env.mem.alloc_and_write_cstr(b"UTF-8").cast_const();
        let from = env.mem.alloc_and_write_cstr(b"CP949").cast_const();
        iconv_open(env, to, from)
    }

    fn allocate_bytes(env: &mut Environment, bytes: &[u8]) -> MutPtr<u8> {
        let pointer = env.mem.alloc(bytes.len() as GuestUSize).cast::<u8>();
        env.mem
            .bytes_at_mut(pointer, bytes.len() as GuestUSize)
            .copy_from_slice(bytes);
        pointer
    }

    #[test]
    fn development_256_iconv_converts_cp949_and_updates_all_cursors() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let converter = open_cp949_to_utf8(&mut env);
        let input = allocate_bytes(&mut env, &[0xb0, 0xa1]);
        let input_cursor = env.mem.alloc_and_write(input.cast_const());
        let input_left = env.mem.alloc_and_write(2u32);
        let output = env.mem.alloc(8).cast::<u8>();
        let output_cursor = env.mem.alloc_and_write(output);
        let output_left = env.mem.alloc_and_write(8u32);

        assert_eq!(
            iconv(
                &mut env,
                converter,
                input_cursor,
                input_left,
                output_cursor,
                output_left,
            ),
            0
        );
        assert_eq!(env.mem.read(input_left), 0);
        assert_eq!(env.mem.read(output_left), 5);
        assert_eq!(env.mem.bytes_at(output, 3), "가".as_bytes());
        assert_eq!(iconv_close(&mut env, converter), 0);
    }

    #[test]
    fn development_256_iconv_reports_output_and_incomplete_input_errors() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let converter = open_cp949_to_utf8(&mut env);
        let input = allocate_bytes(&mut env, &[0xb0, 0xa1]);
        let input_cursor = env.mem.alloc_and_write(input.cast_const());
        let input_left = env.mem.alloc_and_write(2u32);
        let output = env.mem.alloc(2).cast::<u8>();
        let output_cursor = env.mem.alloc_and_write(output);
        let output_left = env.mem.alloc_and_write(2u32);
        assert_eq!(
            iconv(
                &mut env,
                converter,
                input_cursor,
                input_left,
                output_cursor,
                output_left,
            ),
            GuestUSize::MAX
        );
        assert_eq!(get_errno(&mut env), E2BIG);

        let incomplete = allocate_bytes(&mut env, &[0xb0]);
        env.mem.write(input_cursor, incomplete.cast_const());
        env.mem.write(input_left, 1);
        let larger_output = env.mem.alloc(8).cast::<u8>();
        env.mem.write(output_cursor, larger_output);
        env.mem.write(output_left, 8);
        assert_eq!(
            iconv(
                &mut env,
                converter,
                input_cursor,
                input_left,
                output_cursor,
                output_left,
            ),
            GuestUSize::MAX
        );
        assert_eq!(get_errno(&mut env), EINVAL);
        assert_eq!(env.mem.read(input_left), 1);
    }

    #[test]
    fn development_256_iconv_rejects_unimplemented_encoding_pairs() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let to = env.mem.alloc_and_write_cstr(b"UTF-16").cast_const();
        let from = env.mem.alloc_and_write_cstr(b"CP949").cast_const();
        assert_eq!(
            iconv_open(&mut env, to, from).to_bits(),
            INVALID_ICONV
        );
        assert_eq!(get_errno(&mut env), EINVAL);
    }
}
