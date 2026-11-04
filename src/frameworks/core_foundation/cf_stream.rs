/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal Core Foundation stream compatibility.

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::CFIndex;
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::frameworks::foundation::NSUInteger;
use crate::mem::{MutPtr, MutVoidPtr};
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;

type CFStringRef = id;
type CFReadStreamRef = id;
type CFWriteStreamRef = id;

/// Creates an owned file-backed read stream. CFReadStream and NSInputStream
/// are toll-free bridged, so reusing the existing file stream object preserves
/// one source of truth for path, open, read, status, and teardown behavior.
fn CFReadStreamCreateWithFile(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    file_url: id,
) -> CFReadStreamRef {
    assert!(
        allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default(),
        "CFReadStreamCreateWithFile only supports the default allocator"
    );
    if file_url == nil {
        return nil;
    }
    let is_file_url: bool = msg![env; file_url isFileURL];
    if !is_file_url {
        return nil;
    }
    let path: id = msg![env; file_url path];
    if path == nil {
        return nil;
    }
    let stream: id = msg_class![env; NSInputStream alloc];
    msg![env; stream initWithFileAtPath:path]
}

fn CFReadStreamOpen(env: &mut Environment, stream: CFReadStreamRef) -> bool {
    if stream == nil {
        return false;
    }
    () = msg![env; stream open];
    let status: NSUInteger = msg![env; stream streamStatus];
    status == 2 // NSStreamStatusOpen
}

fn CFReadStreamRead(
    env: &mut Environment,
    stream: CFReadStreamRef,
    buffer: MutPtr<u8>,
    buffer_length: CFIndex,
) -> CFIndex {
    if stream == nil || buffer_length < 0 {
        return -1;
    }
    let buffer: MutVoidPtr = buffer.cast();
    let max_length = buffer_length as NSUInteger;
    msg![env; stream read:buffer maxLength:max_length]
}

fn CFReadStreamHasBytesAvailable(env: &mut Environment, stream: CFReadStreamRef) -> bool {
    stream != nil && msg![env; stream hasBytesAvailable]
}

fn CFReadStreamGetStatus(env: &mut Environment, stream: CFReadStreamRef) -> NSUInteger {
    if stream == nil {
        return 0; // kCFStreamStatusNotOpen
    }
    msg![env; stream streamStatus]
}

fn CFReadStreamClose(env: &mut Environment, stream: CFReadStreamRef) {
    if stream != nil {
        () = msg![env; stream close];
    }
}

/// Network-backed NSStream objects are not implemented yet. Returning null
/// stream references matches a failed stream-pair creation and lets callers
/// gracefully disable optional services such as analytics and cloud profiles.
fn CFStreamCreatePairWithSocketToHost(
    env: &mut Environment,
    _allocator: id,
    host: CFStringRef,
    port: u32,
    read_stream: MutPtr<CFReadStreamRef>,
    write_stream: MutPtr<CFWriteStreamRef>,
) {
    log!(
        "Warning: CFStreamCreatePairWithSocketToHost({}, {}) is unavailable; returning NULL streams",
        to_rust_string(env, host),
        port
    );
    if !read_stream.is_null() {
        env.mem.write(read_stream, nil);
    }
    if !write_stream.is_null() {
        env.mem.write(write_stream, nil);
    }
}

/// Callback delivery requires scheduled network streams, which this first
/// network-stream contract does not implement. Report registration failure
/// instead of panicking at the missing symbol or fabricating events.
fn CFWriteStreamSetClient(
    _env: &mut Environment,
    _stream: CFWriteStreamRef,
    _events: u32,
    _callback: MutVoidPtr,
    _context: MutVoidPtr,
) -> bool {
    false
}

fn CFWriteStreamScheduleWithRunLoop(
    env: &mut Environment,
    stream: CFWriteStreamRef,
    run_loop: id,
    mode: CFStringRef,
) {
    if stream != nil {
        () = msg![env; stream scheduleInRunLoop:run_loop forMode:mode];
    }
}

fn CFWriteStreamUnscheduleFromRunLoop(
    env: &mut Environment,
    stream: CFWriteStreamRef,
    run_loop: id,
    mode: CFStringRef,
) {
    if stream != nil {
        () = msg![env; stream removeFromRunLoop:run_loop forMode:mode];
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFStreamCreatePairWithSocketToHost(_, _, _, _, _)),
    export_c_func!(CFWriteStreamSetClient(_, _, _, _)),
    export_c_func!(CFWriteStreamScheduleWithRunLoop(_, _, _)),
    export_c_func!(CFWriteStreamUnscheduleFromRunLoop(_, _, _)),
    export_c_func!(CFReadStreamCreateWithFile(_, _)),
    export_c_func!(CFReadStreamOpen(_)),
    export_c_func!(CFReadStreamRead(_, _, _)),
    export_c_func!(CFReadStreamHasBytesAvailable(_)),
    export_c_func!(CFReadStreamGetStatus(_)),
    export_c_func!(CFReadStreamClose(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::core_foundation::CFRelease;
    use crate::frameworks::foundation::ns_string::from_rust_string;
    use crate::objc::{msg_class, release};

    #[test]
    fn file_stream_creation_is_owned_and_rejects_non_file_urls() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let path = from_rust_string(env, "/var/mobile/test-stream".to_string());
        let file_url: id = msg_class![env; NSURL fileURLWithPath:path];
        let stream = CFReadStreamCreateWithFile(env, kCFAllocatorDefault, file_url);
        assert_ne!(stream, nil);
        assert_eq!(CFReadStreamGetStatus(env, stream), 0);
        CFRelease(env, stream);

        let remote = from_rust_string(env, "https://example.com/file".to_string());
        let remote_url: id = msg_class![env; NSURL URLWithString:remote];
        assert_eq!(
            CFReadStreamCreateWithFile(env, kCFAllocatorDefault, remote_url),
            nil
        );
        assert!(!CFWriteStreamSetClient(
            env,
            nil,
            u32::MAX,
            MutVoidPtr::null(),
            MutVoidPtr::null(),
        ));
        CFWriteStreamScheduleWithRunLoop(env, nil, nil, nil);
        CFWriteStreamUnscheduleFromRunLoop(env, nil, nil, nil);
        // Balance the Create Rule of from_rust_string. The URL convenience
        // results are autoreleased by NSURL itself.
        release(env, path);
        release(env, remote);
    }
}
