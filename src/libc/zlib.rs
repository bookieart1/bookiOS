/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The small zlib `gz*` file interface used by libxml2 and early games.

use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::GuestPath;
use crate::libc::errno::{set_errno, EINVAL, EIO, ENOENT};
use crate::mem::{ConstPtr, GuestISize, GuestUSize, MutVoidPtr, Ptr};
use crate::Environment;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;

const Z_OK: i32 = 0;
const Z_STREAM_ERROR: i32 = -2;

struct GzStream {
    data: Vec<u8>,
    position: usize,
    eof: bool,
    /// True when zlib is transparently reading an uncompressed file.
    direct: bool,
}
#[derive(Default)]
pub struct State {
    streams: HashMap<MutVoidPtr, GzStream>,
}

fn gzopen(env: &mut Environment, path: ConstPtr<u8>, mode: ConstPtr<u8>) -> MutVoidPtr {
    set_errno(env, 0);
    let Some(path) = env.mem.cstr_at_utf8(path).ok().map(str::to_owned) else {
        set_errno(env, EINVAL);
        return Ptr::null();
    };
    let Some(mode) = env.mem.cstr_at_utf8(mode).ok().map(str::to_owned) else {
        set_errno(env, EINVAL);
        return Ptr::null();
    };
    if !mode.starts_with('r') {
        log!("Warning: gzopen({path:?}, {mode:?}) only supports reading");
        set_errno(env, EINVAL);
        return Ptr::null();
    }

    let Ok(mut file) = env.fs.open(GuestPath::new(&path)) else {
        set_errno(env, ENOENT);
        return Ptr::null();
    };
    let mut compressed = Vec::new();
    if let Err(error) = file.read_to_end(&mut compressed) {
        log!("Warning: gzopen could not read {path:?}: {error}");
        set_errno(env, EIO);
        return Ptr::null();
    }
    // zlib's gzopen transparently reads files without a gzip header too.
    let direct = !compressed.starts_with(&[0x1f, 0x8b]);
    let data = if !direct {
        let mut decoded = Vec::new();
        if let Err(error) = GzDecoder::new(&compressed[..]).read_to_end(&mut decoded) {
            log!("Warning: gzopen could not decompress {path:?}: {error}");
            set_errno(env, EIO);
            return Ptr::null();
        }
        decoded
    } else {
        compressed
    };

    let handle = env.mem.alloc(1);
    let previous = env.libc_state.zlib.streams.insert(
        handle,
        GzStream {
            data,
            position: 0,
            eof: false,
            direct,
        },
    );
    assert!(previous.is_none());
    handle
}

fn gzread(
    env: &mut Environment,
    file: MutVoidPtr,
    buffer: MutVoidPtr,
    length: GuestUSize,
) -> i32 {
    if buffer.is_null() && length != 0 {
        return Z_STREAM_ERROR;
    }
    let requested = length as usize;
    let (libc_state, mem) = (&mut env.libc_state, &mut env.mem);
    let Some(stream) = libc_state.zlib.streams.get_mut(&file) else {
        return Z_STREAM_ERROR;
    };
    let read = requested.min(stream.data.len().saturating_sub(stream.position));
    if read != 0 {
        mem
            .bytes_at_mut(buffer.cast(), read as GuestUSize)
            .copy_from_slice(&stream.data[stream.position..stream.position + read]);
        stream.position += read;
    }
    stream.eof = read < requested || stream.position == stream.data.len();
    read.try_into().unwrap_or(i32::MAX)
}

fn gzclose(env: &mut Environment, file: MutVoidPtr) -> i32 {
    if env.libc_state.zlib.streams.remove(&file).is_none() {
        return Z_STREAM_ERROR;
    }
    env.mem.free(file);
    Z_OK
}

fn gzeof(env: &mut Environment, file: MutVoidPtr) -> i32 {
    env.libc_state
        .zlib
        .streams
        .get(&file)
        .map_or(0, |stream| i32::from(stream.eof))
}

fn gzrewind(env: &mut Environment, file: MutVoidPtr) -> i32 {
    let Some(stream) = env.libc_state.zlib.streams.get_mut(&file) else {
        return Z_STREAM_ERROR;
    };
    stream.position = 0;
    stream.eof = false;
    Z_OK
}

fn gztell(env: &mut Environment, file: MutVoidPtr) -> GuestISize {
    env.libc_state
        .zlib
        .streams
        .get(&file)
        .and_then(|stream| stream.position.try_into().ok())
        .unwrap_or(-1)
}

fn gzdirect(env: &mut Environment, file: MutVoidPtr) -> i32 {
    env.libc_state
        .zlib
        .streams
        .get(&file)
        .map_or(0, |stream| i32::from(stream.direct))
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(gzopen(_, _)),
    export_c_func!(gzread(_, _, _)),
    export_c_func!(gzclose(_)),
    export_c_func!(gzeof(_)),
    export_c_func!(gzrewind(_)),
    export_c_func!(gztell(_)),
    export_c_func!(gzdirect(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_gzip_stream_reads_tracks_eof_and_rewinds() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let handle = env.mem.alloc(1);
        env.libc_state.zlib.streams.insert(
            handle,
            GzStream {
                data: b"abcdef".to_vec(),
                position: 0,
                eof: false,
                direct: false,
            },
        );
        let buffer = env.mem.alloc(8);
        assert_eq!(gzread(&mut env, handle, buffer, 4), 4);
        assert_eq!(env.mem.bytes_at(buffer.cast(), 4), b"abcd");
        assert_eq!(gztell(&mut env, handle), 4);
        assert_eq!(gzread(&mut env, handle, buffer, 4), 2);
        assert_eq!(gzeof(&mut env, handle), 1);
        assert_eq!(gzrewind(&mut env, handle), Z_OK);
        assert_eq!(gztell(&mut env, handle), 0);
        assert_eq!(gzdirect(&mut env, handle), 0);
        assert_eq!(gzclose(&mut env, handle), Z_OK);
        env.mem.free(buffer);
    }
}
