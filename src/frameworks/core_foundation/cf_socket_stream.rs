/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! Socket streams in the emulator's offline environment. Creation succeeds;
//! opening fails with ENETUNREACH, never an open-completed or writable event.
//! The finite error notification uses the existing default/common run loop.
use crate::abi::{CallFromHost, GuestFunction};
use crate::frameworks::foundation::ns_run_loop::add_perform_request;
use crate::frameworks::foundation::ns_string::{get_static_str, to_rust_string};
use crate::libc::errno::ENETUNREACH;
use crate::mem::{MutPtr, MutVoidPtr, SafeRead};
use crate::objc::{id, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject};
use crate::{impl_GuestRet_for_large_struct, Environment};

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct StreamError { pub domain: i32, pub error: i32 }
unsafe impl SafeRead for StreamError {}
impl_GuestRet_for_large_struct!(StreamError);

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ClientContext {
    version: i32,
    info: MutVoidPtr,
    retain: GuestFunction,
    release: GuestFunction,
    copy_description: GuestFunction,
}
unsafe impl SafeRead for ClientContext {}

#[derive(Clone, Copy)]
struct Client { events: u32, callback: GuestFunction, context: ClientContext }
#[derive(Default)]
struct SocketStream {
    status: u32,
    client: Option<Client>,
    schedules: Vec<(id, id)>,
    queued: bool,
    delivered: bool,
    closed: bool,
}
impl HostObject for SocketStream {}

pub fn is_socket(env: &Environment, stream: id) -> bool {
    stream != nil && env.objc.get_class_name(crate::objc::ObjC::read_isa(stream, &env.mem)) == "_touchHLE_OfflineSocketStream"
}

pub fn create_pair(env: &mut Environment, host: id, port: u32, read: MutPtr<id>, write: MutPtr<id>) {
    // Validate the actual host object rather than accepting an arbitrary
    // pointer.
    assert_ne!(host, nil);
    let name = to_rust_string(env, host).into_owned();
    assert!(port <= 65535);
    log!("Offline socket stream pair: host={name:?} port={port}; open will report ENETUNREACH");
    let class = env.objc.get_known_class("_touchHLE_OfflineSocketStream", &mut env.mem);
    for out in [read, write] {
        if !out.is_null() {
            let stream = env.objc.alloc_object(class, Box::<SocketStream>::default(), &mut env.mem);
            env.mem.write(out, stream);
        }
    }
}

fn release_client(env: &mut Environment, client: Option<Client>) {
    if let Some(client) = client {
        let callback = client.context.release;
        if !callback.to_ptr().is_null() {
            let _: () = callback.call_from_host(env, (client.context.info,));
        }
    }
}

pub fn set_client(env: &mut Environment, stream: id, events: u32, callback: GuestFunction, context: MutVoidPtr) -> bool {
    if !is_socket(env, stream) { return false; }
    let new = if events == 0 || callback.to_ptr().is_null() {
        None
    } else {
        if context.is_null() { return false; }
        let mut context: ClientContext = env.mem.read(context.cast());
        if context.version != 0 { return false; }
        let retain_callback = context.retain;
        if !retain_callback.to_ptr().is_null() {
            context.info = retain_callback.call_from_host(env, (context.info,));
        }
        Some(Client { events, callback, context })
    };
    let state = env.objc.borrow_mut::<SocketStream>(stream);
    let old = std::mem::replace(&mut state.client, new);
    state.delivered = false;
    release_client(env, old);
    queue_error(env, stream);
    true
}

fn queue_error(env: &mut Environment, stream: id) {
    let state = env.objc.borrow_mut::<SocketStream>(stream);
    if state.closed || state.status != 7 || state.queued || state.delivered || !state.client.is_some_and(|c| c.events & 8 != 0) { return; }
    let Some(&(run_loop, _)) = state.schedules.first() else { return; };
    state.queued = true;
    let selector = env.objc.lookup_selector("_deliverOfflineError:").unwrap();
    add_perform_request(env, run_loop, stream, selector, run_loop, Some(0.0), false);
}

pub fn error(env: &mut Environment, stream: id) -> StreamError {
    assert!(is_socket(env, stream), "stream error for unsupported stream type");
    if env.objc.borrow::<SocketStream>(stream).status == 7 {
        StreamError { domain: 1, error: ENETUNREACH }
    } else { StreamError { domain: 0, error: 0 } }
}

pub fn copy_error(env: &mut Environment, stream: id) -> id {
    let error = error(env, stream);
    if error.error == 0 { return nil; }
    let domain = get_static_str(env, "NSPOSIXErrorDomain");
    let code = error.error;
    let object: id = msg_class![env; NSError errorWithDomain:domain code:code userInfo:nil];
    retain(env, object)
}

pub const CLASSES: ClassExports = objc_classes! {
(env, this, _cmd);
@implementation _touchHLE_OfflineSocketStream: NSObject
- (())open {
    let state = env.objc.borrow_mut::<SocketStream>(this);
    if state.status == 0 { state.status = 7; }
    queue_error(env, this);
}
- (u32)streamStatus { env.objc.borrow::<SocketStream>(this).status }
- (bool)hasBytesAvailable { false }
- (bool)hasSpaceAvailable { false }
- (i32)read:(MutVoidPtr)_buffer maxLength:(u32)_length { -1 }
- (i32)write:(MutVoidPtr)_buffer maxLength:(u32)_length { -1 }
- (())close {
    let state = env.objc.borrow_mut::<SocketStream>(this);
    state.closed = true;
    if state.status != 7 { state.status = 6; }
    // A queued request checks status before delivery. No successful events.
}
- (())scheduleInRunLoop:(id)run_loop forMode:(id)mode {
    assert_ne!(run_loop, nil);
    let mode_name = to_rust_string(env, mode).into_owned();
    assert!(mode_name == "kCFRunLoopDefaultMode" || mode_name == "kCFRunLoopCommonModes", "unimplemented stream run-loop mode {mode_name:?}");
    let schedules = &env.objc.borrow::<SocketStream>(this).schedules;
    if !schedules.contains(&(run_loop, mode)) {
        retain(env, run_loop);
        retain(env, mode);
        env.objc.borrow_mut::<SocketStream>(this).schedules.push((run_loop, mode));
    }
    queue_error(env, this);
}
- (())removeFromRunLoop:(id)run_loop forMode:(id)mode {
    let mode_name = to_rust_string(env, mode).into_owned();
    let schedules = env.objc.borrow::<SocketStream>(this).schedules.clone();
    if let Some(index) = schedules.iter().position(|&(r, m)| r == run_loop && to_rust_string(env, m) == mode_name) {
        let (r, m) = env.objc.borrow_mut::<SocketStream>(this).schedules.remove(index);
        release(env, r); release(env, m);
    }
}
- (())_deliverOfflineError:(id)run_loop {
    let state = env.objc.borrow_mut::<SocketStream>(this);
    state.queued = false;
    if state.closed || state.status != 7 || state.delivered { return; }
    if !state.schedules.iter().any(|&(r, _)| r == run_loop) {
        queue_error(env, this);
        return;
    }
    let Some(client) = state.client.filter(|c| c.events & 8 != 0) else { return; };
    state.delivered = true;
    // Client code may close/release the stream or replace its client
    // recursively.
    let mut info = client.context.info;
    let retain_callback = client.context.retain;
    let release_callback = client.context.release;
    if !retain_callback.to_ptr().is_null() {
        info = retain_callback.call_from_host(env, (info,));
    }
    log!("Offline socket stream error delivered: stream={this:?}, event=8, POSIX ENETUNREACH");
    let _: () = client.callback.call_from_host(env, (this, 8u32, info));
    if !retain_callback.to_ptr().is_null() && !release_callback.to_ptr().is_null() {
        let _: () = release_callback.call_from_host(env, (info,));
    }
}
- (())dealloc {
    let state = env.objc.borrow_mut::<SocketStream>(this);
    let client = state.client.take();
    let schedules = std::mem::take(&mut state.schedules);
    release_client(env, client);
    for (run_loop, mode) in schedules { release(env, mode); release(env, run_loop); }
    env.objc.dealloc_object(this, &mut env.mem);
}
@end
};
