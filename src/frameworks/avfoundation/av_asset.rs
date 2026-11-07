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

use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};

pub type AVPlayerItemStatus = NSInteger;
pub const AVPLAYER_ITEM_STATUS_UNKNOWN: AVPlayerItemStatus = 0;

pub const AVPlayerItemDidPlayToEndTimeNotification: &str =
    "AVPlayerItemDidPlayToEndTimeNotification";
pub const CONSTANTS: ConstantExports = &[(
    "_AVPlayerItemDidPlayToEndTimeNotification",
    HostConstant::NSString(AVPlayerItemDidPlayToEndTimeNotification),
)];

struct AVURLAssetHostObject {
    url: id,
    options: id,
}
impl HostObject for AVURLAssetHostObject {}

struct AVPlayerItemHostObject {
    asset: id,
}
impl HostObject for AVPlayerItemHostObject {}

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
    env.diagnostic_media_event("asset-init", this, url, options.to_bits());
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
    env.diagnostic_media_event("asset-dealloc", this, url, options.to_bits());
    release(env, url);
    release(env, options);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation AVPlayerItem: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(AVPlayerItemHostObject { asset: nil }),
        &mut env.mem,
    )
}

+ (id)playerItemWithAsset:(id)asset {
    let item: id = msg![env; this alloc];
    let item: id = msg![env; item initWithAsset:asset];
    autorelease(env, item)
}

- (id)initWithAsset:(id)asset {
    assert_ne!(asset, nil, "AVPlayerItem requires a non-nil asset");
    env.diagnostic_media_event("player-item-init", this, asset, 0);
    let asset = retain(env, asset);
    let host = env.objc.borrow_mut::<AVPlayerItemHostObject>(this);
    assert_eq!(host.asset, nil);
    host.asset = asset;
    this
}

- (id)asset {
    env.objc.borrow::<AVPlayerItemHostObject>(this).asset
}

// A newly constructed item begins in Unknown. Media inspection/loading is not
// implemented, so this partial contract deliberately never claims Ready or
// Failed and never fabricates an NSError.
- (AVPlayerItemStatus)status {
    let asset = env.objc.borrow::<AVPlayerItemHostObject>(this).asset;
    env.diagnostic_media_event(
        "player-item-status",
        this,
        asset,
        AVPLAYER_ITEM_STATUS_UNKNOWN as u32,
    );
    AVPLAYER_ITEM_STATUS_UNKNOWN
}

- (id)error {
    let asset = env.objc.borrow::<AVPlayerItemHostObject>(this).asset;
    env.diagnostic_media_event("player-item-error", this, asset, 0);
    nil
}

- (())dealloc {
    let asset = env.objc.borrow::<AVPlayerItemHostObject>(this).asset;
    env.diagnostic_media_event("player-item-dealloc", this, asset, 0);
    release(env, asset);
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

    #[test]
    fn player_item_preserves_asset_identity_and_balances_ownership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let asset: id = msg_class![env; AVAsset new];
        let count_before: NSUInteger = msg![env; asset retainCount];
        let item: id = msg_class![env; AVPlayerItem alloc];
        let item: id = msg![env; item initWithAsset:asset];
        let returned: id = msg![env; item asset];
        let status: AVPlayerItemStatus = msg![env; item status];
        let error: id = msg![env; item error];
        assert_eq!(returned, asset);
        assert_eq!(status, AVPLAYER_ITEM_STATUS_UNKNOWN);
        assert_eq!(error, nil);
        let count_during: NSUInteger = msg![env; asset retainCount];
        assert_eq!(count_during, count_before + 1);
        release(env, item);
        let count_after: NSUInteger = msg![env; asset retainCount];
        assert_eq!(count_after, count_before);
        release(env, asset);
    }

    #[test]
    fn dev_248_playback_end_notification_has_historical_identity() {
        let (_, constant) = CONSTANTS
            .iter()
            .find(|(symbol, _)| *symbol == "_AVPlayerItemDidPlayToEndTimeNotification")
            .unwrap();
        let HostConstant::NSString(value) = constant else {
            panic!("notification was not exported as NSString")
        };
        assert_eq!(*value, "AVPlayerItemDidPlayToEndTimeNotification");
    }
}
