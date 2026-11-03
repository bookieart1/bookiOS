/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGPath`
use super::cg_affine_transform::CGAffineTransform;
use super::CGPoint;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::mem::{ConstPtr, GuestUSize};
use crate::objc::{objc_classes, ClassExports, HostObject};
use crate::Environment;

// Note: on iOS SDK side this type is defined as a pointer to an opaque struct
pub type CGMutablePathRef = CFTypeRef;

#[derive(Clone)]
pub struct CGMutablePathHostObject {
    pub path_elements: Vec<CGPathElement>,
}
impl HostObject for CGMutablePathHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation _touchHLE_CGMutablePath: NSObject
@end

};

pub type CGPathElementType = i32;
pub const kCGPathElementMoveToPoint: CGPathElementType = 0;
pub const kCGPathElementAddLineToPoint: CGPathElementType = 1;
pub const kCGPathElementCloseSubpath: CGPathElementType = 4;

pub type CGLineCap = i32;
pub const kCGLineCapButt: CGLineCap = 0; // Default
pub const kCGLineCapRound: CGLineCap = 1;
pub const kCGLineCapSquare: CGLineCap = 2;

#[derive(Clone)]
pub struct CGPathElement {
    pub points: Vec<CGPoint>,
    pub r#type: CGPathElementType,
}

pub fn CGPathCreateMutable(env: &mut Environment) -> CGMutablePathRef {
    let path_elements: Vec<CGPathElement> = Vec::new();
    let host_obj = Box::new(CGMutablePathHostObject { path_elements });
    let class = env
        .objc
        .get_known_class("_touchHLE_CGMutablePath", &mut env.mem);
    env.objc.alloc_object(class, host_obj, &mut env.mem)
}

pub fn CGPathRetain(env: &mut Environment, path: CGMutablePathRef) -> CGMutablePathRef {
    if !path.is_null() {
        CFRetain(env, path)
    } else {
        path
    }
}
pub fn CGPathRelease(env: &mut Environment, path: CGMutablePathRef) {
    if !path.is_null() {
        CFRelease(env, path);
    }
}

/// Apply the optional transform that CGPath's adding functions take.
fn transformed(env: &Environment, m: ConstPtr<CGAffineTransform>, point: CGPoint) -> CGPoint {
    if m.is_null() {
        point
    } else {
        env.mem.read(m).apply_to_point(point)
    }
}

fn add_element(
    env: &mut Environment,
    path: CGMutablePathRef,
    type_: CGPathElementType,
    point: CGPoint,
) {
    env.objc
        .borrow_mut::<CGMutablePathHostObject>(path)
        .path_elements
        .push(CGPathElement {
            points: vec![point],
            r#type: type_,
        });
}

pub fn CGPathMoveToPoint(
    env: &mut Environment,
    path: CGMutablePathRef,
    m: ConstPtr<CGAffineTransform>,
    x: f32,
    y: f32,
) {
    let point = transformed(env, m, CGPoint { x, y });
    add_element(env, path, kCGPathElementMoveToPoint, point);
}

pub fn CGPathAddLineToPoint(
    env: &mut Environment,
    path: CGMutablePathRef,
    m: ConstPtr<CGAffineTransform>,
    x: f32,
    y: f32,
) {
    let point = transformed(env, m, CGPoint { x, y });
    add_element(env, path, kCGPathElementAddLineToPoint, point);
}

fn current_and_subpath_start(
    path_elements: &[CGPathElement],
) -> (Option<CGPoint>, Option<CGPoint>) {
    let mut current = None;
    let mut subpath_start = None;
    for element in path_elements {
        match element.r#type {
            kCGPathElementMoveToPoint => {
                current = element.points.first().copied();
                subpath_start = current;
            }
            kCGPathElementAddLineToPoint => current = element.points.first().copied(),
            kCGPathElementCloseSubpath => current = subpath_start,
            _ => unreachable!(),
        }
    }
    (current, subpath_start)
}

/// Close the current subpath. As in Core Graphics, the close element itself
/// contains no points; consumers use the latest move-to point to materialize
/// the closing segment.
pub fn CGPathCloseSubpath(env: &mut Environment, path: CGMutablePathRef) {
    let can_close = {
        let host_object = env.objc.borrow::<CGMutablePathHostObject>(path);
        !host_object
            .path_elements
            .last()
            .is_some_and(|element| element.r#type == kCGPathElementCloseSubpath)
            && current_and_subpath_start(&host_object.path_elements)
                .1
                .is_some()
    };

    if can_close {
        env.objc
            .borrow_mut::<CGMutablePathHostObject>(path)
            .path_elements
            .push(CGPathElement {
                points: Vec::new(),
                r#type: kCGPathElementCloseSubpath,
            });
    }
}

pub fn CGPathGetCurrentPoint(env: &mut Environment, path: CGMutablePathRef) -> CGPoint {
    current_and_subpath_start(
        &env.objc
            .borrow::<CGMutablePathHostObject>(path)
            .path_elements,
    )
    .0
    .unwrap_or_default()
}

/// Add a line followed by the circular arc tangent to the lines from the
/// current point to `point1` and from `point1` to `point2`.
///
/// The path renderer currently consumes moves and lines, so the arc is stored
/// as a short polyline. This preserves it for drawing and path animations
/// without introducing a curve element that existing consumers cannot read.
pub fn CGPathAddArcToPoint(
    env: &mut Environment,
    path: CGMutablePathRef,
    m: ConstPtr<CGAffineTransform>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    radius: f32,
) {
    let point1 = transformed(env, m, CGPoint { x: x1, y: y1 });
    let point2 = transformed(env, m, CGPoint { x: x2, y: y2 });
    let current = current_and_subpath_start(
        &env.objc
            .borrow::<CGMutablePathHostObject>(path)
            .path_elements,
    )
    .0;

    let Some(current) = current else {
        add_element(env, path, kCGPathElementMoveToPoint, point1);
        return;
    };

    // Work in the path's stored coordinate space. This is exact for the
    // overwhelmingly common null transform. For an affine transform, scale
    // the radius by the mean transformed basis-vector length.
    let radius = if m.is_null() {
        radius
    } else {
        let transform = env.mem.read(m);
        let x_scale = transform.a.hypot(transform.b);
        let y_scale = transform.c.hypot(transform.d);
        radius * (x_scale + y_scale) * 0.5
    };

    for point in arc_to_points(current, point1, point2, radius) {
        add_element(env, path, kCGPathElementAddLineToPoint, point);
    }
}

fn arc_to_points(
    current: CGPoint,
    point1: CGPoint,
    point2: CGPoint,
    radius: f32,
) -> Vec<CGPoint> {
    const EPSILON: f32 = 1.0e-6;

    let from_corner = CGPoint {
        x: current.x - point1.x,
        y: current.y - point1.y,
    };
    let to_next = CGPoint {
        x: point2.x - point1.x,
        y: point2.y - point1.y,
    };
    let from_len = from_corner.x.hypot(from_corner.y);
    let to_len = to_next.x.hypot(to_next.y);
    if radius <= 0.0 || from_len <= EPSILON || to_len <= EPSILON {
        return vec![point1];
    }

    let from = CGPoint {
        x: from_corner.x / from_len,
        y: from_corner.y / from_len,
    };
    let to = CGPoint {
        x: to_next.x / to_len,
        y: to_next.y / to_len,
    };
    let dot = (from.x * to.x + from.y * to.y).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let half_angle = angle * 0.5;
    let sin_half = half_angle.sin();
    let tan_half = half_angle.tan();
    let cross = from.x * to.y - from.y * to.x;
    if sin_half.abs() <= EPSILON || tan_half.abs() <= EPSILON || cross.abs() <= EPSILON {
        return vec![point1];
    }

    let tangent_distance = radius / tan_half;
    let tangent1 = CGPoint {
        x: point1.x + from.x * tangent_distance,
        y: point1.y + from.y * tangent_distance,
    };
    let tangent2 = CGPoint {
        x: point1.x + to.x * tangent_distance,
        y: point1.y + to.y * tangent_distance,
    };
    let bisector_scale = (2.0 + 2.0 * dot).sqrt();
    if bisector_scale <= EPSILON {
        return vec![point1];
    }
    let bisector = CGPoint {
        x: (from.x + to.x) / bisector_scale,
        y: (from.y + to.y) / bisector_scale,
    };
    let center_distance = radius / sin_half;
    let center = CGPoint {
        x: point1.x + bisector.x * center_distance,
        y: point1.y + bisector.y * center_distance,
    };

    let start_angle = (tangent1.y - center.y).atan2(tangent1.x - center.x);
    let end_angle = (tangent2.y - center.y).atan2(tangent2.x - center.x);
    let mut sweep = end_angle - start_angle;
    if cross < 0.0 {
        while sweep < 0.0 {
            sweep += std::f32::consts::TAU;
        }
    } else {
        while sweep > 0.0 {
            sweep -= std::f32::consts::TAU;
        }
    }

    let segment_count = (sweep.abs() / (std::f32::consts::PI / 16.0))
        .ceil()
        .max(1.0) as usize;
    let mut points = Vec::with_capacity(segment_count + 1);
    points.push(tangent1);
    for i in 1..=segment_count {
        let angle = start_angle + sweep * i as f32 / segment_count as f32;
        points.push(CGPoint {
            x: center.x + radius * angle.cos(),
            y: center.y + radius * angle.sin(),
        });
    }
    *points.last_mut().unwrap() = tangent2;
    points
}

/// Like Apple's: a move to the first point, then lines to the others.
pub fn CGPathAddLines(
    env: &mut Environment,
    path: CGMutablePathRef,
    m: ConstPtr<CGAffineTransform>,
    points: ConstPtr<CGPoint>,
    count: GuestUSize,
) {
    for i in 0..count {
        let point = transformed(env, m, env.mem.read(points + i));
        let type_ = if i == 0 {
            kCGPathElementMoveToPoint
        } else {
            kCGPathElementAddLineToPoint
        };
        add_element(env, path, type_, point);
    }
}

/// The points of a flattened path made of moves, lines and closed subpaths,
/// for keyframe animations. A close operation contributes the subpath's first
/// point so interpolation follows the closing segment.
pub fn path_points(env: &Environment, path: CGMutablePathRef) -> Vec<CGPoint> {
    let host_obj = env.objc.borrow::<CGMutablePathHostObject>(path);
    let mut points = Vec::new();
    let mut subpath_start = None;
    for element in &host_obj.path_elements {
        match element.r#type {
            kCGPathElementMoveToPoint => {
                subpath_start = element.points.first().copied();
                points.extend(subpath_start);
            }
            kCGPathElementAddLineToPoint => points.extend(element.points.first().copied()),
            kCGPathElementCloseSubpath => points.extend(subpath_start),
            _ => unreachable!(),
        }
    }
    points
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGPathCreateMutable()),
    export_c_func!(CGPathRetain(_)),
    export_c_func!(CGPathRelease(_)),
    export_c_func!(CGPathMoveToPoint(_, _, _, _)),
    export_c_func!(CGPathAddLineToPoint(_, _, _, _)),
    export_c_func!(CGPathCloseSubpath(_)),
    export_c_func!(CGPathGetCurrentPoint(_)),
    export_c_func!(CGPathAddArcToPoint(_, _, _, _, _, _, _)),
    export_c_func!(CGPathAddLines(_, _, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_circle_arc_is_tangent_to_both_lines() {
        let points = arc_to_points(
            CGPoint { x: 0.0, y: 0.0 },
            CGPoint { x: 1.0, y: 0.0 },
            CGPoint { x: 1.0, y: 1.0 },
            0.25,
        );
        let first = points.first().unwrap();
        let last = points.last().unwrap();
        assert!((first.x - 0.75).abs() < 1.0e-5);
        assert!(first.y.abs() < 1.0e-5);
        assert!((last.x - 1.0).abs() < 1.0e-5);
        assert!((last.y - 0.25).abs() < 1.0e-5);
        assert!(points.len() > 2);
    }

    #[test]
    fn degenerate_arc_falls_back_to_the_corner() {
        assert_eq!(
            arc_to_points(
                CGPoint { x: 0.0, y: 0.0 },
                CGPoint { x: 1.0, y: 0.0 },
                CGPoint { x: 2.0, y: 0.0 },
                1.0,
            ),
            vec![CGPoint { x: 1.0, y: 0.0 }]
        );
    }

    #[test]
    fn close_element_has_no_points_and_restores_subpath_start() {
        assert_eq!(kCGPathElementCloseSubpath, 4);
        let elements = vec![
            CGPathElement {
                points: vec![CGPoint { x: 2.0, y: 3.0 }],
                r#type: kCGPathElementMoveToPoint,
            },
            CGPathElement {
                points: vec![CGPoint { x: 5.0, y: 7.0 }],
                r#type: kCGPathElementAddLineToPoint,
            },
            CGPathElement {
                points: Vec::new(),
                r#type: kCGPathElementCloseSubpath,
            },
        ];
        assert_eq!(
            current_and_subpath_start(&elements),
            (
                Some(CGPoint { x: 2.0, y: 3.0 }),
                Some(CGPoint { x: 2.0, y: 3.0 })
            )
        );
    }

    #[test]
    fn current_point_tracks_moves_lines_and_closed_subpaths() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let path = CGPathCreateMutable(&mut env);
        assert_eq!(CGPathGetCurrentPoint(&mut env, path), CGPoint::default());
        CGPathMoveToPoint(&mut env, path, ConstPtr::null(), 2.0, 3.0);
        assert_eq!(
            CGPathGetCurrentPoint(&mut env, path),
            CGPoint { x: 2.0, y: 3.0 }
        );
        CGPathAddLineToPoint(&mut env, path, ConstPtr::null(), 5.0, 7.0);
        assert_eq!(
            CGPathGetCurrentPoint(&mut env, path),
            CGPoint { x: 5.0, y: 7.0 }
        );
        CGPathCloseSubpath(&mut env, path);
        assert_eq!(
            CGPathGetCurrentPoint(&mut env, path),
            CGPoint { x: 2.0, y: 3.0 }
        );
        CGPathRelease(&mut env, path);
    }
}
