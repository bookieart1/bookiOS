/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AVAsset` and the URL identity/lifetime portion of `AVURLAsset`.
//!
//! This deliberately does not claim media inspection, track discovery,
//! duration, metadata or playback support. It only implements the historical
//! construction contract needed before those later operations can be seen.

use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};

struct AVURLAssetHostObject {
    url: id,
    options: id,
}
impl HostObject for AVURLAssetHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation AVAsset: NSObject
@end

@implementation AVURLAsset: AVAsset

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(AVURLAssetHostObject { url: nil, options: nil }),
        &mut env.mem,
    )
}

+ (id)URLAssetWithURL:(id)url options:(id)options {
    let asset: id = msg![env; this alloc];
    let asset: id = msg![env; asset initWithURL:url options:options];
    autorelease(env, asset)
}

- (id)initWithURL:(id)url options:(id)options {
    assert!(url != nil, "AVURLAsset requires a non-nil URL");
    let url = retain(env, url);
    let options = retain(env, options);
    let host_object = env.objc.borrow_mut::<AVURLAssetHostObject>(this);
    assert_eq!(host_object.url, nil);
    assert_eq!(host_object.options, nil);
    host_object.url = url;
    host_object.options = options;
    this
}

- (id)URL {
    env.objc.borrow::<AVURLAssetHostObject>(this).url
}

- (())dealloc {
    let (url, options) = {
        let host_object = env.objc.borrow::<AVURLAssetHostObject>(this);
        (host_object.url, host_object.options)
    };
    release(env, url);
    release(env, options);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::{ns_string, NSUInteger};
    use crate::objc::{msg_class, release};
    use crate::Environment;

    #[test]
    fn url_asset_preserves_identity_and_balances_ownership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let url_string = ns_string::get_static_str(env, "file:///tmp/media.mov");
        let url: id = msg_class![env; NSURL alloc];
        let url: id = msg![env; url initWithString:url_string];
        let options: id = msg_class![env; NSMutableDictionary new];
        let url_count_before: NSUInteger = msg![env; url retainCount];
        let options_count_before: NSUInteger = msg![env; options retainCount];

        let asset: id = msg_class![env; AVURLAsset alloc];
        let asset: id = msg![env; asset initWithURL:url options:options];
        let returned_url: id = msg![env; asset URL];
        assert_eq!(returned_url, url);
        assert_eq!(
            env.objc.borrow::<AVURLAssetHostObject>(asset).options,
            options
        );
        let url_count_during: NSUInteger = msg![env; url retainCount];
        let options_count_during: NSUInteger = msg![env; options retainCount];
        assert_eq!(url_count_during, url_count_before + 1);
        assert_eq!(options_count_during, options_count_before + 1);

        release(env, asset);
        let url_count_after: NSUInteger = msg![env; url retainCount];
        let options_count_after: NSUInteger = msg![env; options retainCount];
        assert_eq!(url_count_after, url_count_before);
        assert_eq!(options_count_after, options_count_before);
        release(env, url);
        release(env, options);
    }

    #[test]
    fn url_asset_factory_returns_the_same_url() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let url_string = ns_string::get_static_str(env, "https://example.invalid/media.mov");
        let url: id = msg_class![env; NSURL alloc];
        let url: id = msg![env; url initWithString:url_string];
        let asset: id = msg_class![env; AVURLAsset URLAssetWithURL:url options:nil];
        let returned_url: id = msg![env; asset URL];
        assert_eq!(returned_url, url);
        release(env, pool);
        release(env, url);
    }
}
