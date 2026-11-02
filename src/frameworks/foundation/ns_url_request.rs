/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSURLRequest and NSMutableURLRequest`.

use super::{ns_string, NSTimeInterval, NSUInteger};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::objc::{
    autorelease, id, nil, objc_classes, release, ClassExports, HostObject, NSZonePtr,
};
use crate::{msg, msg_class, Environment};

type NSURLRequestCachePolicy = NSUInteger;
const NSURLRequestUseProtocolCachePolicy: NSURLRequestCachePolicy = 0;

struct NSURLRequestHostObject {
    /// `NSURL*`
    url: id,
    cache_policy: NSURLRequestCachePolicy,
    timeout_interval: NSTimeInterval,
    http_should_handle_cookies: bool,
    // Request components
    /// `NSString*`
    http_method: id,
    /// `NSData*`
    http_body: id,
    // Header fields
    /// `NSDictionary*`
    http_header_fields: id,
}
impl HostObject for NSURLRequestHostObject {}

/// Make an independent request snapshot, preserving the mutable/immutable
/// distinction required by NSCopying and NSMutableCopying.
fn copy_request(env: &mut Environment, source: id, mutable: bool) -> id {
    let (
        url,
        cache_policy,
        timeout_interval,
        http_should_handle_cookies,
        http_method,
        http_body,
        http_header_fields,
    ) = {
        let source = env.objc.borrow::<NSURLRequestHostObject>(source);
        (
            source.url,
            source.cache_policy,
            source.timeout_interval,
            source.http_should_handle_cookies,
            source.http_method,
            source.http_body,
            source.http_header_fields,
        )
    };

    let url: id = msg![env; url copy];
    let http_method: id = msg![env; http_method copy];
    let http_body: id = msg![env; http_body copy];
    // The backing dictionary must stay mutable even for NSURLRequest because
    // NSMutableURLRequest inherits the same host-object representation.
    let http_header_fields: id = msg![env; http_header_fields mutableCopy];

    let copy: id = if mutable {
        msg_class![env; NSMutableURLRequest alloc]
    } else {
        msg_class![env; NSURLRequest alloc]
    };

    let (old_url, old_method, old_body, old_headers) = {
        let host = env.objc.borrow::<NSURLRequestHostObject>(copy);
        (host.url, host.http_method, host.http_body, host.http_header_fields)
    };
    *env.objc.borrow_mut::<NSURLRequestHostObject>(copy) = NSURLRequestHostObject {
        url,
        cache_policy,
        timeout_interval,
        http_should_handle_cookies,
        http_method,
        http_body,
        http_header_fields,
    };
    release(env, old_url);
    release(env, old_method);
    release(env, old_body);
    release(env, old_headers);
    copy
}

fn mutable_header_fields(env: &mut Environment, request: id) -> id {
    let fields = env
        .objc
        .borrow::<NSURLRequestHostObject>(request)
        .http_header_fields;
    if fields != nil {
        return fields;
    }

    let fields: id = msg_class![env; NSMutableDictionary new];
    env.objc
        .borrow_mut::<NSURLRequestHostObject>(request)
        .http_header_fields = fields;
    fields
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSURLRequest: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    // TODO: this should be mutable _only_ in the subclass
    // TODO: fill default headers
    let http_header_fields: id = msg_class![env; NSMutableDictionary new];
    let host_object = Box::new(NSURLRequestHostObject {
        url: nil,
        cache_policy: NSURLRequestUseProtocolCachePolicy,
        timeout_interval: 60.0,
        http_should_handle_cookies: true,
        http_method: ns_string::get_static_str(env, "GET"),
        http_body: nil,
        http_header_fields,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)requestWithURL:(id)url {
    msg![env; this requestWithURL:url
                      cachePolicy:NSURLRequestUseProtocolCachePolicy
                  timeoutInterval:60.0]
}

+ (id)requestWithURL:(id)url
         cachePolicy:(NSURLRequestCachePolicy)cache_policy
     timeoutInterval:(NSTimeInterval)timeout_interval {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithURL:url
                                cachePolicy:cache_policy
                            timeoutInterval:timeout_interval];
    autorelease(env, new)
}

- (id)initWithURL:(id)url
        cachePolicy:(NSURLRequestCachePolicy)cache_policy
    timeoutInterval:(NSTimeInterval)timeout_interval {
    if url == nil {
        return nil;
    }
    let url_desc: id = msg![env; url description];
    log_dbg!(
        "[(NSURLRequest *){:?} initWithURL:{} cachePolicy:{} timeoutInterval:{}]",
        this,
        to_rust_string(env, url_desc),
        cache_policy,
        timeout_interval,
    );

    let url_copy = msg![env; url copy];
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).url = url_copy;
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).cache_policy = cache_policy;
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).timeout_interval = timeout_interval;

    this
}

- (id)initWithURL:(id)url {
    if url == nil {
        return nil;
    }
    let url_desc: id = msg![env; url description];
        log_dbg!(
        "[(NSURLRequest *){:?} initWithURL:{}]",
        this,
        to_rust_string(env, url_desc)
    );

    let url_copy = msg![env; url copy];
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).url = url_copy;
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).cache_policy = NSURLRequestUseProtocolCachePolicy;
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).timeout_interval = 60.0;

    this
}

- (id)URL {
    env.objc.borrow::<NSURLRequestHostObject>(this).url
}
- (NSURLRequestCachePolicy)cachePolicy {
    env.objc.borrow::<NSURLRequestHostObject>(this).cache_policy
}
- (NSTimeInterval)timeoutInterval {
    env.objc.borrow::<NSURLRequestHostObject>(this).timeout_interval
}
- (bool)HTTPShouldHandleCookies {
    env.objc.borrow::<NSURLRequestHostObject>(this).http_should_handle_cookies
}
- (id)HTTPMethod {
    env.objc.borrow::<NSURLRequestHostObject>(this).http_method
}
- (id)HTTPBody {
    env.objc.borrow::<NSURLRequestHostObject>(this).http_body
}
- (id)allHTTPHeaderFields {
    let fields = env.objc.borrow::<NSURLRequestHostObject>(this).http_header_fields;
    if fields == nil {
        return nil;
    }
    let fields: id = msg![env; fields copy];
    autorelease(env, fields)
}
- (id)valueForHTTPHeaderField:(id)field {
    let fields = env.objc.borrow::<NSURLRequestHostObject>(this).http_header_fields;
    msg![env; fields objectForKey:field]
}

// NSCopying and NSMutableCopying. A copy of a mutable request is immutable;
// a mutable copy is always a distinct mutable snapshot.
- (id)copyWithZone:(NSZonePtr)_zone {
    copy_request(env, this, false)
}
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    copy_request(env, this, true)
}

- (())dealloc {
    log_dbg!("[(NSURLRequest*){:?} dealloc]", this);
    let &NSURLRequestHostObject {
        url,
        http_method,
        http_body,
        http_header_fields,
        ..
    } = env.objc.borrow(this);
    release(env, url);
    release(env, http_method);
    release(env, http_body);
    release(env, http_header_fields);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation NSMutableURLRequest: NSURLRequest

- (())setCachePolicy:(NSURLRequestCachePolicy)cache_policy {
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).cache_policy = cache_policy;
}

- (())setTimeoutInterval:(NSTimeInterval)timeout_interval {
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).timeout_interval = timeout_interval;
}

- (())setHTTPShouldHandleCookies:(bool)should_handle_cookies {
    env.objc.borrow_mut::<NSURLRequestHostObject>(this).http_should_handle_cookies = should_handle_cookies;
}

- (())setHTTPMethod:(id)http_method { // NSString *
    let http_method_copy = msg![env; http_method copy];

    let host_obj = env.objc.borrow_mut::<NSURLRequestHostObject>(this);
    let old_http_method = std::mem::replace(&mut host_obj.http_method, http_method_copy);
    release(env, old_http_method);
    // No need to retain http_method as we made a copy
}

- (())setHTTPBody:(id)http_body { // NSData *
    let http_body_copy = msg![env; http_body copy];

    let host_obj = env.objc.borrow_mut::<NSURLRequestHostObject>(this);
    let old_http_body = std::mem::replace(&mut host_obj.http_body, http_body_copy);
    release(env, old_http_body);
    // No need to retain http_body as we made a copy
}

- (())setURL:(id)url { // NSURL *
    let url_copy = msg![env; url copy];

    let host_obj = env.objc.borrow_mut::<NSURLRequestHostObject>(this);
    let old_url = std::mem::replace(&mut host_obj.url, url_copy);
    release(env, old_url);
    // No need to retain url_copy as we made a copy
}

- (())setValue:(id)value // NSString *
    forHTTPHeaderField:(id)field { // NSString *
    log_dbg!("[(NSURLRequest*){:?} setValue:{:?} forHTTPHeaderField:'{}']", this, value, to_rust_string(env, field));
    let http_header_fields = mutable_header_fields(env, this);
    if value == nil {
        () = msg![env; http_header_fields removeObjectForKey:field];
    } else {
        () = msg![env; http_header_fields setObject:value forKey:field];
    }
}

- (())setAllHTTPHeaderFields:(id)header_fields { // NSDictionary<NSString*, NSString*>*
    log_dbg!("[(NSURLRequest*){:?} setAllHTTPHeaderFields:{:?}]", this, header_fields);
    let header_fields: id = if header_fields == nil {
        nil
    } else {
        // Keep a private mutable backing dictionary while honoring the public
        // copy property: later mutations of the caller's dictionary must not
        // affect the request.
        msg![env; header_fields mutableCopy]
    };
    let host = env.objc.borrow_mut::<NSURLRequestHostObject>(this);
    let old = std::mem::replace(&mut host.http_header_fields, header_fields);
    release(env, old);
}

- (())addValue:(id)value // NSString *
    forHTTPHeaderField:(id)field { // NSString *
    log_dbg!("[(NSURLRequest*){:?} addValue:'{}' forHTTPHeaderField:'{}']", this, to_rust_string(env, value), to_rust_string(env, field));
    let http_header_fields = mutable_header_fields(env, this);
    let existing: id = msg![env; http_header_fields objectForKey:field];
    assert_eq!(existing, nil); // TODO: append values with comma
    () = msg![env; http_header_fields setObject:value forKey:field];
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_http_header_fields_replaces_with_an_independent_copy_and_nil_clears() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let request: id = msg_class![env; NSMutableURLRequest new];
        let source: id = msg_class![env; NSMutableDictionary new];
        let first_key = ns_string::get_static_str(env, "X-First");
        let second_key = ns_string::get_static_str(env, "X-Second");
        let first_value = ns_string::get_static_str(env, "one");
        let second_value = ns_string::get_static_str(env, "two");

        () = msg![env; source setObject:first_value forKey:first_key];
        () = msg![env; request setAllHTTPHeaderFields:source];
        () = msg![env; source setObject:second_value forKey:second_key];

        let snapshot: id = msg![env; request allHTTPHeaderFields];
        let count: NSUInteger = msg![env; snapshot count];
        let actual: id = msg![env; snapshot objectForKey:first_key];
        let absent: id = msg![env; snapshot objectForKey:second_key];
        assert_eq!(count, 1);
        assert_eq!(actual, first_value);
        assert_eq!(absent, nil);

        () = msg![env; request setAllHTTPHeaderFields:nil];
        let cleared: id = msg![env; request allHTTPHeaderFields];
        assert_eq!(cleared, nil);

        // Per-field mutation remains usable after clearing the aggregate
        // property and lazily creates a fresh backing dictionary.
        () = msg![env; request setValue:second_value forHTTPHeaderField:second_key];
        let restored: id = msg![env; request valueForHTTPHeaderField:second_key];
        assert_eq!(restored, second_value);

        release(env, source);
        release(env, request);
        release(env, pool);
    }
}
