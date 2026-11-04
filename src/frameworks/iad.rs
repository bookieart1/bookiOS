/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! iOS 4 banner configuration and a finite unavailable-service error path.
//! No ad content, successful load, action, or service connection is fabricated.
//! Refresh policy and interstitial ads are not implemented.
use crate::dyld::{ConstantExports, HostConstant, HostDylib};
use crate::frameworks::core_graphics::{CGRect, CGSize};
use crate::frameworks::foundation::ns_run_loop::add_perform_request;
use crate::frameworks::foundation::ns_string::{get_static_str, to_rust_string};
use crate::frameworks::uikit::ui_view::UIViewHostObject;
use crate::objc::{id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil,
    objc_classes, release, ClassExports, NSZonePtr};
use crate::window::DeviceFamily;
use crate::Environment;

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/iAd.framework/iAd",
    aliases: &[],
    class_exports: &[CLASSES],
    constant_exports: &[CONSTANTS],
    function_exports: &[],
};
const CONSTANTS: ConstantExports = &[
    ("_ADErrorDomain", HostConstant::NSString("ADErrorDomain")),
    ("_ADBannerContentSizeIdentifier320x50", HostConstant::NSString("ADBannerContentSizeIdentifier320x50")),
    ("_ADBannerContentSizeIdentifier480x32", HostConstant::NSString("ADBannerContentSizeIdentifier480x32")),
    ("_ADBannerContentSizeIdentifierPortrait", HostConstant::NSString("ADBannerContentSizeIdentifierPortrait")),
    ("_ADBannerContentSizeIdentifierLandscape", HostConstant::NSString("ADBannerContentSizeIdentifierLandscape")),
];

#[derive(Default)]
struct Banner {
    superclass: UIViewHostObject,
    // iOS 4 assign semantics: the owner must clear the delegate before
    // teardown.
    delegate: id,
    required: id,
    current: id,
    section: id,
    failure_queued: bool,
    failure_delivered: bool,
}
impl_HostObject_with_superclass!(Banner);

fn banner_size(env: &mut Environment, identifier: id) -> CGSize {
    let tablet = matches!(env.options.device_family, Some(DeviceFamily::iPad));
    let name = to_rust_string(env, identifier);
    let (width, height) = match name.as_ref() {
        "ADBannerContentSizeIdentifier320x50" => (320.0, 50.0),
        "ADBannerContentSizeIdentifier480x32" => (480.0, 32.0),
        "ADBannerContentSizeIdentifierPortrait" => if tablet { (768.0, 66.0) } else { (320.0, 50.0) },
        "ADBannerContentSizeIdentifierLandscape" => if tablet { (1024.0, 66.0) } else { (480.0, 32.0) },
        _ => panic!("Unknown iAd content size identifier {name:?}"),
    };
    CGSize { width, height }
}

const CLASSES: ClassExports = objc_classes! {
(env, this, _cmd);
@implementation ADBannerView: UIView
+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<Banner>::default(), &mut env.mem)
}
+ (CGSize)sizeFromBannerContentSizeIdentifier:(id)identifier { banner_size(env, identifier) }
- (id)initWithFrame:(CGRect)frame {
    let current = get_static_str(env, "ADBannerContentSizeIdentifierPortrait");
    let size = banner_size(env, current);
    let frame = CGRect { size, ..frame };
    let this: id = msg_super![env; this initWithFrame:frame];
    let required: id = msg_class![env; NSSet setWithObject:current];
    let required: id = msg![env; required copy];
    let state = env.objc.borrow_mut::<Banner>(this);
    state.required = required;
    state.current = current;
    () = msg![env; this setOpaque:false];
    // An unloaded banner has no ad to interact with.
    () = msg![env; this setUserInteractionEnabled:false];
    this
}
- (id)delegate { env.objc.borrow::<Banner>(this).delegate }
- (())setDelegate:(id)delegate {
    let state = env.objc.borrow_mut::<Banner>(this);
    state.delegate = delegate;
    if delegate != nil && !state.failure_queued && !state.failure_delivered {
        state.failure_queued = true;
        let run_loop: id = msg_class![env; NSRunLoop mainRunLoop];
        let selector = env.objc.lookup_selector("_deliverUnavailableAd:").unwrap();
        add_perform_request(env, run_loop, this, selector, nil, Some(0.0), false);
    }
}
- (())_deliverUnavailableAd:(id)_unused {
    let state = env.objc.borrow_mut::<Banner>(this);
    state.failure_queued = false;
    let delegate = state.delegate;
    if state.failure_delivered || delegate == nil { return; }
    state.failure_delivered = true;
    let selector = env.objc.register_host_selector("bannerView:didFailToReceiveAdWithError:".into(), &mut env.mem);
    if msg![env; delegate respondsToSelector:selector] {
        let domain = get_static_str(env, "ADErrorDomain");
        let error: id = msg_class![env; NSError errorWithDomain:domain code:1i32 userInfo:nil];
        log!("iAd unavailable: banner={this:?}, delegate={delegate:?}, ADErrorServerFailure; no ad loaded");
        () = msg![env; delegate bannerView:this didFailToReceiveAdWithError:error];
    }
}
- (bool)isBannerLoaded { false }
- (bool)isBannerViewActionInProgress { false }
- (())cancelBannerViewAction {}
- (id)requiredContentSizeIdentifiers { env.objc.borrow::<Banner>(this).required }
- (())setRequiredContentSizeIdentifiers:(id)value {
    assert_ne!(value, nil);
    let copy: id = msg![env; value copy];
    let old = std::mem::replace(&mut env.objc.borrow_mut::<Banner>(this).required, copy);
    release(env, old);
}
- (id)currentContentSizeIdentifier { env.objc.borrow::<Banner>(this).current }
- (())setCurrentContentSizeIdentifier:(id)value {
    let size = banner_size(env, value);
    let copy: id = msg![env; value copy];
    let old = std::mem::replace(&mut env.objc.borrow_mut::<Banner>(this).current, copy);
    release(env, old);
    let mut frame: CGRect = msg![env; this frame];
    frame.size = size;
    () = msg![env; this setFrame:frame];
}
- (id)advertisingSection { env.objc.borrow::<Banner>(this).section }
- (())setAdvertisingSection:(id)value {
    let copy: id = msg![env; value copy];
    let old = std::mem::replace(&mut env.objc.borrow_mut::<Banner>(this).section, copy);
    release(env, old);
}
- (())dealloc {
    let state = env.objc.borrow_mut::<Banner>(this);
    let (required, current, section) = (state.required, state.current, state.section);
    release(env, required);
    release(env, current);
    release(env, section);
    msg_super![env; this dealloc]
}
@end
};
