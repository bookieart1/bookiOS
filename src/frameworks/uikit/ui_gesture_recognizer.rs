/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The pre-iOS-5 gesture recognizer model used by early UIKit games.

use super::ui_event::UIEventHostObject;
use super::ui_touch::{
    UITouchHostObject, UITouchPhase, UITouchPhaseEnded, UITouchPhaseMoved,
};
use crate::frameworks::core_graphics::{CGFloat, CGPoint};
use crate::frameworks::foundation::{NSInteger, NSTimeInterval, NSUInteger};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_send, msg_super, nil, objc_classes, Class,
    ClassExports, HostObject, NSZonePtr, SEL,
};
use crate::Environment;

pub type UIGestureRecognizerState = NSInteger;
pub const UIGestureRecognizerStatePossible: UIGestureRecognizerState = 0;
pub const UIGestureRecognizerStateBegan: UIGestureRecognizerState = 1;
pub const UIGestureRecognizerStateChanged: UIGestureRecognizerState = 2;
pub const UIGestureRecognizerStateEnded: UIGestureRecognizerState = 3;
pub const UIGestureRecognizerStateCancelled: UIGestureRecognizerState = 4;
pub const UIGestureRecognizerStateFailed: UIGestureRecognizerState = 5;

#[derive(Default)]
struct UIGestureRecognizerHostObject {
    /// Target-action targets are non-retaining, as in UIKit.
    targets: Vec<(id, SEL)>,
    /// The view is retained by the view-recognizer relationship, not here.
    view: id,
    /// UIKit's gesture-recognizer delegate property is assign/non-retaining.
    delegate: id,
    state: UIGestureRecognizerState,
    enabled: bool,
    cancels_touches_in_view: bool,
}
impl HostObject for UIGestureRecognizerHostObject {}

struct UIPinchGestureRecognizerHostObject {
    superclass: UIGestureRecognizerHostObject,
    scale: CGFloat,
    velocity: CGFloat,
    initial_distance: Option<CGFloat>,
    previous_scale: CGFloat,
    previous_timestamp: NSTimeInterval,
}
impl Default for UIPinchGestureRecognizerHostObject {
    fn default() -> Self {
        Self {
            superclass: UIGestureRecognizerHostObject {
                enabled: true,
                cancels_touches_in_view: true,
                ..Default::default()
            },
            scale: 1.0,
            velocity: 0.0,
            initial_distance: None,
            previous_scale: 1.0,
            previous_timestamp: 0.0,
        }
    }
}
impl_HostObject_with_superclass!(UIPinchGestureRecognizerHostObject);

fn active_touches_for_view(env: &mut Environment, event: id, view: id) -> Vec<id> {
    let touches = env.objc.borrow::<UIEventHostObject>(event).touches;
    let array: id = msg![env; touches allObjects];
    let count: NSUInteger = msg![env; array count];
    let mut result = Vec::new();
    for index in 0..count {
        let touch: id = msg![env; array objectAtIndex:index];
        let host = env.objc.borrow::<UITouchHostObject>(touch);
        if host.view == view && host.phase != UITouchPhaseEnded {
            result.push(touch);
        }
    }
    result
}

fn touch_location(env: &Environment, touch: id) -> CGPoint {
    env.objc.borrow::<UITouchHostObject>(touch).location
}

fn send_actions(env: &mut Environment, recognizer: id) {
    let targets = env
        .objc
        .borrow::<UIGestureRecognizerHostObject>(recognizer)
        .targets
        .clone();
    for (target, action) in targets {
        if target != nil && !action.is_null() {
            () = msg_send(env, (target, action, recognizer));
        }
    }
}

pub(super) fn set_view(env: &mut Environment, recognizer: id, view: id) {
    env.objc
        .borrow_mut::<UIGestureRecognizerHostObject>(recognizer)
        .view = view;
}

pub(super) fn view_requires_multiple_touches(env: &mut Environment, view: id) -> bool {
    let recognizers = env
        .objc
        .borrow::<super::ui_view::UIViewHostObject>(view)
        .gesture_recognizers
        .clone();
    let pinch = env.objc.get_known_class("UIPinchGestureRecognizer", &mut env.mem);
    recognizers.into_iter().any(|recognizer| {
        let class: Class = msg![env; recognizer class];
        env.objc.class_is_subclass_of(class, pinch)
            && env
                .objc
                .borrow::<UIGestureRecognizerHostObject>(recognizer)
                .enabled
    })
}

/// Update recognizers attached to `view` before delivering the corresponding
/// responder touch callback. This first implementation supports the polling
/// properties and target-action lifecycle needed by UIPinchGestureRecognizer.
pub(super) fn process_touches(
    env: &mut Environment,
    view: id,
    event: id,
    delivered_phase: UITouchPhase,
) {
    let recognizers = env
        .objc
        .borrow::<super::ui_view::UIViewHostObject>(view)
        .gesture_recognizers
        .clone();
    for recognizer in recognizers {
        let enabled = env
            .objc
            .borrow::<UIGestureRecognizerHostObject>(recognizer)
            .enabled;
        if !enabled {
            continue;
        }
        let class: Class = msg![env; recognizer class];
        let pinch_class = env
            .objc
            .get_known_class("UIPinchGestureRecognizer", &mut env.mem);
        if !env.objc.class_is_subclass_of(class, pinch_class) {
            continue;
        }

        let touches = active_touches_for_view(env, event, view);
        if touches.len() >= 2 {
            let a = touch_location(env, touches[0]);
            let b = touch_location(env, touches[1]);
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let distance = (dx * dx + dy * dy).sqrt();
            let timestamp = env.objc.borrow::<UITouchHostObject>(touches[0]).timestamp;
            let host = env
                .objc
                .borrow_mut::<UIPinchGestureRecognizerHostObject>(recognizer);
            if host.initial_distance.is_none() {
                host.initial_distance = Some(distance.max(CGFloat::EPSILON));
                host.previous_scale = host.scale;
                host.previous_timestamp = timestamp;
                host.superclass.state = UIGestureRecognizerStateBegan;
            } else {
                let next_scale = distance / host.initial_distance.unwrap();
                let elapsed = timestamp - host.previous_timestamp;
                host.velocity = if elapsed > 0.0 {
                    (next_scale - host.previous_scale) / elapsed as CGFloat
                } else {
                    0.0
                };
                host.scale = next_scale;
                host.previous_scale = next_scale;
                host.previous_timestamp = timestamp;
                host.superclass.state = UIGestureRecognizerStateChanged;
            }
            send_actions(env, recognizer);
        } else if delivered_phase == UITouchPhaseEnded {
            let was_active = env
                .objc
                .borrow::<UIPinchGestureRecognizerHostObject>(recognizer)
                .initial_distance
                .is_some();
            if was_active {
                env.objc
                    .borrow_mut::<UIGestureRecognizerHostObject>(recognizer)
                    .state = UIGestureRecognizerStateEnded;
                send_actions(env, recognizer);
                let host = env
                    .objc
                    .borrow_mut::<UIPinchGestureRecognizerHostObject>(recognizer);
                host.initial_distance = None;
                host.velocity = 0.0;
                host.superclass.state = UIGestureRecognizerStatePossible;
            }
        } else if delivered_phase == UITouchPhaseMoved {
            // A single moving touch is not sufficient to begin a pinch.
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIGestureRecognizer: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::new(UIGestureRecognizerHostObject {
        enabled: true,
        cancels_touches_in_view: true,
        ..Default::default()
    }), &mut env.mem)
}

- (id)initWithTarget:(id)target action:(SEL)action {
    if target != nil && !action.is_null() {
        env.objc
            .borrow_mut::<UIGestureRecognizerHostObject>(this)
            .targets
            .push((target, action));
    }
    this
}

- (())addTarget:(id)target action:(SEL)action {
    if target != nil && !action.is_null() {
        env.objc
            .borrow_mut::<UIGestureRecognizerHostObject>(this)
            .targets
            .push((target, action));
    }
}

- (())removeTarget:(id)target action:(SEL)action {
    env.objc
        .borrow_mut::<UIGestureRecognizerHostObject>(this)
        .targets
        .retain(|&(candidate, candidate_action)| {
            !((target == nil || candidate == target)
                && (action.is_null() || candidate_action == action))
        });
}

- (UIGestureRecognizerState)state {
    env.objc.borrow::<UIGestureRecognizerHostObject>(this).state
}
- (id)view {
    env.objc.borrow::<UIGestureRecognizerHostObject>(this).view
}
- (id)delegate {
    env.objc.borrow::<UIGestureRecognizerHostObject>(this).delegate
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIGestureRecognizerHostObject>(this).delegate = delegate;
}
- (bool)isEnabled {
    env.objc.borrow::<UIGestureRecognizerHostObject>(this).enabled
}
- (())setEnabled:(bool)enabled {
    let host = env.objc.borrow_mut::<UIGestureRecognizerHostObject>(this);
    host.enabled = enabled;
    if !enabled {
        host.state = UIGestureRecognizerStatePossible;
    }
}
- (bool)cancelsTouchesInView {
    env.objc.borrow::<UIGestureRecognizerHostObject>(this).cancels_touches_in_view
}
- (())setCancelsTouchesInView:(bool)value {
    env.objc
        .borrow_mut::<UIGestureRecognizerHostObject>(this)
        .cancels_touches_in_view = value;
}
- (NSUInteger)numberOfTouches {
    let view = env.objc.borrow::<UIGestureRecognizerHostObject>(this).view;
    if view == nil { 0 } else {
        env.framework_state
            .uikit
            .ui_touch
            .current_touches
            .values()
            .filter(|&&touch| env.objc.borrow::<UITouchHostObject>(touch).view == view)
            .count() as NSUInteger
    }
}
- (CGPoint)locationInView:(id)view {
    let own_view = env.objc.borrow::<UIGestureRecognizerHostObject>(this).view;
    let points: Vec<CGPoint> = env
        .framework_state
        .uikit
        .ui_touch
        .current_touches
        .values()
        .filter_map(|&touch| {
            let host = env.objc.borrow::<UITouchHostObject>(touch);
            (host.view == own_view).then_some(host.location)
        })
        .collect();
    if points.is_empty() {
        CGPoint { x: 0.0, y: 0.0 }
    } else {
        let screen = points
            .iter()
            .fold(CGPoint { x: 0.0, y: 0.0 }, |sum, point| CGPoint {
                x: sum.x + point.x,
                y: sum.y + point.y,
            });
        let screen = CGPoint {
            x: screen.x / points.len() as CGFloat,
            y: screen.y / points.len() as CGFloat,
        };
        if view == nil { screen } else { msg![env; view convertPoint:screen fromView:nil] }
    }
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation UIPinchGestureRecognizer: UIGestureRecognizer

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UIPinchGestureRecognizerHostObject>::default(), &mut env.mem)
}

- (CGFloat)scale {
    env.objc.borrow::<UIPinchGestureRecognizerHostObject>(this).scale
}
- (())setScale:(CGFloat)scale {
    let host = env.objc.borrow_mut::<UIPinchGestureRecognizerHostObject>(this);
    host.scale = scale;
    host.previous_scale = scale;
}
- (CGFloat)velocity {
    env.objc.borrow::<UIPinchGestureRecognizerHostObject>(this).velocity
}

- (())dealloc {
    let _ = std::mem::take(env.objc.borrow_mut::<UIPinchGestureRecognizerHostObject>(this));
    msg_super![env; this dealloc]
}

@end

};
