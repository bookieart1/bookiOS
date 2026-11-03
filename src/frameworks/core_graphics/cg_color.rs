/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGColor.h`

use std::ops::{Add, Mul, Sub};

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::core_graphics::cg_color_space::{
    get_static_color_space, kCGColorSpaceGenericRGB, CGColorSpaceHostObject, CGColorSpaceRef,
};
use crate::frameworks::core_graphics::CGFloat;
use crate::mem::{guest_size_of, ConstPtr, MutPtr, Ptr};
use crate::objc::{objc_classes, ClassExports, HostObject, ObjC};
use crate::Environment;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGColor seems to be a CFType-based type, but in our implementation
// those are just Objective-C types, so we need a class for it, but its name is
// not visible anywhere.
@implementation _touchHLE_CGColor: NSObject

- (())dealloc {
    let components = env.objc.borrow::<CGColorHostObject>(this).components;
    if !components.is_null() {
        env.mem.free(components.cast());
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

#[derive(Copy, Clone)]
pub struct CGColorHostObject {
    pub color_space_name: &'static str,
    // this assumes usage of CGColorSpaceGenericRGB
    // TODO: support other color spaces
    pub r: CGFloat,
    pub g: CGFloat,
    pub b: CGFloat,
    pub a: CGFloat,
    /// Stable guest-visible storage returned by CGColorGetComponents. Copies
    /// used only for host-side animation interpolation leave this null.
    pub components: MutPtr<CGFloat>,
}
impl HostObject for CGColorHostObject {}
// Implemented to aid animation code.
// Theres are the operations needed for the interpolation.
impl Mul<f32> for CGColorHostObject {
    type Output = CGColorHostObject;

    fn mul(self, rhs: f32) -> Self::Output {
        CGColorHostObject {
            color_space_name: self.color_space_name,
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
            components: Ptr::null(),
        }
    }
}
impl Add<CGColorHostObject> for CGColorHostObject {
    type Output = CGColorHostObject;

    fn add(self, rhs: CGColorHostObject) -> Self::Output {
        CGColorHostObject {
            color_space_name: self.color_space_name,
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
            components: Ptr::null(),
        }
    }
}
impl Sub<CGColorHostObject> for CGColorHostObject {
    type Output = CGColorHostObject;

    fn sub(self, rhs: CGColorHostObject) -> Self::Output {
        CGColorHostObject {
            color_space_name: self.color_space_name,
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
            components: Ptr::null(),
        }
    }
}

pub type CGColorRef = CFTypeRef;
pub fn CGColorRelease(env: &mut Environment, c: CGColorRef) {
    if !c.is_null() {
        CFRelease(env, c);
    }
}
pub fn CGColorRetain(env: &mut Environment, c: CGColorRef) -> CGColorRef {
    if !c.is_null() {
        CFRetain(env, c)
    } else {
        c
    }
}

fn CGColorCreate(
    env: &mut Environment,
    space: CGColorSpaceRef,
    components: MutPtr<CGFloat>,
) -> CGColorRef {
    let color_space = env.objc.borrow::<CGColorSpaceHostObject>(space).name;
    assert_eq!(color_space, kCGColorSpaceGenericRGB);
    let r = env.mem.read(components);
    let g = env.mem.read(components + 1);
    let b = env.mem.read(components + 2);
    let a = env.mem.read(components + 3);
    from_rgba(env, (r, g, b, a))
}

fn CGColorCreateGenericRGB(
    env: &mut Environment,
    r: CGFloat,
    g: CGFloat,
    b: CGFloat,
    a: CGFloat,
) -> CGColorRef {
    from_rgba(env, (r, g, b, a))
}

fn CGColorEqualToColor(env: &mut Environment, a: CGColorRef, b: CGColorRef) -> bool {
    to_rgba(&env.objc, a) == to_rgba(&env.objc, b)
}

/// Returns the color space associated with the color. This follows Core
/// Foundation's Get rule: the result is borrowed and remains owned by the
/// framework rather than being newly created for the caller.
fn CGColorGetColorSpace(env: &mut Environment, color: CGColorRef) -> CGColorSpaceRef {
    let color_space_name = env
        .objc
        .borrow::<CGColorHostObject>(color)
        .color_space_name;
    get_static_color_space(env, color_space_name)
}

fn CGColorGetComponents(env: &mut Environment, color: CGColorRef) -> ConstPtr<CGFloat> {
    env.objc
        .borrow::<CGColorHostObject>(color)
        .components
        .cast_const()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGColorRetain(_)),
    export_c_func!(CGColorRelease(_)),
    export_c_func!(CGColorCreate(_, _)),
    export_c_func!(CGColorCreateGenericRGB(_, _, _, _)),
    export_c_func!(CGColorEqualToColor(_, _)),
    export_c_func!(CGColorGetColorSpace(_)),
    export_c_func!(CGColorGetComponents(_)),
];

/// Shortcut for use by `UIColor`: directly construct a `CGColor` instance from
/// an rgba tuple of CGFloats.
pub fn from_rgba(env: &mut Environment, rgba: (CGFloat, CGFloat, CGFloat, CGFloat)) -> CGColorRef {
    let (r, g, b, a) = rgba;
    let components: MutPtr<CGFloat> = env.mem.alloc(4 * guest_size_of::<CGFloat>()).cast();
    env.mem.write(components, r);
    env.mem.write(components + 1, g);
    env.mem.write(components + 2, b);
    env.mem.write(components + 3, a);
    let host_obj = Box::new(CGColorHostObject {
        color_space_name: kCGColorSpaceGenericRGB,
        r,
        g,
        b,
        a,
        components,
    });
    let class = env.objc.get_known_class("_touchHLE_CGColor", &mut env.mem);
    env.objc.alloc_object(class, host_obj, &mut env.mem)
}

/// Shortcut for use by `UIColor`
pub fn to_rgba(objc: &ObjC, color: CGColorRef) -> (CGFloat, CGFloat, CGFloat, CGFloat) {
    let &CGColorHostObject {
        color_space_name,
        r,
        g,
        b,
        a,
        ..
    } = objc.borrow(color);
    assert_eq!(color_space_name, kCGColorSpaceGenericRGB);
    (r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::core_graphics::cg_color_space::{
        kCGColorSpaceModelRGB, CGColorSpaceGetModel,
    };

    #[test]
    fn color_space_getter_returns_a_stable_borrowed_rgb_space() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let color = from_rgba(&mut env, (0.1, 0.2, 0.3, 1.0));
        let first = CGColorGetColorSpace(&mut env, color);
        let second = CGColorGetColorSpace(&mut env, color);
        assert_eq!(first, second);
        assert_eq!(CGColorSpaceGetModel(&mut env, first), kCGColorSpaceModelRGB);

        CGColorRelease(&mut env, color);
        // Get-rule results are not owned by the caller, but this canonical
        // associated space remains valid after the source color is released.
        assert_eq!(CGColorSpaceGetModel(&mut env, first), kCGColorSpaceModelRGB);
    }

    #[test]
    fn component_getter_returns_stable_guest_rgba_storage() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let color = from_rgba(&mut env, (0.125, 0.25, 0.5, 0.75));
        let first = CGColorGetComponents(&mut env, color);
        let second = CGColorGetComponents(&mut env, color);
        assert_eq!(first, second);
        assert_eq!(env.mem.read(first), 0.125);
        assert_eq!(env.mem.read(first + 1), 0.25);
        assert_eq!(env.mem.read(first + 2), 0.5);
        assert_eq!(env.mem.read(first + 3), 0.75);
        CGColorRelease(&mut env, color);
    }
}
