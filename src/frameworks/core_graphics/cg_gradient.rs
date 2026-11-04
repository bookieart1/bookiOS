/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGGradient.h` creation and ownership.

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::core_graphics::cg_color::{to_rgba, CGColorRef};
use crate::frameworks::core_graphics::cg_color_space::{
    kCGColorSpaceModelMonochrome, kCGColorSpaceModelRGB, CGColorSpaceGetModel,
    CGColorSpaceGetNumberOfComponents, CGColorSpaceRef,
};
use crate::frameworks::core_graphics::CGFloat;
use crate::mem::{ConstPtr, GuestUSize};
use crate::objc::{id, msg, objc_classes, ClassExports, HostObject};
use crate::Environment;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation _touchHLE_CGGradient: NSObject
@end

};

#[derive(Debug)]
pub(super) struct CGGradientHostObject {
    /// Snapshotting is intentional: Core Graphics gradients own their stop
    /// data independently of the caller's temporary array and component data.
    pub(super) stops: Vec<(CGFloat, (CGFloat, CGFloat, CGFloat, CGFloat))>,
}
impl HostObject for CGGradientHostObject {}

pub type CGGradientRef = CFTypeRef;

fn CGGradientCreateWithColors(
    env: &mut Environment,
    color_space: CGColorSpaceRef,
    colors: CFTypeRef,
    locations: ConstPtr<CGFloat>,
) -> CGGradientRef {
    if colors.is_null() {
        return CGGradientRef::null();
    }
    if !color_space.is_null() && CGColorSpaceGetModel(env, color_space) != kCGColorSpaceModelRGB {
        return CGGradientRef::null();
    }

    let count: u32 = msg![env; colors count];
    if count == 0 {
        return CGGradientRef::null();
    }
    let mut stops = Vec::with_capacity(count as usize);
    for index in 0..count {
        let color: id = msg![env; colors objectAtIndex:index];
        if color.is_null() {
            return CGGradientRef::null();
        }
        let color: CGColorRef = color;
        let location = if locations.is_null() {
            if count == 1 {
                0.0
            } else {
                index as CGFloat / (count - 1) as CGFloat
            }
        } else {
            env.mem.read(locations + index)
        };
        if !location.is_finite() {
            return CGGradientRef::null();
        }
        stops.push((location, to_rgba(&env.objc, color)));
    }

    let class = env
        .objc
        .get_known_class("_touchHLE_CGGradient", &mut env.mem);
    env.objc.alloc_object(
        class,
        Box::new(CGGradientHostObject { stops }),
        &mut env.mem,
    )
}

fn CGGradientCreateWithColorComponents(
    env: &mut Environment,
    color_space: CGColorSpaceRef,
    components: ConstPtr<CGFloat>,
    locations: ConstPtr<CGFloat>,
    count: GuestUSize,
) -> CGGradientRef {
    if color_space.is_null() || components.is_null() || count == 0 {
        return CGGradientRef::null();
    }
    let model = CGColorSpaceGetModel(env, color_space);
    if !matches!(model, kCGColorSpaceModelRGB | kCGColorSpaceModelMonochrome) {
        return CGGradientRef::null();
    }
    let color_component_count = CGColorSpaceGetNumberOfComponents(env, color_space);
    // Core Graphics component arrays include alpha after the color-space
    // components: RGBA for RGB and gray/alpha for monochrome.
    let components_per_stop = color_component_count.checked_add(1).unwrap();
    let mut stops = Vec::with_capacity(count as usize);
    let mut previous_location = None;
    for index in 0..count {
        let base = index.checked_mul(components_per_stop).unwrap();
        let component = |offset| env.mem.read(components + base + offset);
        let rgba = match model {
            kCGColorSpaceModelRGB => (component(0), component(1), component(2), component(3)),
            kCGColorSpaceModelMonochrome => {
                let gray = component(0);
                (gray, gray, gray, component(1))
            }
            _ => unreachable!(),
        };
        if ![rgba.0, rgba.1, rgba.2, rgba.3]
            .into_iter()
            .all(CGFloat::is_finite)
        {
            return CGGradientRef::null();
        }
        let location = if locations.is_null() {
            if count == 1 {
                0.0
            } else {
                index as CGFloat / (count - 1) as CGFloat
            }
        } else {
            env.mem.read(locations + index)
        };
        if !location.is_finite()
            || !(0.0..=1.0).contains(&location)
            || previous_location.is_some_and(|previous| location < previous)
        {
            return CGGradientRef::null();
        }
        previous_location = Some(location);
        stops.push((location, rgba));
    }

    let class = env
        .objc
        .get_known_class("_touchHLE_CGGradient", &mut env.mem);
    env.objc.alloc_object(
        class,
        Box::new(CGGradientHostObject { stops }),
        &mut env.mem,
    )
}

fn CGGradientRetain(env: &mut Environment, gradient: CGGradientRef) -> CGGradientRef {
    if gradient.is_null() {
        gradient
    } else {
        CFRetain(env, gradient)
    }
}

fn CGGradientRelease(env: &mut Environment, gradient: CGGradientRef) {
    if !gradient.is_null() {
        CFRelease(env, gradient);
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGGradientCreateWithColorComponents(_, _, _, _)),
    export_c_func!(CGGradientCreateWithColors(_, _, _)),
    export_c_func!(CGGradientRetain(_)),
    export_c_func!(CGGradientRelease(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::core_graphics::cg_color::from_rgba;
    use crate::frameworks::foundation::ns_array;
    use crate::objc::release;

    #[test]
    fn colors_are_snapshotted_with_even_default_locations() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let first = from_rgba(&mut env, (1.0, 0.0, 0.0, 1.0));
        let second = from_rgba(&mut env, (0.0, 0.0, 1.0, 1.0));
        let colors = ns_array::from_vec(&mut env, vec![first, second]);
        let gradient =
            CGGradientCreateWithColors(&mut env, CGColorSpaceRef::null(), colors, ConstPtr::null());
        let stops = &env.objc.borrow::<CGGradientHostObject>(gradient).stops;
        assert_eq!(stops[0].0, 0.0);
        assert_eq!(stops[1].0, 1.0);
        assert_eq!(stops[0].1, (1.0, 0.0, 0.0, 1.0));
        CGGradientRelease(&mut env, gradient);
        release(&mut env, colors);
    }

    #[test]
    fn component_gradient_snapshots_rgba_and_locations() {
        use crate::frameworks::core_graphics::cg_color_space::{
            CGColorSpaceCreateDeviceRGB, CGColorSpaceRelease,
        };

        let mut env = Environment::new_fake_for_test_with_thread();
        let color_space = CGColorSpaceCreateDeviceRGB(&mut env);
        let component_values = [1.0f32, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0, 0.25];
        let components = env.mem.alloc(8 * 4).cast::<f32>();
        for (index, value) in component_values.into_iter().enumerate() {
            env.mem.write(components + index as u32, value);
        }
        let locations = env.mem.alloc(2 * 4).cast::<f32>();
        env.mem.write(locations, 0.25f32);
        env.mem.write(locations + 1, 0.75f32);
        let gradient = CGGradientCreateWithColorComponents(
            &mut env,
            color_space,
            components.cast::<CGFloat>().cast_const(),
            locations.cast::<CGFloat>().cast_const(),
            2,
        );
        let stops = &env.objc.borrow::<CGGradientHostObject>(gradient).stops;
        assert_eq!(stops, &vec![(0.25, (1.0, 0.0, 0.0, 1.0)), (0.75, (0.0, 0.5, 1.0, 0.25))]);
        CGGradientRelease(&mut env, gradient);
        CGColorSpaceRelease(&mut env, color_space);
    }
}
