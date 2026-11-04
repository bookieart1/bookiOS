/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIProgressView`.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_super, nil, objc_classes, release, retain,
    ClassExports, NSZonePtr,
};

type UIProgressViewStyle = NSInteger;
const UIProgressViewStyleDefault: UIProgressViewStyle = 0;

pub struct UIProgressViewHostObject {
    superclass: super::UIViewHostObject,
    style: UIProgressViewStyle,
    progress: f32,
    progress_tint_color: id,
    track_tint_color: id,
    progress_image: id,
    track_image: id,
}
impl_HostObject_with_superclass!(UIProgressViewHostObject);

impl Default for UIProgressViewHostObject {
    fn default() -> Self {
        Self {
            superclass: Default::default(),
            style: UIProgressViewStyleDefault,
            progress: 0.0,
            progress_tint_color: nil,
            track_tint_color: nil,
            progress_image: nil,
            track_image: nil,
        }
    }
}

fn replace_owned(env: &mut crate::Environment, old: id, value: id) -> id {
    retain(env, value);
    release(env, old);
    value
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIProgressView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UIProgressViewHostObject>::default(), &mut env.mem)
}

- (id)init {
    msg![env; this initWithProgressViewStyle:UIProgressViewStyleDefault]
}

- (id)initWithProgressViewStyle:(UIProgressViewStyle)style {
    let frame = CGRect {
        origin: Default::default(),
        size: crate::frameworks::core_graphics::CGSize {
            width: 0.0,
            height: 2.0,
        },
    };
    let this: id = msg_super![env; this initWithFrame:frame];
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).style = style;
    this
}

- (UIProgressViewStyle)progressViewStyle {
    env.objc.borrow::<UIProgressViewHostObject>(this).style
}
- (())setProgressViewStyle:(UIProgressViewStyle)style {
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).style = style;
    () = msg![env; this setNeedsDisplay];
}

- (f32)progress {
    env.objc.borrow::<UIProgressViewHostObject>(this).progress
}
- (())setProgress:(f32)progress {
    let progress = if progress.is_nan() { 0.0 } else { progress.clamp(0.0, 1.0) };
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).progress = progress;
    () = msg![env; this setNeedsDisplay];
}
- (())setProgress:(f32)progress animated:(bool)_animated {
    // Animation is not yet rendered, but the model value changes immediately
    // just as it does at the start/end of the native animation contract.
    () = msg![env; this setProgress:progress];
}

- (id)progressTintColor { env.objc.borrow::<UIProgressViewHostObject>(this).progress_tint_color }
- (())setProgressTintColor:(id)value {
    let old = env.objc.borrow::<UIProgressViewHostObject>(this).progress_tint_color;
    let value = replace_owned(env, old, value);
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).progress_tint_color = value;
    () = msg![env; this setNeedsDisplay];
}
- (id)trackTintColor { env.objc.borrow::<UIProgressViewHostObject>(this).track_tint_color }
- (())setTrackTintColor:(id)value {
    let old = env.objc.borrow::<UIProgressViewHostObject>(this).track_tint_color;
    let value = replace_owned(env, old, value);
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).track_tint_color = value;
    () = msg![env; this setNeedsDisplay];
}
- (id)progressImage { env.objc.borrow::<UIProgressViewHostObject>(this).progress_image }
- (())setProgressImage:(id)value {
    let old = env.objc.borrow::<UIProgressViewHostObject>(this).progress_image;
    let value = replace_owned(env, old, value);
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).progress_image = value;
    () = msg![env; this setNeedsDisplay];
}
- (id)trackImage { env.objc.borrow::<UIProgressViewHostObject>(this).track_image }
- (())setTrackImage:(id)value {
    let old = env.objc.borrow::<UIProgressViewHostObject>(this).track_image;
    let value = replace_owned(env, old, value);
    env.objc.borrow_mut::<UIProgressViewHostObject>(this).track_image = value;
    () = msg![env; this setNeedsDisplay];
}

- (())dealloc {
    let owned = {
        let host = env.objc.borrow::<UIProgressViewHostObject>(this);
        [host.progress_tint_color, host.track_tint_color, host.progress_image, host.track_image]
    };
    for value in owned { release(env, value); }
    msg_super![env; this dealloc]
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{msg_class, Environment};

    #[test]
    fn progress_is_clamped_and_style_is_stable() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let view: id = msg_class![&mut env; UIProgressView alloc];
        let view: id = msg![&mut env; view initWithProgressViewStyle:1i32];
        () = msg![&mut env; view setProgress:1.5f32 animated:true];
        let progress: f32 = msg![&mut env; view progress];
        let style: NSInteger = msg![&mut env; view progressViewStyle];
        assert_eq!(progress, 1.0);
        assert_eq!(style, 1);
        release(&mut env, view);
    }
}
