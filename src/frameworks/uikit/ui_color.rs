/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIColor`.

use super::ui_graphics::UIGraphicsGetCurrentContext;
use crate::frameworks::core_graphics::cg_color::{CGColorRef, CGColorRelease, CGColorRetain};
use crate::frameworks::core_graphics::cg_context::CGContextSetRGBFillColor;
use crate::frameworks::core_graphics::{cg_color, CGFloat};
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::NSInteger;
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, ClassExports, HostObject, NSZonePtr, ObjC,
    SEL,
};
use crate::Environment;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    standard_colors: HashMap<SEL, id>,
}

fn get_standard_color(
    env: &mut Environment,
    sel: SEL,
    r: CGFloat,
    g: CGFloat,
    b: CGFloat,
    a: CGFloat,
) -> id {
    if let Some(&existing) = env.framework_state.uikit.ui_color.standard_colors.get(&sel) {
        existing
    } else {
        let new: id = msg_class![env; _touchHLE_UIColor_Static alloc];
        let new: id = msg![env; new initWithRed:r green:g blue:b alpha:a];
        env.framework_state
            .uikit
            .ui_color
            .standard_colors
            .insert(sel, new);
        new
    }
}

struct UIColorHostObject {
    cg_color: CGColorRef,
}
impl HostObject for UIColorHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIColor: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIColorHostObject {
        cg_color: nil,
    });
    let object = env.objc.alloc_object(this, host_object, &mut env.mem);
    log!("Development 2.55 UIColor producer trace: alloc object={object:?} class={this:?}");
    object
}

+ (id)colorWithCGColor:(CGColorRef)cg_color {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCGColor:cg_color];
    autorelease(env, new)
}

+ (id)colorWithRed:(CGFloat)r
             green:(CGFloat)g
              blue:(CGFloat)b
             alpha:(CGFloat)a {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithRed:r green:g blue:b alpha:a];
    autorelease(env, new)
}

+ (id)colorWithHue:(CGFloat)h
        saturation:(CGFloat)s
        brightness:(CGFloat)b
             alpha:(CGFloat)a {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithHue:h saturation:s brightness:b alpha:a];
    autorelease(env, new)
}

+ (id)colorWithWhite:(CGFloat)w alpha:(CGFloat)a {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithWhite:w alpha:a];
    autorelease(env, new)
}

+ (id)clearColor    { get_standard_color(env, _cmd, 0.0, 0.0, 0.0, 0.0) }
+ (id)blackColor    { get_standard_color(env, _cmd, 0.0, 0.0, 0.0, 1.0) }
+ (id)whiteColor    { get_standard_color(env, _cmd, 1.0, 1.0, 1.0, 1.0) }
+ (id)darkGrayColor {
    get_standard_color(env, _cmd, 1.0/3.0, 1.0/3.0, 1.0/3.0, 1.0)
}
+ (id)grayColor {
    get_standard_color(env, _cmd, 1.0/2.0, 1.0/2.0, 1.0/2.0, 1.0)
}
+ (id)lightGrayColor {
    get_standard_color(env, _cmd, 2.0/3.0, 2.0/3.0, 2.0/3.0, 1.0)
}
+ (id)blueColor     { get_standard_color(env, _cmd, 0.0, 0.0, 1.0, 1.0) }
+ (id)brownColor    { get_standard_color(env, _cmd, 0.6, 0.4, 0.2, 1.0) }
+ (id)cyanColor     { get_standard_color(env, _cmd, 0.0, 1.0, 1.0, 1.0) }
+ (id)greenColor    { get_standard_color(env, _cmd, 0.0, 1.0, 0.0, 1.0) }
+ (id)magentaColor  { get_standard_color(env, _cmd, 1.0, 0.0, 1.0, 1.0) }
+ (id)orangeColor   { get_standard_color(env, _cmd, 1.0, 0.5, 0.0, 1.0) }
+ (id)purpleColor   { get_standard_color(env, _cmd, 0.5, 0.0, 1.5, 1.0) }
+ (id)redColor      { get_standard_color(env, _cmd, 1.0, 0.0, 0.0, 1.0) }
+ (id)yellowColor   { get_standard_color(env, _cmd, 1.0, 1.0, 0.0, 1.0) }

// TODO: more initializers, set methods, more accessors

- (id)initWithCGColor:(CGColorRef)cg_color {
    let registers = env.cpu.regs();
    log!(
        "Development 2.55 UIColor producer trace: initWithCGColor object={this:?} color={cg_color:?} pc={:#x} lr={:#x}",
        registers[crate::cpu::Cpu::PC],
        registers[crate::cpu::Cpu::LR],
    );
    CGColorRetain(env, cg_color);
    env.objc.borrow_mut::<UIColorHostObject>(this).cg_color = cg_color;
    this
}

- (id)initWithWhite:(CGFloat)w alpha:(CGFloat)a {
    log!("Development 2.55 UIColor producer trace: initWithWhite object={this:?} white={w} alpha={a}");
    let w = w.clamp(0.0, 1.0);
    let a = a.clamp(0.0, 1.0);

    env.objc.borrow_mut::<UIColorHostObject>(this).cg_color = cg_color::from_rgba(env, (w, w, w, a));

    this
}

- (id)initWithRed:(CGFloat)r
            green:(CGFloat)g
             blue:(CGFloat)b
            alpha:(CGFloat)a {
    log!("Development 2.55 UIColor producer trace: initWithRed object={this:?} red={r} green={g} blue={b} alpha={a}");
    env.objc.borrow_mut::<UIColorHostObject>(this).cg_color = cg_color::from_rgba(env, (r, g, b, a));
    this
}

- (id)initWithHue:(CGFloat)h
        saturation:(CGFloat)s
        brightness:(CGFloat)b
             alpha:(CGFloat)a {
    log!("Development 2.55 UIColor producer trace: initWithHue object={this:?} hue={h} saturation={s} brightness={b} alpha={a}");
    let h = h.rem_euclid(1.0) * 6.0;
    let s = s.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);
    let a = a.clamp(0.0, 1.0);

    let (r, g, blue) = if s == 0.0 {
        (b, b, b)
    } else {
        let sector = h.floor() as u32;
        let fraction = h - h.floor();
        let p = b * (1.0 - s);
        let q = b * (1.0 - s * fraction);
        let t = b * (1.0 - s * (1.0 - fraction));
        match sector % 6 {
            0 => (b, t, p),
            1 => (q, b, p),
            2 => (p, b, t),
            3 => (p, q, b),
            4 => (t, p, b),
            5 => (b, p, q),
            _ => unreachable!(),
        }
    };

    msg![env; this initWithRed:r green:g blue:blue alpha:a]
}

// NSCoding implementation
- (())encodeWithCoder:(id)coder {
    let (r, g, b, a) = get_rgba(&env.objc, this);
    () = msg![env; coder encodeFloat:a forKey:(get_static_str(env, "UIAlpha"))];
    if r == g && g == b {
        () = msg![env; coder encodeInteger:2i32 forKey:(get_static_str(env, "UIColorComponentCount"))];
        () = msg![env; coder encodeFloat:r forKey:(get_static_str(env, "UIWhite"))];
    } else {
        () = msg![env; coder encodeInteger:4i32 forKey:(get_static_str(env, "UIColorComponentCount"))];
        () = msg![env; coder encodeFloat:r forKey:(get_static_str(env, "UIRed"))];
        () = msg![env; coder encodeFloat:g forKey:(get_static_str(env, "UIGreen"))];
        () = msg![env; coder encodeFloat:b forKey:(get_static_str(env, "UIBlue"))];
    }
}

- (id)initWithCoder:(id)coder {
    let key_ns_string = get_static_str(env, "UIAlpha");
    let a: CGFloat = msg![env; coder decodeFloatForKey:key_ns_string];

    let key_ns_string = get_static_str(env, "UIColorComponentCount");
    let count: NSInteger = msg![env; coder decodeIntegerForKey:key_ns_string];

    match count {
        4 => {
            let key_ns_string = get_static_str(env, "UIRed");

            // Both RGBA and HSBA colors have 4 components.
            // We assume presence of the red component as the indication of RGBA
            // TODO: support HSBA decoding too
            assert!(msg![env; coder containsValueForKey:key_ns_string]);

            let r: CGFloat = msg![env; coder decodeFloatForKey:key_ns_string];

            let key_ns_string = get_static_str(env, "UIGreen");
            let g: CGFloat = msg![env; coder decodeFloatForKey:key_ns_string];

            let key_ns_string = get_static_str(env, "UIBlue");
            let b: CGFloat = msg![env; coder decodeFloatForKey:key_ns_string];

            log_dbg!(
                "[(UIColor*){:?} initWithCoder:{:?}] => count {}, r {}, g {}, b {}, a {}",
                this, coder, count, r, g, b, a
            );

            msg![env; this initWithRed:r green:g blue:b alpha:a]
        }
        2 => {
            let key_ns_string = get_static_str(env, "UIWhite");
            let w: CGFloat = msg![env; coder decodeFloatForKey:key_ns_string];

            log_dbg!(
                "[(UIColor*){:?} initWithCoder:{:?}] => count {}, w {}, a {}",
                this, coder, count, w, a
            );

            msg![env; this initWithWhite:w alpha:a]
        }
        _ => unimplemented!()
    }
}

- (bool)getRed:(MutPtr<CGFloat>)r
         green:(MutPtr<CGFloat>)g
          blue:(MutPtr<CGFloat>)b
         alpha:(MutPtr<CGFloat>)a {
    let color = env.objc.borrow::<UIColorHostObject>(this).cg_color;
    let (r_, g_, b_, a_) = cg_color::to_rgba(&env.objc, color);
    env.mem.write(r, r_);
    env.mem.write(g, g_);
    env.mem.write(b, b_);
    env.mem.write(a, a_);
    true
}

- (())set {
    msg![env; this setFill]
    // TODO: set stroke color as well
}

- (())setFill {
    let context = UIGraphicsGetCurrentContext(env);
    assert_ne!(context, nil);
    let (r, g, b, a) = get_rgba(&env.objc, this);
    CGContextSetRGBFillColor(env, context, r, g, b, a);
}

- (CGColorRef)CGColor {
    let cg_color = env.objc.borrow::<UIColorHostObject>(this).cg_color;
    if cg_color == nil {
        let registers = env.cpu.regs();
        log!(
            "Development 2.55 UIColor producer trace: CGColor returned null object={this:?} pc={:#x} lr={:#x}",
            registers[crate::cpu::Cpu::PC],
            registers[crate::cpu::Cpu::LR],
        );
    }
    cg_color
}

- (())dealloc {
    let color = env.objc.borrow_mut::<UIColorHostObject>(this).cg_color;
    CGColorRelease(env, color);

    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)colorWithAlphaComponent:(CGFloat)a {
    let a = a.clamp(0.0, 1.0);
    let (r, g, b, _) = get_rgba(&env.objc, this);
    msg_class![env; UIColor colorWithRed:r green:g blue:b alpha:a]
}

// Historical UIKit builds exposed this mutating selector privately. A small
// number of old applications use it directly instead of the public
// colorWithAlphaComponent: copy operation.
- (())setAlpha:(CGFloat)a {
    let a = a.clamp(0.0, 1.0);
    let old = env.objc.borrow::<UIColorHostObject>(this).cg_color;
    let (r, g, b, _) = cg_color::to_rgba(&env.objc, old);
    let new = cg_color::from_rgba(env, (r, g, b, a));
    env.objc.borrow_mut::<UIColorHostObject>(this).cg_color = new;
    CGColorRelease(env, old);
}

@end

// Undocumented classes used in NIBs
@implementation UICGColor: UIColor
@end
@implementation UIDeviceRGBColor: UIColor
@end

// Special subclass for standard colors with a static lifetime.
// See `get_standard_color`.
@implementation _touchHLE_UIColor_Static: UIColor

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIColorHostObject {
        cg_color: nil,
    });
    env.objc.alloc_static_object(this, host_object, &mut env.mem)
}

- (id) retain { this }
- (()) release {}
- (id) autorelease { this }

@end

};

/// Shortcut for use in Core Animation's compositor: get the RGBA triple for a
/// `UIColor*`.
pub fn get_rgba(objc: &ObjC, ui_color: id) -> (CGFloat, CGFloat, CGFloat, CGFloat) {
    let color = objc.borrow::<UIColorHostObject>(ui_color).cg_color;
    cg_color::to_rgba(objc, color)
}
