/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIGraphics.h`

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_graphics::cg_context::{
    CGContextRef, CGContextRelease, CGContextRetain, CGContextScaleCTM,
};
use crate::frameworks::core_graphics::cg_bitmap_context::{
    CGBitmapContextCreate, CGBitmapContextCreateImage,
};
use crate::frameworks::core_graphics::cg_color_space::{
    CGColorSpaceCreateDeviceRGB, CGColorSpaceRelease,
};
use crate::frameworks::core_graphics::cg_image::{
    kCGImageAlphaPremultipliedLast, CGImageRelease,
};
use crate::frameworks::core_graphics::{CGFloat, CGSize};
use crate::mem::Ptr;
use crate::objc::{id, msg_class, nil};
use crate::Environment;

#[derive(Default)]
pub(super) struct State {
    pub(super) context_stack: Vec<CGContextRef>,
    image_contexts: Vec<CGContextRef>,
}

pub fn UIGraphicsPushContext(env: &mut Environment, context: CGContextRef) {
    CGContextRetain(env, context);
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .push(context);
}
pub fn UIGraphicsPopContext(env: &mut Environment) {
    let context = env.framework_state.uikit.ui_graphics.context_stack.pop();
    CGContextRelease(env, context.unwrap());
}
pub fn UIGraphicsGetCurrentContext(env: &mut Environment) -> CGContextRef {
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .last()
        .copied()
        .unwrap_or(nil)
}

fn UIGraphicsBeginImageContext(env: &mut Environment, size: CGSize) {
    UIGraphicsBeginImageContextWithOptions(env, size, false, 1.0)
}

fn UIGraphicsBeginImageContextWithOptions(
    env: &mut Environment,
    size: CGSize,
    opaque: bool,
    scale: CGFloat,
) {
    assert!(size.width > 0.0 && size.height > 0.0);
    let scale = if scale == 0.0 {
        let screen: id = msg_class![env; UIScreen mainScreen];
        crate::objc::msg![env; screen scale]
    } else {
        scale
    };
    assert!(scale > 0.0 && scale.is_finite());
    let width = (size.width * scale).ceil() as u32;
    let height = (size.height * scale).ceil() as u32;
    let color_space = CGColorSpaceCreateDeviceRGB(env);
    let bitmap_info = if opaque {
        crate::frameworks::core_graphics::cg_image::kCGImageAlphaNoneSkipLast
    } else {
        kCGImageAlphaPremultipliedLast
    };
    let context = CGBitmapContextCreate(
        env,
        Ptr::null(),
        width,
        height,
        8,
        0,
        color_space,
        bitmap_info,
    );
    CGColorSpaceRelease(env, color_space);
    assert!(!context.is_null());
    if scale != 1.0 {
        CGContextScaleCTM(env, context, scale, scale);
    }
    UIGraphicsPushContext(env, context);
    CGContextRelease(env, context);
    env.framework_state
        .uikit
        .ui_graphics
        .image_contexts
        .push(context);
}

fn UIGraphicsGetImageFromCurrentImageContext(env: &mut Environment) -> id {
    let context = UIGraphicsGetCurrentContext(env);
    if context.is_null()
        || env
            .framework_state
            .uikit
            .ui_graphics
            .image_contexts
            .last()
            .copied()
            != Some(context)
    {
        return nil;
    }
    let image = CGBitmapContextCreateImage(env, context);
    let ui_image: id = msg_class![env; UIImage imageWithCGImage:image];
    CGImageRelease(env, image);
    ui_image
}

fn UIGraphicsEndImageContext(env: &mut Environment) {
    let context = env
        .framework_state
        .uikit
        .ui_graphics
        .image_contexts
        .pop()
        .expect("UIGraphicsEndImageContext without a matching begin");
    assert_eq!(UIGraphicsGetCurrentContext(env), context);
    UIGraphicsPopContext(env);
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(UIGraphicsPushContext(_)),
    export_c_func!(UIGraphicsPopContext()),
    export_c_func!(UIGraphicsGetCurrentContext()),
    export_c_func!(UIGraphicsBeginImageContext(_)),
    export_c_func!(UIGraphicsBeginImageContextWithOptions(_, _, _)),
    export_c_func!(UIGraphicsGetImageFromCurrentImageContext()),
    export_c_func!(UIGraphicsEndImageContext()),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_244_image_context_lifecycle_is_balanced_and_extracts_image() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        UIGraphicsBeginImageContext(
            env,
            CGSize {
                width: 8.0,
                height: 4.0,
            },
        );
        assert!(!UIGraphicsGetCurrentContext(env).is_null());
        assert_ne!(UIGraphicsGetImageFromCurrentImageContext(env), nil);
        UIGraphicsEndImageContext(env);
        assert!(UIGraphicsGetCurrentContext(env).is_null());
    }
}
