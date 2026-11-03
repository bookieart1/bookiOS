/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSURLConnection` and `NSURLResponse`.

use super::{ns_run_loop, ns_string, NSInteger, NSUInteger};
use crate::dyld::{ConstantExports, HostConstant};
use crate::environment::Environment;
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, nil, objc_classes, release, retain, ClassExports,
    HostObject, NSZonePtr, SEL,
};
use std::borrow::Cow;

pub const NSURLErrorDomain: &str = "NSURLErrorDomain";
pub const NSErrorFailingURLStringKey: &str = "NSErrorFailingURLStringKey";

pub const CONSTANTS: ConstantExports = &[
    (
        "_NSURLErrorDomain",
        HostConstant::NSString(NSURLErrorDomain),
    ),
    (
        "_NSErrorFailingURLStringKey",
        HostConstant::NSString(NSErrorFailingURLStringKey),
    ),
    (
        "_NSURLErrorFailingURLStringErrorKey",
        HostConstant::NSString(NSErrorFailingURLStringKey),
    ),
];

/// Our helper type, Foundation just uses ints.
type NSURLErrorCode = NSInteger;
const NSURLErrorNotConnectedToInternet: NSURLErrorCode = -1009;
const NSURLErrorFileDoesNotExist: NSURLErrorCode = -1100;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ConnectionState {
    Initialized,
    Scheduled,
    Running,
    Finished,
    Cancelled,
}

struct NSURLConnectionHostObject {
    /// Independent immutable snapshot, as required by NSURLConnection.
    request: id,
    /// Strong until completion, failure, or cancellation.
    delegate: id,
    /// Run loop on which the connection was created or explicitly scheduled.
    run_loop: id,
    state: ConnectionState,
}
impl HostObject for NSURLConnectionHostObject {}

struct NSURLResponseHostObject {
    url: id,
    mime_type: id,
    expected_content_length: i64,
    text_encoding_name: id,
}
impl HostObject for NSURLResponseHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSURLResponse: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSURLResponseHostObject {
        url: nil,
        mime_type: nil,
        expected_content_length: -1,
        text_encoding_name: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithURL:(id)url
          MIMEType:(id)mime_type
expectedContentLength:(NSInteger)expected_content_length
  textEncodingName:(id)text_encoding_name {
    if url == nil {
        release(env, this);
        return nil;
    }
    let url: id = msg![env; url copy];
    let mime_type: id = msg![env; mime_type copy];
    let text_encoding_name: id = msg![env; text_encoding_name copy];
    *env.objc.borrow_mut::<NSURLResponseHostObject>(this) = NSURLResponseHostObject {
        url,
        mime_type,
        expected_content_length: i64::from(expected_content_length),
        text_encoding_name,
    };
    this
}

- (id)URL {
    env.objc.borrow::<NSURLResponseHostObject>(this).url
}

- (id)MIMEType {
    env.objc.borrow::<NSURLResponseHostObject>(this).mime_type
}

- (i64)expectedContentLength {
    env.objc.borrow::<NSURLResponseHostObject>(this).expected_content_length
}

- (id)textEncodingName {
    env.objc.borrow::<NSURLResponseHostObject>(this).text_encoding_name
}

- (id)suggestedFilename {
    let url = env.objc.borrow::<NSURLResponseHostObject>(this).url;
    let path: id = msg![env; url path];
    msg![env; path lastPathComponent]
}

- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (())dealloc {
    let &NSURLResponseHostObject {
        url,
        mime_type,
        text_encoding_name,
        ..
    } = env.objc.borrow(this);
    release(env, url);
    release(env, mime_type);
    release(env, text_encoding_name);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end


@implementation NSURLConnection: NSObject

+ (bool)canHandleRequest:(id)request { // NSURLRequest *
    // This is a preflight for whether a connection can actually be created and
    // started. The current transport implements local files only, so report
    // that boundary truthfully instead of claiming unavailable network I/O.
    local_file_url_from_request(env, request).is_some()
}

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSURLConnectionHostObject {
        request: nil,
        delegate: nil,
        run_loop: nil,
        state: ConnectionState::Initialized,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)sendSynchronousRequest:(id)request // NSURLRequest *
           returningResponse:(MutPtr<id>)response // NSURLResponse **
                       error:(MutPtr<id>)out_error { // NSError **
    if !response.is_null() {
        env.mem.write(response, nil);
    }
    if !out_error.is_null() {
        env.mem.write(out_error, nil);
    }

    let Some(url) = local_file_url_from_request(env, request) else {
        log!(
            "TODO: [NSURLConnection sendSynchronousRequest:{:?} ('{}') response:{:?} error:{:?}] -> nil",
            request,
            url_string_from_request(env, request),
            response,
            out_error,
        );
        if !out_error.is_null() {
            let error = make_url_error(env, request, NSURLErrorNotConnectedToInternet);
            env.mem.write(out_error, error);
        }
        return nil;
    };

    let data: id = msg_class![env; NSData dataWithContentsOfURL:url];
    if data == nil {
        log!(
            "NSURLConnection could not read local file request '{}'",
            url_string_from_request(env, request),
        );
        if !out_error.is_null() {
            let error = make_url_error(env, request, NSURLErrorFileDoesNotExist);
            env.mem.write(out_error, error);
        }
        return nil;
    }

    if !response.is_null() {
        let length: NSUInteger = msg![env; data length];
        let expected_length = NSInteger::try_from(length).unwrap_or(-1);
        let response_object = make_file_response(env, url, expected_length);
        autorelease(env, response_object);
        env.mem.write(response, response_object);
    }
    data
}

+ (id)connectionWithRequest:(id)request // NSURLRequest *
                   delegate:(id)delegate {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithRequest:request delegate:delegate];
    autorelease(env, new)
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate {
    msg![env; this initWithRequest:request delegate:delegate startImmediately:true]
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate
     startImmediately:(bool)start_immediately {
    if local_file_url_from_request(env, request).is_none() {
        log!(
            "TODO: [(NSURLConnection *){:?} initWithRequest:{:?} ('{}') delegate:{:?} startImmediately:{}] -> nil",
            this,
            request,
            url_string_from_request(env, request),
            delegate,
            start_immediately,
        );
        release(env, this);
        return nil;
    }

    let request: id = msg![env; request copy];
    let delegate = retain(env, delegate);
    let run_loop: id = msg_class![env; NSRunLoop currentRunLoop];
    *env.objc.borrow_mut::<NSURLConnectionHostObject>(this) = NSURLConnectionHostObject {
        request,
        delegate,
        run_loop,
        state: ConnectionState::Initialized,
    };

    log!(
        "[(NSURLConnection *){:?} initWithRequest:{:?} ('{}') delegate:{:?} startImmediately:{}] using local-file transport",
        this,
        request,
        url_string_from_request(env, request),
        delegate,
        start_immediately,
    );
    if start_immediately {
        schedule_connection(env, this);
    }
    this
}

- (())start {
    schedule_connection(env, this);
}

- (())cancel {
    cancel_connection(env, this);
}

- (())scheduleInRunLoop:(id)run_loop
                 forMode:(id)_mode {
    let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(this);
    if host.state == ConnectionState::Initialized {
        host.run_loop = run_loop;
    }
}

- (())_touchHLEStartLocalFileRequest {
    run_local_file_connection(env, this);
}

- (())dealloc {
    let &NSURLConnectionHostObject {
        request, delegate, ..
    } = env.objc.borrow(this);
    release(env, request);
    release(env, delegate);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

};

fn local_file_url_from_request(env: &mut Environment, request: id) -> Option<id> {
    if request == nil {
        return None;
    }
    let url: id = msg![env; request URL];
    if url == nil || !msg![env; url isFileURL] {
        None
    } else {
        Some(url)
    }
}

fn make_url_error(env: &mut Environment, request: id, code: NSURLErrorCode) -> id {
    let domain = ns_string::get_static_str(env, NSURLErrorDomain);
    let user_info = if request == nil {
        nil
    } else {
        let url: id = msg![env; request URL];
        let url_string: id = msg![env; url absoluteString];
        if url_string == nil {
            nil
        } else {
            let key = ns_string::get_static_str(env, NSErrorFailingURLStringKey);
            msg_class![env; NSDictionary dictionaryWithObject:url_string forKey:key]
        }
    };
    msg_class![env; NSError errorWithDomain:domain code:code userInfo:user_info]
}

fn make_file_response(env: &mut Environment, url: id, expected_content_length: NSInteger) -> id {
    let response: id = msg_class![env; NSURLResponse alloc];
    msg![env; response initWithURL:url
                         MIMEType:nil
            expectedContentLength:expected_content_length
                 textEncodingName:nil]
}

fn local_start_selector(env: &mut Environment) -> SEL {
    env.objc
        .register_host_selector("_touchHLEStartLocalFileRequest".to_string(), &mut env.mem)
}

fn schedule_connection(env: &mut Environment, connection: id) {
    let run_loop = {
        let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
        if host.state != ConnectionState::Initialized {
            return;
        }
        host.state = ConnectionState::Scheduled;
        host.run_loop
    };
    let selector = local_start_selector(env);
    ns_run_loop::add_perform_request(env, run_loop, connection, selector, nil, Some(0.0), false);
}

fn cancel_connection(env: &mut Environment, connection: id) {
    let (state, run_loop, delegate) = {
        let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
        if matches!(
            host.state,
            ConnectionState::Finished | ConnectionState::Cancelled
        ) {
            return;
        }
        let state = host.state;
        host.state = ConnectionState::Cancelled;
        (
            state,
            host.run_loop,
            std::mem::replace(&mut host.delegate, nil),
        )
    };
    if state == ConnectionState::Scheduled {
        let selector = local_start_selector(env);
        ns_run_loop::cancel_perform_requests(env, run_loop, connection, selector, nil);
    }
    release(env, delegate);
}

fn connection_is_running(env: &Environment, connection: id) -> bool {
    env.objc
        .borrow::<NSURLConnectionHostObject>(connection)
        .state
        == ConnectionState::Running
}

fn delegate_if_responds(env: &mut Environment, connection: id, name: &str) -> Option<(id, SEL)> {
    let delegate = env
        .objc
        .borrow::<NSURLConnectionHostObject>(connection)
        .delegate;
    if delegate == nil {
        return None;
    }
    let selector = env
        .objc
        .register_host_selector(name.to_string(), &mut env.mem);
    if msg![env; delegate respondsToSelector:selector] {
        Some((delegate, selector))
    } else {
        None
    }
}

fn finish_connection(env: &mut Environment, connection: id) -> id {
    let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
    if host.state != ConnectionState::Running {
        return nil;
    }
    host.state = ConnectionState::Finished;
    host.delegate
}

fn release_finished_delegate(env: &mut Environment, connection: id) {
    let delegate = {
        let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
        if host.state != ConnectionState::Finished {
            return;
        }
        std::mem::replace(&mut host.delegate, nil)
    };
    release(env, delegate);
}

fn fail_local_file_connection(env: &mut Environment, connection: id) {
    let request = env
        .objc
        .borrow::<NSURLConnectionHostObject>(connection)
        .request;
    let delegate = finish_connection(env, connection);
    if delegate != nil {
        let selector = env
            .objc
            .register_host_selector("connection:didFailWithError:".to_string(), &mut env.mem);
        if msg![env; delegate respondsToSelector:selector] {
            let error = make_url_error(env, request, NSURLErrorFileDoesNotExist);
            () = msg_send(env, (delegate, selector, connection, error));
        }
    }
    release_finished_delegate(env, connection);
}

fn run_local_file_connection(env: &mut Environment, connection: id) {
    let request = {
        let host = env.objc.borrow_mut::<NSURLConnectionHostObject>(connection);
        if host.state != ConnectionState::Scheduled {
            return;
        }
        host.state = ConnectionState::Running;
        host.request
    };
    let Some(url) = local_file_url_from_request(env, request) else {
        fail_local_file_connection(env, connection);
        return;
    };

    let data: id = msg_class![env; NSData alloc];
    let data: id = msg![env; data initWithContentsOfURL:url];
    if data == nil {
        log!(
            "NSURLConnection could not read local file request '{}'",
            url_string_from_request(env, request),
        );
        fail_local_file_connection(env, connection);
        return;
    }

    let length: NSUInteger = msg![env; data length];
    let expected_length = NSInteger::try_from(length).unwrap_or(-1);
    let response = make_file_response(env, url, expected_length);

    if let Some((delegate, selector)) =
        delegate_if_responds(env, connection, "connection:didReceiveResponse:")
    {
        () = msg_send(env, (delegate, selector, connection, response));
    }
    if connection_is_running(env, connection) {
        if let Some((delegate, selector)) =
            delegate_if_responds(env, connection, "connection:didReceiveData:")
        {
            () = msg_send(env, (delegate, selector, connection, data));
        }
    }

    release(env, response);
    release(env, data);
    if !connection_is_running(env, connection) {
        return;
    }

    let delegate = finish_connection(env, connection);
    if delegate != nil {
        let selector = env
            .objc
            .register_host_selector("connectionDidFinishLoading:".to_string(), &mut env.mem);
        if msg![env; delegate respondsToSelector:selector] {
            () = msg_send(env, (delegate, selector, connection));
        }
    }
    release_finished_delegate(env, connection);
}

fn url_string_from_request(env: &mut Environment, request: id) -> Cow<'static, str> {
    if request == nil {
        Cow::from("(null)")
    } else {
        let url = msg![env; request URL];
        if url == nil {
            return Cow::from("(null)");
        }
        let ns_string = msg![env; url absoluteString];
        if ns_string == nil {
            return Cow::from("(null)");
        }
        ns_string::to_rust_string(env, ns_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_query_matches_available_local_file_transport() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // NSURLRequest's convenience factory delegates to an initializer with
        // a fifth ARM argument, so this fixture needs synthetic stack space.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];

        let path = ns_string::from_rust_string(env, "/tmp/idared-local-request".to_owned());
        let file_url: id = msg_class![env; NSURL fileURLWithPath:path];
        let file_request: id = msg_class![env; NSURLRequest requestWithURL:file_url];
        assert!(msg_class![env; NSURLConnection canHandleRequest:file_request]);

        let string = ns_string::from_rust_string(env, "https://example.invalid/".to_owned());
        let network_url: id = msg_class![env; NSURL URLWithString:string];
        let network_request: id = msg_class![env; NSURLRequest requestWithURL:network_url];
        assert!(!msg_class![env; NSURLConnection canHandleRequest:network_request]);
        assert!(!msg_class![env; NSURLConnection canHandleRequest:nil]);

        release(env, path);
        release(env, string);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
