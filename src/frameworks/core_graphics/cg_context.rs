/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGContext.h`

use super::cg_affine_transform::{CGAffineTransform, CGAffineTransformIdentity};
use super::cg_bitmap_context::{
    draw_rect_with_line_cap, CGBitmapContextDrawer, CGBitmapContextGetHeight,
};
use super::cg_color::CGColorRef;
use super::cg_color_space::{
    kCGColorSpaceModelMonochrome, kCGColorSpaceModelRGB, CGColorSpaceGetModel, CGColorSpaceRef,
};
use super::cg_font::{
    CGFontCreateWithFontName, CGFontHostObject, CGFontRef, CGFontRelease, CGFontRetain, CGGlyph,
};
use super::cg_gradient::{CGGradientHostObject, CGGradientRef};
use super::cg_image::CGImageRef;
use super::cg_path::{
    kCGLineCapButt, kCGLineCapRound, kCGLineCapSquare, kCGPathElementAddLineToPoint,
    kCGPathElementCloseSubpath, kCGPathElementMoveToPoint, CGLineCap, CGMutablePathHostObject,
    CGMutablePathRef, CGPathCreateMutable, CGPathElement,
};
use super::{cg_bitmap_context, cg_color, CGFloat, CGPoint, CGRect, CGSize};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::foundation::ns_string::from_rust_string;
use crate::frameworks::uikit;
use crate::mem::{ConstPtr, ConstVoidPtr, GuestUSize};
use crate::objc::{objc_classes, release, ClassExports, HostObject};
use crate::Environment;
use encoding_rs::Encoding;

type CGInterpolationQuality = i32;
type CGGradientDrawingOptions = u32;
const K_CG_GRADIENT_DRAWS_BEFORE_START_LOCATION: CGGradientDrawingOptions = 1;
const K_CG_GRADIENT_DRAWS_AFTER_END_LOCATION: CGGradientDrawingOptions = 2;

type CGTextDrawingMode = i32;
pub(super) const kCGTextFill: CGTextDrawingMode = 0;
const kCGTextStroke: CGTextDrawingMode = 1;
const kCGTextFillStroke: CGTextDrawingMode = 2;
const kCGTextInvisible: CGTextDrawingMode = 3;
const kCGTextFillClip: CGTextDrawingMode = 4;
const kCGTextStrokeClip: CGTextDrawingMode = 5;
const kCGTextFillStrokeClip: CGTextDrawingMode = 6;
const kCGTextClip: CGTextDrawingMode = 7;

type CGTextEncoding = i32;
const kCGEncodingFontSpecific: CGTextEncoding = 0;
const kCGEncodingMacRoman: CGTextEncoding = 1;

pub type CGBlendMode = i32;
pub const kCGBlendModeNormal: CGBlendMode = 0;
pub const kCGBlendModeMultiply: CGBlendMode = 1;
pub const kCGBlendModeScreen: CGBlendMode = 2;
pub const kCGBlendModeOverlay: CGBlendMode = 3;
pub const kCGBlendModeDarken: CGBlendMode = 4;
pub const kCGBlendModeLighten: CGBlendMode = 5;
pub const kCGBlendModeColorDodge: CGBlendMode = 6;
pub const kCGBlendModeColorBurn: CGBlendMode = 7;
pub const kCGBlendModeSoftLight: CGBlendMode = 8;
pub const kCGBlendModeHardLight: CGBlendMode = 9;
pub const kCGBlendModeDifference: CGBlendMode = 10;
pub const kCGBlendModeExclusion: CGBlendMode = 11;
pub const kCGBlendModeHue: CGBlendMode = 12;
pub const kCGBlendModeSaturation: CGBlendMode = 13;
pub const kCGBlendModeColor: CGBlendMode = 14;
pub const kCGBlendModeLuminosity: CGBlendMode = 15;
pub const kCGBlendModeClear: CGBlendMode = 16;
pub const kCGBlendModeCopy: CGBlendMode = 17;
pub const kCGBlendModeSourceIn: CGBlendMode = 18;
pub const kCGBlendModeSourceOut: CGBlendMode = 19;
pub const kCGBlendModeSourceAtop: CGBlendMode = 20;
pub const kCGBlendModeDestinationOver: CGBlendMode = 21;
pub const kCGBlendModeDestinationIn: CGBlendMode = 22;
pub const kCGBlendModeDestinationOut: CGBlendMode = 23;
pub const kCGBlendModeDestinationAtop: CGBlendMode = 24;
pub const kCGBlendModeXOR: CGBlendMode = 25;
pub const kCGBlendModePlusDarker: CGBlendMode = 26;
pub const kCGBlendModePlusLighter: CGBlendMode = 27;

// Helper function for logging blend modes
// It also allowes us to define blend mode conststs we are not using yet!
pub(super) fn blend_mode_name(mode: CGBlendMode) -> &'static str {
    match mode {
        kCGBlendModeNormal => "Normal",
        kCGBlendModeMultiply => "Multiply",
        kCGBlendModeScreen => "Screen",
        kCGBlendModeOverlay => "Overlay",
        kCGBlendModeDarken => "Darken",
        kCGBlendModeLighten => "Lighten",
        kCGBlendModeColorDodge => "ColorDodge",
        kCGBlendModeColorBurn => "ColorBurn",
        kCGBlendModeSoftLight => "SoftLight",
        kCGBlendModeHardLight => "HardLight",
        kCGBlendModeDifference => "Difference",
        kCGBlendModeExclusion => "Exclusion",
        kCGBlendModeHue => "Hue",
        kCGBlendModeSaturation => "Saturation",
        kCGBlendModeColor => "Color",
        kCGBlendModeLuminosity => "Luminosity",
        kCGBlendModeClear => "Clear",
        kCGBlendModeCopy => "Copy",
        kCGBlendModeSourceIn => "SourceIn",
        kCGBlendModeSourceOut => "SourceOut",
        kCGBlendModeSourceAtop => "SourceAtop",
        kCGBlendModeDestinationOver => "DestinationOver",
        kCGBlendModeDestinationIn => "DestinationIn",
        kCGBlendModeDestinationOut => "DestinationOut",
        kCGBlendModeDestinationAtop => "DestinationAtop",
        kCGBlendModeXOR => "XOR",
        kCGBlendModePlusDarker => "PlusDarker",
        kCGBlendModePlusLighter => "PlusLighter",
        _ => "Unknown",
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGContext seems to be a CFType-based type, but in our implementation those
// are just Objective-C types, so we need a class for it, but its name is not
// visible anywhere.
@implementation _touchHLE_CGContext: NSObject

- (())dealloc {
    let host_obj = env.objc.borrow::<CGContextHostObject>(this);
    let CGContextSubclass::CGBitmapContext(bitmap_data) = host_obj.subclass;
    if bitmap_data.data_is_owned {
        env.mem.free(bitmap_data.data);
    }
    CGFontRelease(env, host_obj.font);

    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

/// One clipping operation, stored in device-space coordinates. Successive
/// operations are intersected by requiring a pixel to be inside every entry.
/// Each entry may contain several subpaths; their combined nonzero winding
/// number determines whether the pixel is inside that operation.
#[derive(Clone, Debug)]
pub(super) struct ClipPath {
    subpaths: Vec<Vec<CGPoint>>,
    bounds: CGRect,
}

impl ClipPath {
    fn from_subpaths(subpaths: Vec<Vec<CGPoint>>) -> Self {
        let mut points = subpaths.iter().flatten();
        let first = points.next().copied().unwrap_or_default();
        let (mut min_x, mut max_x) = (first.x, first.x);
        let (mut min_y, mut max_y) = (first.y, first.y);
        for point in points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        ClipPath {
            subpaths,
            bounds: CGRect {
                origin: CGPoint { x: min_x, y: min_y },
                size: CGSize {
                    width: max_x - min_x,
                    height: max_y - min_y,
                },
            },
        }
    }

    /// Test the nonzero winding rule at a device-space point.
    pub(super) fn contains(&self, point: CGPoint) -> bool {
        if self.subpaths.is_empty()
            || point.x < self.bounds.origin.x
            || point.y < self.bounds.origin.y
            || point.x > self.bounds.origin.x + self.bounds.size.width
            || point.y > self.bounds.origin.y + self.bounds.size.height
        {
            return false;
        }

        let mut winding = 0i32;
        for subpath in &self.subpaths {
            if subpath.len() < 3 {
                continue;
            }
            for (&start, &end) in subpath
                .iter()
                .zip(subpath.iter().cycle().skip(1))
                .take(subpath.len())
            {
                let side = (end.x - start.x) * (point.y - start.y)
                    - (point.x - start.x) * (end.y - start.y);
                if start.y <= point.y {
                    if end.y > point.y && side > 0.0 {
                        winding += 1;
                    }
                } else if end.y <= point.y && side < 0.0 {
                    winding -= 1;
                }
            }
        }
        winding != 0
    }
}

fn clip_path_from_elements(
    elements: &[CGPathElement],
    transform: CGAffineTransform,
) -> Option<ClipPath> {
    if elements.is_empty() {
        return None;
    }

    let mut subpaths = Vec::new();
    let mut current = Vec::new();
    for element in elements {
        match element.r#type {
            kCGPathElementMoveToPoint => {
                if !current.is_empty() {
                    subpaths.push(std::mem::take(&mut current));
                }
                if let Some(&point) = element.points.first() {
                    current.push(transform.apply_to_point(point));
                }
            }
            kCGPathElementAddLineToPoint => {
                if let Some(&point) = element.points.first() {
                    current.push(transform.apply_to_point(point));
                }
            }
            kCGPathElementCloseSubpath => {
                if !current.is_empty() {
                    subpaths.push(std::mem::take(&mut current));
                }
            }
            _ => unimplemented!("clipping this Core Graphics path element"),
        }
    }
    if !current.is_empty() {
        subpaths.push(current);
    }

    Some(ClipPath::from_subpaths(subpaths))
}

// TODO: keep more states saved once they are implemented
type ContextState = (
    (CGFloat, CGFloat, CGFloat, CGFloat), // RGB fill color
    CGAffineTransform,                    // transform
    CGFontRef,                            // font
    CGFloat,                              // font size
    CGBlendMode,                          // blend mode
    CGLineCap,                            // line cap
    (CGFloat, CGFloat, CGFloat, CGFloat), // RGB stroke color
    Vec<ClipPath>,                        // clipping path intersections
    CGTextDrawingMode,                    // text drawing mode
    CGTextEncoding,                       // legacy byte-to-glyph encoding
);

pub(super) struct CGContextHostObject {
    pub(super) subclass: CGContextSubclass,
    pub(super) rgb_fill_color: (CGFloat, CGFloat, CGFloat, CGFloat),
    pub(super) font: CGFontRef,
    pub(super) font_size: CGFloat,
    /// Current transform.
    pub(super) transform: CGAffineTransform,
    pub(super) blend_mode: CGBlendMode,
    /// Text transform.
    pub(super) text_transform: Option<CGAffineTransform>,
    /// Current text position in user-space coordinates. The text matrix and
    /// position are context attributes rather than saved graphics state.
    pub(super) text_position: CGPoint,
    pub(super) text_drawing_mode: CGTextDrawingMode,
    pub(super) text_encoding: CGTextEncoding,
    /// Line/Path data
    pub(super) line_cap: CGLineCap,
    pub(super) line_width: CGFloat,
    pub(super) state_stack: Vec<ContextState>,
    pub(super) rgb_stroke_color: (CGFloat, CGFloat, CGFloat, CGFloat),
    pub(super) path: CGMutablePathRef,
    pub(super) clip_paths: Vec<ClipPath>,
}
impl HostObject for CGContextHostObject {}

pub(super) enum CGContextSubclass {
    CGBitmapContext(cg_bitmap_context::CGBitmapContextData),
}

pub type CGContextRef = CFTypeRef;

pub fn CGContextFlush(_env: &mut Environment, ctx: CGContextRef) {
    log!("TODO: CGContextFlush(ctx={:?})", ctx);
}

pub fn CGContextRelease(env: &mut Environment, c: CGContextRef) {
    if !c.is_null() {
        CFRelease(env, c);
    }
}
pub fn CGContextRetain(env: &mut Environment, c: CGContextRef) -> CGContextRef {
    if !c.is_null() {
        CFRetain(env, c)
    } else {
        c
    }
}

fn CGContextBeginPath(env: &mut Environment, context: CGContextRef) {
    let new_path = CGPathCreateMutable(env);
    let old_path = std::mem::replace(
        &mut env.objc.borrow_mut::<CGContextHostObject>(context).path,
        new_path,
    );
    if !old_path.is_null() {
        CFRelease(env, old_path);
    }
}

fn CGContextDrawLinearGradient(
    env: &mut Environment,
    context: CGContextRef,
    gradient: CGGradientRef,
    start: CGPoint,
    end: CGPoint,
    options: CGGradientDrawingOptions,
) {
    assert!(!gradient.is_null());
    let stops = env
        .objc
        .borrow::<CGGradientHostObject>(gradient)
        .stops
        .clone();
    if stops.is_empty() {
        return;
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return;
    }
    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    for y in 0..drawer.height() as i32 {
        for x in 0..drawer.width() as i32 {
            let point = drawer.user_point_at_pixel(x, y);
            let raw_t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared;
            if (raw_t < 0.0 && options & K_CG_GRADIENT_DRAWS_BEFORE_START_LOCATION == 0)
                || (raw_t > 1.0 && options & K_CG_GRADIENT_DRAWS_AFTER_END_LOCATION == 0)
            {
                continue;
            }
            let t = raw_t.clamp(0.0, 1.0);
            let mut lower = stops[0];
            let mut upper = stops[stops.len() - 1];
            for pair in stops.windows(2) {
                if t >= pair[0].0 && t <= pair[1].0 {
                    lower = pair[0];
                    upper = pair[1];
                    break;
                }
            }
            let span = upper.0 - lower.0;
            let u = if span > 0.0 {
                ((t - lower.0) / span).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let lerp = |a: CGFloat, b: CGFloat| a + (b - a) * u;
            let srgb = (
                lerp(lower.1 .0, upper.1 .0),
                lerp(lower.1 .1, upper.1 .1),
                lerp(lower.1 .2, upper.1 .2),
                lerp(lower.1 .3, upper.1 .3),
            );
            let color = drawer.linear_color(srgb);
            drawer.put_pixel((x, y), color, true);
        }
    }
}

pub fn CGContextSelectFont(
    env: &mut Environment,
    ctx: CGContextRef,
    name_ptr: ConstVoidPtr, // const char * (guest)
    size: CGFloat,
    encoding: i32, // CGTextEncoding
) {
    assert!(matches!(
        encoding,
        kCGEncodingFontSpecific | kCGEncodingMacRoman
    ));
    if name_ptr.is_null() {
        return;
    }
    let name = env
        .mem
        .cstr_at_utf8(name_ptr.cast())
        .expect("CGContextSelectFont font name is not UTF-8")
        .to_owned();
    let name_string = from_rust_string(env, name.clone());
    let font = CGFontCreateWithFontName(env, name_string);
    release(env, name_string);
    if font.is_null() {
        log!("CGContextSelectFont: no font named {name:?}");
        return;
    }

    CGContextSetFont(env, ctx, font);
    CGContextSetFontSize(env, ctx, size);
    CGFontRelease(env, font);
    env.objc
        .borrow_mut::<CGContextHostObject>(ctx)
        .text_encoding = encoding;
}

fn CGContextSetBlendMode(env: &mut Environment, context: CGContextRef, blend_mode: CGBlendMode) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .blend_mode = blend_mode;
}

fn CGContextSetFillColorSpace(
    env: &mut Environment,
    _context: CGContextRef,
    space: CGColorSpaceRef,
) {
    let color_model = CGColorSpaceGetModel(env, space);
    assert!(color_model == kCGColorSpaceModelMonochrome || color_model == kCGColorSpaceModelRGB);
    // TODO
}

pub fn CGContextSetCharacterSpacing(_env: &mut Environment, ctx: CGContextRef, spacing: CGFloat) {
    log!(
        "TODO: CGContextSetCharacterSpacing(ctx={:?}, spacing={})",
        ctx,
        spacing
    );
}

fn CGContextSetFillColorWithColor(env: &mut Environment, context: CGContextRef, color: CGColorRef) {
    let (r, g, b, a) = cg_color::to_rgba(&env.objc, color);
    CGContextSetRGBFillColor(env, context, r, g, b, a)
}

pub fn CGContextSetRGBFillColor(
    env: &mut Environment,
    context: CGContextRef,
    red: CGFloat,
    green: CGFloat,
    blue: CGFloat,
    alpha: CGFloat,
) {
    let color = (red, green, blue, alpha);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = color;
}

fn CGContextSetGrayFillColor(
    env: &mut Environment,
    context: CGContextRef,
    gray: CGFloat,
    alpha: CGFloat,
) {
    let color = (gray, gray, gray, alpha);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = color;
}

fn CGContextSetGrayStrokeColor(
    _env: &mut Environment,
    context: CGContextRef,
    gray: CGFloat,
    alpha: CGFloat,
) {
    log!(
        "TODO: CGContextSetGrayStrokeColor({:?}, {}, {})",
        context,
        gray,
        alpha,
    );
}
fn CGContextSetRGBStrokeColor(
    env: &mut Environment,
    context: CGContextRef,
    r: CGFloat,
    g: CGFloat,
    b: CGFloat,
    a: CGFloat,
) {
    let color = (r, g, b, a);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_stroke_color = color;
}

fn CGContextSetShadowWithColor(
    _env: &mut Environment,
    context: CGContextRef,
    offset: CGSize,
    blur: CGFloat,
    color: CGColorRef,
) {
    log!(
        "TODO: CGContextSetShadowWithColor({:?}, {}, {}, {:?})",
        context,
        offset,
        blur,
        color
    );
}

pub fn CGContextFillRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    cg_bitmap_context::fill_rect(env, context, rect, /* clear: */ false);
}

/// Fill the current path with the nonzero winding rule and then clear it.
/// Path points are transformed to device space at draw time, matching the
/// existing clipping and bitmap drawing machinery.
fn CGContextFillPath(env: &mut Environment, context: CGContextRef) {
    let (path, transform) = {
        let host = env.objc.borrow::<CGContextHostObject>(context);
        (host.path, host.transform)
    };
    if path.is_null() {
        return;
    }
    let elements = env
        .objc
        .borrow::<CGMutablePathHostObject>(path)
        .path_elements
        .clone();
    let fill = clip_path_from_elements(&elements, transform);
    CGContextBeginPath(env, context);
    let Some(fill) = fill else {
        return;
    };

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let color = drawer.rgb_fill_color();
    let min_x = fill.bounds.origin.x.floor().max(0.0) as i32;
    let min_y = fill.bounds.origin.y.floor().max(0.0) as i32;
    let max_x = (fill.bounds.origin.x + fill.bounds.size.width)
        .ceil()
        .min(drawer.width() as CGFloat) as i32;
    let max_y = (fill.bounds.origin.y + fill.bounds.size.height)
        .ceil()
        .min(drawer.height() as CGFloat) as i32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let point = CGPoint {
                x: x as CGFloat + 0.5,
                y: y as CGFloat + 0.5,
            };
            if fill.contains(point) {
                drawer.put_pixel((x, y), color, true);
            }
        }
    }
}

pub fn CGContextClearRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    cg_bitmap_context::fill_rect(env, context, rect, /* clear: */ true);
}

pub fn CGContextFillEllipseInRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    cg_bitmap_context::fill_ellipse(env, context, rect);
}

pub fn CGContextClipToRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    let transform = env.objc.borrow::<CGContextHostObject>(context).transform;
    let corners = vec![
        rect.origin,
        CGPoint {
            x: rect.origin.x + rect.size.width,
            y: rect.origin.y,
        },
        CGPoint {
            x: rect.origin.x + rect.size.width,
            y: rect.origin.y + rect.size.height,
        },
        CGPoint {
            x: rect.origin.x,
            y: rect.origin.y + rect.size.height,
        },
    ]
    .into_iter()
    .map(|point| transform.apply_to_point(point))
    .collect();
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .clip_paths
        .push(ClipPath::from_subpaths(vec![corners]));
}

/// Intersect the current clipping path with the current path using the
/// nonzero winding rule, then reset the current path. Open subpaths are closed
/// implicitly by [ClipPath::contains].
fn CGContextClip(env: &mut Environment, context: CGContextRef) {
    let (path, transform) = {
        let host = env.objc.borrow::<CGContextHostObject>(context);
        (host.path, host.transform)
    };
    if path.is_null() {
        return;
    }
    let elements = env
        .objc
        .borrow::<CGMutablePathHostObject>(path)
        .path_elements
        .clone();
    let Some(clip) = clip_path_from_elements(&elements, transform) else {
        return;
    };
    let new_path = CGPathCreateMutable(env);
    let host = env.objc.borrow_mut::<CGContextHostObject>(context);
    host.clip_paths.push(clip);
    host.path = new_path;
}

pub fn CGContextConcatCTM(
    env: &mut Environment,
    context: CGContextRef,
    transform: CGAffineTransform,
) {
    log_dbg!("CGContextConcatCTM({:?})", transform);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = transform.concat(host_obj.transform);
}
pub fn CGContextGetCTM(env: &mut Environment, context: CGContextRef) -> CGAffineTransform {
    let res = env.objc.borrow::<CGContextHostObject>(context).transform;
    log_dbg!("CGContextGetCTM() => {:?}", res);
    res
}
/// Not a real Core Graphics function: replaces the CTM, for host code that
/// changes it temporarily.
pub fn set_ctm(env: &mut Environment, context: CGContextRef, transform: CGAffineTransform) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .transform = transform;
}
pub fn CGContextGetUserSpaceToDeviceSpaceTransform(
    env: &mut Environment,
    context: CGContextRef,
) -> CGAffineTransform {
    let ctm = CGContextGetCTM(env, context);
    let host_obj = env.objc.borrow::<CGContextHostObject>(context);
    #[allow(unreachable_patterns)]
    match &host_obj.subclass {
        CGContextSubclass::CGBitmapContext(_) => {
            let height = CGBitmapContextGetHeight(env, context) as CGFloat;
            let flip = CGAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: -1.0,
                tx: 0.0,
                ty: height,
            };
            flip.concat(ctm)
        }
        _ => ctm,
    }
}
pub fn CGContextRotateCTM(env: &mut Environment, context: CGContextRef, angle: CGFloat) {
    log_dbg!("CGContextRotateCTM({:?})", angle);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.rotate(angle);
}
pub fn CGContextScaleCTM(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    log_dbg!("CGContextScaleCTM({:?})", (x, y));
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.scale(x, y);
}
pub fn CGContextTranslateCTM(
    env: &mut Environment,
    context: CGContextRef,
    tx: CGFloat,
    ty: CGFloat,
) {
    log_dbg!("CGContextTranslateCTM({:?})", (tx, ty));
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.translate(tx, ty);
}

pub fn CGContextDrawImage(
    env: &mut Environment,
    context: CGContextRef,
    rect: CGRect,
    image: CGImageRef,
) {
    cg_bitmap_context::draw_image(env, context, rect, image);
}

pub fn CGContextSaveGState(env: &mut Environment, context: CGContextRef) {
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.state_stack.push((
        host_obj.rgb_fill_color,
        host_obj.transform,
        host_obj.font,
        host_obj.font_size,
        host_obj.blend_mode,
        host_obj.line_cap,
        host_obj.rgb_stroke_color,
        host_obj.clip_paths.clone(),
        host_obj.text_drawing_mode,
        host_obj.text_encoding,
    ));
    CGFontRetain(env, env.objc.borrow::<CGContextHostObject>(context).font);
}

pub fn CGContextRestoreGState(env: &mut Environment, context: CGContextRef) {
    // We need to release _old_ font, there are 2 cases:
    // - font hasn't been set between save/restore -> this release corresponds
    // the font retain from save
    // - font has been set between save/restore -> we need to release old font
    // retained on the set
    CGFontRelease(env, env.objc.borrow::<CGContextHostObject>(context).font);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    let state = host_obj.state_stack.pop().unwrap();
    host_obj.rgb_fill_color = state.0;
    host_obj.transform = state.1;
    host_obj.font = state.2;
    host_obj.font_size = state.3;
    host_obj.blend_mode = state.4;
    host_obj.line_cap = state.5;
    host_obj.rgb_stroke_color = state.6;
    host_obj.clip_paths = state.7;
    host_obj.text_drawing_mode = state.8;
    host_obj.text_encoding = state.9;
}

fn CGContextSetInterpolationQuality(
    _env: &mut Environment,
    context: CGContextRef,
    quality: CGInterpolationQuality,
) {
    log!(
        "TODO: CGContextSetInterpolationQuality({:?}, {:?})",
        context,
        quality
    );
}
fn CGContextSetAllowsAntialiasing(_env: &mut Environment, context: CGContextRef, allow: bool) {
    log!(
        "TODO: CGContextSetAllowsAntialiasing({:?}, {})",
        context,
        allow
    );
}

fn CGContextSetShouldSmoothFonts(_env: &mut Environment, context: CGContextRef, should: bool) {
    log!(
        "TODO: CGContextSetShouldSmoothFonts({:?}, {})",
        context,
        should
    );
}

fn CGContextSetAlpha(_env: &mut Environment, context: CGContextRef, alpha: CGFloat) {
    // Since 1.0 is the default value for alpha, we can just ignore it
    if alpha != 1.0 {
        log!("TODO: CGContextSetAlpha({:?}, {:?})", context, alpha);
    }
}

fn CGContextSetFont(env: &mut Environment, context: CGContextRef, font: CGFontRef) {
    CGFontRetain(env, font);
    let old_font = env.objc.borrow_mut::<CGContextHostObject>(context).font;
    CGFontRelease(env, old_font);
    env.objc.borrow_mut::<CGContextHostObject>(context).font = font;
}

fn CGContextSetFontSize(env: &mut Environment, context: CGContextRef, size: CGFloat) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .font_size = size;
}

fn CGContextSetTextDrawingMode(
    env: &mut Environment,
    context: CGContextRef,
    mode: CGTextDrawingMode,
) {
    assert!((kCGTextFill..=kCGTextClip).contains(&mode));
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .text_drawing_mode = mode;
}

fn text_mode_draws_fill(mode: CGTextDrawingMode) -> bool {
    matches!(
        mode,
        kCGTextFill | kCGTextFillStroke | kCGTextFillClip | kCGTextFillStrokeClip
    )
}

fn CGContextSetTextMatrix(
    env: &mut Environment,
    context: CGContextRef,
    transform: CGAffineTransform,
) {
    log_dbg!("CGContextSetTextMatrix({:?})", transform);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .text_transform = Some(transform);
}

fn CGContextSetTextPosition(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .text_position = CGPoint { x, y };
}

fn CGContextGetTextPosition(env: &mut Environment, context: CGContextRef) -> CGPoint {
    env.objc
        .borrow::<CGContextHostObject>(context)
        .text_position
}

fn advanced_text_position(
    origin: CGPoint,
    horizontal_advance: CGFloat,
    text_transform: CGAffineTransform,
) -> CGPoint {
    let transformed_advance = text_transform.apply_to_size(CGSize {
        width: horizontal_advance,
        height: 0.0,
    });
    CGPoint {
        x: origin.x + transformed_advance.width,
        y: origin.y + transformed_advance.height,
    }
}

#[cfg(test)]
mod text_drawing_mode_tests {
    use super::*;

    #[test]
    fn only_modes_with_a_fill_component_rasterize_with_current_support() {
        assert!(text_mode_draws_fill(kCGTextFill));
        assert!(!text_mode_draws_fill(kCGTextStroke));
        assert!(text_mode_draws_fill(kCGTextFillStroke));
        assert!(!text_mode_draws_fill(kCGTextInvisible));
        assert!(text_mode_draws_fill(kCGTextFillClip));
        assert!(!text_mode_draws_fill(kCGTextStrokeClip));
        assert!(text_mode_draws_fill(kCGTextFillStrokeClip));
        assert!(!text_mode_draws_fill(kCGTextClip));
    }

    #[test]
    fn mac_roman_decoder_preserves_ascii_text() {
        assert_eq!(
            decode_mac_roman(b"Cube"),
            vec![b'C' as u16, b'u' as u16, b'b' as u16, b'e' as u16]
        );
    }

    #[test]
    fn text_advance_follows_the_linear_text_transform() {
        let origin = CGPoint { x: 3.0, y: 5.0 };
        let transform = CGAffineTransform {
            a: 0.0,
            b: 2.0,
            c: -1.0,
            d: 0.0,
            tx: 99.0,
            ty: 101.0,
        };
        assert_eq!(
            advanced_text_position(origin, 4.0, transform),
            CGPoint { x: 3.0, y: 13.0 }
        );
    }
}

fn CGContextShowGlyphsAtPoint(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    glyphs: ConstPtr<CGGlyph>,
    count: GuestUSize,
) {
    let mut glyph_ids = Vec::new();
    for i in 0..count {
        let glyph_id = env.mem.read(glyphs + i);
        glyph_ids.push(rusttype::GlyphId(glyph_id));
    }
    CGContextShowGlyphIdsAtPoint(env, context, x, y, glyph_ids)
}

fn CGContextShowTextAtPoint(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    text: ConstPtr<u8>,
    length: GuestUSize,
) {
    let encoding = env
        .objc
        .borrow::<CGContextHostObject>(context)
        .text_encoding;
    let bytes = env.mem.bytes_at(text.cast(), length).to_vec();
    let font_ref = env.objc.borrow::<CGContextHostObject>(context).font;
    if font_ref.is_null() {
        log!("CGContextShowTextAtPoint called without a selected font");
        return;
    }
    let font = &env.objc.borrow::<CGFontHostObject>(font_ref).font;
    let glyph_ids = match encoding {
        // Font-specific text treats each byte as the font's glyph index.
        kCGEncodingFontSpecific => bytes
            .into_iter()
            .map(|byte| rusttype::GlyphId(byte.into()))
            .collect(),
        kCGEncodingMacRoman => decode_mac_roman(&bytes)
            .into_iter()
            .map(|unit| font.glyph_id_for_char(unit))
            .collect(),
        _ => unreachable!(),
    };
    CGContextShowGlyphIdsAtPoint(env, context, x, y, glyph_ids)
}

fn decode_mac_roman(bytes: &[u8]) -> Vec<u16> {
    let encoding = Encoding::for_label(b"macintosh").expect("encoding_rs has Macintosh");
    let (decoded, _, _) = encoding.decode(bytes);
    decoded.encode_utf16().collect()
}

fn CGContextShowGlyphIdsAtPoint(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    glyph_ids: Vec<rusttype::GlyphId>,
) {
    let (text_drawing_mode, font_ref, font_size, text_transform) = {
        let host = env.objc.borrow::<CGContextHostObject>(context);
        (
            host.text_drawing_mode,
            host.font,
            host.font_size,
            host.text_transform.unwrap_or(CGAffineTransformIdentity),
        )
    };
    if font_ref.is_null() {
        log!("CGContext text drawing called without a selected font");
        return;
    }

    let horizontal_advance = env
        .objc
        .borrow::<CGFontHostObject>(font_ref)
        .font
        .glyphs_horizontal_advance(font_size, glyph_ids.iter().copied());
    let origin = CGPoint { x, y };
    let text_position = advanced_text_position(origin, horizontal_advance, text_transform);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .text_position = text_position;

    if matches!(text_drawing_mode, kCGTextInvisible | kCGTextClip) {
        return;
    }
    if !text_mode_draws_fill(text_drawing_mode) {
        if matches!(text_drawing_mode, kCGTextStroke | kCGTextStrokeClip) {
            log!("TODO: stroke-only Core Graphics text drawing");
        }
        // Stroke-only modes are valid, but touchHLE does not rasterize them.
        return;
    }
    if matches!(text_drawing_mode, kCGTextFillClip | kCGTextFillStrokeClip) {
        log!("TODO: Core Graphics text clipping");
    }

    let font = &env.objc.borrow::<CGFontHostObject>(font_ref).font;

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();

    font.draw_glyphs(
        font_size,
        glyph_ids,
        (x, y),
        text_transform,
        |raster_glyph| {
            uikit::ui_font::draw_font_glyph(
                &mut drawer,
                raster_glyph,
                fill_color,
                /* clip_x: */ None,
                /* clip_y: */ None,
            )
        },
    );
}

fn CGContextShowGlyphsAtPositions(
    env: &mut Environment,
    context: CGContextRef,
    glyphs: ConstPtr<CGGlyph>,
    positions: ConstPtr<CGPoint>,
    count: GuestUSize,
) {
    let text_transform = env
        .objc
        .borrow::<CGContextHostObject>(context)
        .text_transform
        .unwrap_or(CGAffineTransformIdentity);
    assert!(text_transform.tx == 0.0 && text_transform.ty == 0.0); // TODO

    for i in 0..count {
        let glyph_ptr = glyphs + i;
        let point = env.mem.read(positions + i);
        let transformed_point = text_transform.apply_to_point(point);
        CGContextShowGlyphsAtPoint(
            env,
            context,
            transformed_point.x,
            transformed_point.y,
            glyph_ptr,
            1,
        );
    }
}

fn CGContextSetLineCap(env: &mut Environment, context: CGContextRef, cap: CGLineCap) {
    log_dbg!("CGContextSetLineCap({:?}, {})", context, cap);

    assert!(cap == kCGLineCapButt || cap == kCGLineCapRound || cap == kCGLineCapSquare);
    env.objc.borrow_mut::<CGContextHostObject>(context).line_cap = cap;
}

fn CGContextSetLineWidth(env: &mut Environment, context: CGContextRef, width: CGFloat) {
    log_dbg!("CGContextSetLineCap({:?}, {})", context, width,);

    assert!(width > 0.0);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .line_width = width;
}

fn CGContextSetStrokeColorWithColor(
    env: &mut Environment,
    context: CGContextRef,
    color: CGColorRef,
) {
    let (r, g, b, a) = cg_color::to_rgba(&env.objc, color);
    CGContextSetRGBStrokeColor(env, context, r, g, b, a)
}

fn CGContextMoveToPoint(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    log_dbg!("CGContextMoveToPoint({:?}, {}, {})", context, x, y);

    let host_obj = if env
        .objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .is_null()
    {
        CGPathCreateMutable(env)
    } else {
        env.objc.borrow_mut::<CGContextHostObject>(context).path
    };

    env.objc
        .borrow_mut::<CGMutablePathHostObject>(host_obj)
        .path_elements
        .insert(
            0,
            CGPathElement {
                points: vec![CGPoint { x, y }],
                r#type: kCGPathElementMoveToPoint,
            },
        );

    env.objc.borrow_mut::<CGContextHostObject>(context).path = host_obj;
}

fn CGContextAddLineToPoint(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    log_dbg!("CGContextAddLineToPoint({:?}, {}, {})", context, x, y);

    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context).path;

    env.objc
        .borrow_mut::<CGMutablePathHostObject>(host_obj)
        .path_elements
        .push(CGPathElement {
            points: vec![CGPoint { x, y }],
            r#type: kCGPathElementAddLineToPoint,
        });

    env.objc.borrow_mut::<CGContextHostObject>(context).path = host_obj;
}

/// Append a path object to the context's current path without drawing it.
///
/// Core Graphics path objects are immutable from the caller's perspective, so
/// the context must copy the elements rather than retain the source as its
/// mutable current path. Coordinates remain in user space here; the context's
/// transform is applied later when the current path is rendered.
fn CGContextAddPath(env: &mut Environment, context: CGContextRef, path: CGMutablePathRef) {
    log_dbg!("CGContextAddPath({:?}, {:?})", context, path);
    assert!(!path.is_null());

    let elements = env
        .objc
        .borrow::<CGMutablePathHostObject>(path)
        .path_elements
        .clone();
    let mut current_path = env.objc.borrow::<CGContextHostObject>(context).path;
    if current_path.is_null() {
        current_path = CGPathCreateMutable(env);
        env.objc.borrow_mut::<CGContextHostObject>(context).path = current_path;
    }
    env.objc
        .borrow_mut::<CGMutablePathHostObject>(current_path)
        .path_elements
        .extend(elements);
}

// NOTE: We might be able to replace the drawing functionality here
//       with a 3rd party 2D graphics lib
fn CGContextStrokePath(env: &mut Environment, context: CGContextRef) {
    log_dbg!("CGContextStrokePath({:?})", context);

    let line_cap = env.objc.borrow::<CGContextHostObject>(context).line_cap;
    let line_width = env.objc.borrow::<CGContextHostObject>(context).line_width;
    let line_size = CGSize {
        width: line_width,
        height: line_width,
    };

    let (r, g, b, a) = env
        .objc
        .borrow::<CGContextHostObject>(context)
        .rgb_stroke_color;
    CGContextSetRGBFillColor(env, context, r, g, b, a);

    let host_obj = env.objc.borrow::<CGContextHostObject>(context).path;
    let path_elements = &env
        .objc
        .borrow_mut::<CGMutablePathHostObject>(host_obj)
        .path_elements
        .clone();

    let mut current_point = None;
    let mut subpath_start = None;

    for elem in path_elements {
        if elem.r#type == kCGPathElementMoveToPoint {
            current_point = elem.points.first().copied();
            subpath_start = current_point;
        } else {
            let next_point = match elem.r#type {
                kCGPathElementAddLineToPoint => elem.points.first().copied(),
                kCGPathElementCloseSubpath => subpath_start,
                _ => unimplemented!(),
            };
            let (Some(point_0), Some(point_1)) = (current_point, next_point) else {
                continue;
            };

            draw_rect_with_line_cap(
                env,
                context,
                line_cap,
                CGRect {
                    origin: point_0,
                    size: line_size,
                },
            );

            // Not sure if this is accurate
            let step_limit = 25;

            for step in 0..step_limit {
                let t = step as f32 / step_limit as f32;
                let bezier_curve_point = CGPoint {
                    x: (1.0 - t) * point_0.x + t * point_1.x,
                    y: (1.0 - t) * point_0.y + t * point_1.y,
                };

                draw_rect_with_line_cap(
                    env,
                    context,
                    line_cap,
                    CGRect {
                        origin: bezier_curve_point,
                        size: line_size,
                    },
                );
            }

            draw_rect_with_line_cap(
                env,
                context,
                line_cap,
                CGRect {
                    origin: point_1,
                    size: line_size,
                },
            );

            current_point = Some(point_1);
        }
    }

    env.objc.borrow_mut::<CGContextHostObject>(context).path = CGPathCreateMutable(env);
}

#[cfg(test)]
mod clipping_tests {
    use super::*;

    fn point(x: CGFloat, y: CGFloat) -> CGPoint {
        CGPoint { x, y }
    }

    #[test]
    fn nonzero_winding_combines_subpaths_and_honors_holes() {
        let outer = vec![
            point(0.0, 0.0),
            point(10.0, 0.0),
            point(10.0, 10.0),
            point(0.0, 10.0),
        ];
        let reverse_inner = vec![
            point(3.0, 3.0),
            point(3.0, 7.0),
            point(7.0, 7.0),
            point(7.0, 3.0),
        ];
        let clip = ClipPath::from_subpaths(vec![outer, reverse_inner]);

        assert!(clip.contains(point(1.0, 1.0)));
        assert!(!clip.contains(point(5.0, 5.0)));
        assert!(!clip.contains(point(11.0, 5.0)));
    }

    #[test]
    fn current_path_points_are_frozen_in_transformed_device_space() {
        let elements = vec![
            CGPathElement {
                points: vec![point(0.0, 0.0)],
                r#type: kCGPathElementMoveToPoint,
            },
            CGPathElement {
                points: vec![point(2.0, 0.0)],
                r#type: kCGPathElementAddLineToPoint,
            },
            CGPathElement {
                points: vec![point(2.0, 2.0)],
                r#type: kCGPathElementAddLineToPoint,
            },
            CGPathElement {
                points: vec![point(0.0, 2.0)],
                r#type: kCGPathElementAddLineToPoint,
            },
        ];
        let clip =
            clip_path_from_elements(&elements, CGAffineTransform::make_translation(5.0, 7.0))
                .unwrap();

        assert!(clip.contains(point(6.0, 8.0)));
        assert!(!clip.contains(point(1.0, 1.0)));
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGContextBeginPath(_)),
    export_c_func!(CGContextDrawLinearGradient(_, _, _, _, _)),
    export_c_func!(CGContextFlush(_)),
    export_c_func!(CGContextRetain(_)),
    export_c_func!(CGContextSelectFont(_, _, _, _)),
    export_c_func!(CGContextRelease(_)),
    export_c_func!(CGContextSetBlendMode(_, _)),
    export_c_func!(CGContextSetFillColorSpace(_, _)),
    export_c_func!(CGContextSetCharacterSpacing(_, _)),
    export_c_func!(CGContextSetFillColorWithColor(_, _)),
    export_c_func!(CGContextSetRGBFillColor(_, _, _, _, _)),
    export_c_func!(CGContextSetGrayFillColor(_, _, _)),
    export_c_func!(CGContextSetGrayStrokeColor(_, _, _)),
    export_c_func!(CGContextSetStrokeColorWithColor(_, _)),
    export_c_func!(CGContextSetRGBStrokeColor(_, _, _, _, _)),
    export_c_func!(CGContextSetShadowWithColor(_, _, _, _)),
    export_c_func!(CGContextFillRect(_, _)),
    export_c_func!(CGContextFillPath(_)),
    export_c_func!(CGContextClearRect(_, _)),
    export_c_func!(CGContextClip(_)),
    export_c_func!(CGContextClipToRect(_, _)),
    export_c_func!(CGContextConcatCTM(_, _)),
    export_c_func!(CGContextGetCTM(_)),
    export_c_func!(CGContextGetUserSpaceToDeviceSpaceTransform(_)),
    export_c_func!(CGContextRotateCTM(_, _)),
    export_c_func!(CGContextScaleCTM(_, _, _)),
    export_c_func!(CGContextTranslateCTM(_, _, _)),
    export_c_func!(CGContextDrawImage(_, _, _)),
    export_c_func!(CGContextSaveGState(_)),
    export_c_func!(CGContextRestoreGState(_)),
    export_c_func!(CGContextSetInterpolationQuality(_, _)),
    export_c_func!(CGContextSetAllowsAntialiasing(_, _)),
    export_c_func!(CGContextSetShouldSmoothFonts(_, _)),
    export_c_func!(CGContextSetAlpha(_, _)),
    export_c_func!(CGContextSetFont(_, _)),
    export_c_func!(CGContextSetFontSize(_, _)),
    export_c_func!(CGContextSetTextDrawingMode(_, _)),
    export_c_func!(CGContextSetTextMatrix(_, _)),
    export_c_func!(CGContextSetTextPosition(_, _, _)),
    export_c_func!(CGContextGetTextPosition(_)),
    export_c_func!(CGContextShowGlyphsAtPoint(_, _, _, _, _)),
    export_c_func!(CGContextShowGlyphsAtPositions(_, _, _, _)),
    export_c_func!(CGContextShowTextAtPoint(_, _, _, _, _)),
    export_c_func!(CGContextSetLineCap(_, _)),
    export_c_func!(CGContextSetLineWidth(_, _)),
    export_c_func!(CGContextMoveToPoint(_, _, _)),
    export_c_func!(CGContextAddLineToPoint(_, _, _)),
    export_c_func!(CGContextAddPath(_, _)),
    export_c_func!(CGContextStrokePath(_)),
    export_c_func!(CGContextFillEllipseInRect(_, _)),
];
