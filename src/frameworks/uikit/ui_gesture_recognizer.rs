/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The pre-iOS-5 gesture recognizer model used by early UIKit games.

use super::ui_event::UIEventHostObject;
use super::ui_touch::{
    UITouchHostObject, UITouchPhase, UITouchPhaseBegan, UITouchPhaseEnded, UITouchPhaseMoved,
};

#[cfg(test)]
mod dev_233_tests {
    use super::*;

    #[test]
    fn tap_and_pan_recognizers_use_historical_defaults() {
        let tap = UITapGestureRecognizerHostObject::default();
        assert!(tap.superclass.enabled);
        assert_eq!(tap.number_of_taps_required, 1);
        assert_eq!(tap.number_of_touches_required, 1);

        let pan = UIPanGestureRecognizerHostObject::default();
        assert!(pan.superclass.enabled);
        assert_eq!(pan.minimum_number_of_touches, 1);
        assert_eq!(pan.maximum_number_of_touches, NSUInteger::MAX);
        assert_eq!(pan.translation, CGPoint { x: 0.0, y: 0.0 });
    }
}
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

struct UITapGestureRecognizerHostObject {
    superclass: UIGestureRecognizerHostObject,
    number_of_taps_required: NSUInteger,
    number_of_touches_required: NSUInteger,
    initial_location: Option<CGPoint>,
}
impl Default for UITapGestureRecognizerHostObject {
    fn default() -> Self {
        Self {
            superclass: UIGestureRecognizerHostObject {
                enabled: true,
                cancels_touches_in_view: true,
                ..Default::default()
            },
            number_of_taps_required: 1,
            number_of_touches_required: 1,
            initial_location: None,
        }
    }
}
impl_HostObject_with_superclass!(UITapGestureRecognizerHostObject);

struct UIPanGestureRecognizerHostObject {
    superclass: UIGestureRecognizerHostObject,
    minimum_number_of_touches: NSUInteger,
    maximum_number_of_touches: NSUInteger,
    initial_location: Option<CGPoint>,
    previous_location: CGPoint,
    previous_timestamp: NSTimeInterval,
    translation: CGPoint,
    velocity: CGPoint,
}
impl Default for UIPanGestureRecognizerHostObject {
    fn default() -> Self {
        Self {
            superclass: UIGestureRecognizerHostObject {
                enabled: true,
                cancels_touches_in_view: true,
                ..Default::default()
            },
            minimum_number_of_touches: 1,
            maximum_number_of_touches: NSUInteger::MAX,
            initial_location: None,
            previous_location: CGPoint { x: 0.0, y: 0.0 },
            previous_timestamp: 0.0,
            translation: CGPoint { x: 0.0, y: 0.0 },
            velocity: CGPoint { x: 0.0, y: 0.0 },
        }
    }
}
impl_HostObject_with_superclass!(UIPanGestureRecognizerHostObject);

fn active_touches_for_view(env: &mut Environment, event: id, view: id) -> Vec<id> {
    let touches = env.objc.borrow::<UIEventHostObject>(event).touches;
    let array: id = msg![env; touches allObjects];
    let count: NSUInteger = msg![env; array count];
    let mut result = Vec::new();
    for index in 0..count {
        let touch: id = msg![env; array objectAtIndex:index];
        let (touch_view, phase) = {
            let host = env.objc.borrow::<UITouchHostObject>(touch);
            (host.view, host.phase)
        };
        if view_contains_touch_view(env, view, touch_view) && phase != UITouchPhaseEnded {
            result.push(touch);
        }
    }
    result
}

fn touch_location(env: &Environment, touch: id) -> CGPoint {
    env.objc.borrow::<UITouchHostObject>(touch).location
}

fn view_contains_touch_view(env: &mut Environment, ancestor: id, mut touch_view: id) -> bool {
    while touch_view != nil {
        if touch_view == ancestor {
            return true;
        }
        touch_view = msg![env; touch_view superview];
    }
    false
}

fn event_touch_for_view(env: &mut Environment, event: id, view: id) -> Option<id> {
    let touches = env.objc.borrow::<UIEventHostObject>(event).touches;
    let array: id = msg![env; touches allObjects];
    let count: NSUInteger = msg![env; array count];
    (0..count).find_map(|index| {
        let touch: id = msg![env; array objectAtIndex:index];
        let touch_view = env.objc.borrow::<UITouchHostObject>(touch).view;
        view_contains_touch_view(env, view, touch_view).then_some(touch)
    })
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
fn process_recognizers_for_view(
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
        let pinch_class = env.objc.get_known_class("UIPinchGestureRecognizer", &mut env.mem);
        let tap_class = env.objc.get_known_class("UITapGestureRecognizer", &mut env.mem);
        let pan_class = env.objc.get_known_class("UIPanGestureRecognizer", &mut env.mem);

        if env.objc.class_is_subclass_of(class, tap_class) {
            let touch = event_touch_for_view(env, event, view);
            let Some(touch) = touch else { continue };
            let location = touch_location(env, touch);
            match delivered_phase {
                UITouchPhaseBegan => {
                    let host = env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(recognizer);
                    host.initial_location = Some(location);
                    host.superclass.state = UIGestureRecognizerStatePossible;
                }
                UITouchPhaseMoved => {
                    let host = env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(recognizer);
                    if let Some(initial) = host.initial_location {
                        let dx = location.x - initial.x;
                        let dy = location.y - initial.y;
                        if dx * dx + dy * dy > 100.0 {
                            host.superclass.state = UIGestureRecognizerStateFailed;
                        }
                    }
                }
                UITouchPhaseEnded => {
                    let recognized = {
                        let host = env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(recognizer);
                        let recognized = host.initial_location.is_some()
                            && host.superclass.state != UIGestureRecognizerStateFailed
                            && host.number_of_taps_required == 1
                            && host.number_of_touches_required == 1;
                        host.superclass.state = if recognized {
                            UIGestureRecognizerStateEnded
                        } else {
                            UIGestureRecognizerStateFailed
                        };
                        recognized
                    };
                    if recognized { send_actions(env, recognizer); }
                    let host = env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(recognizer);
                    host.initial_location = None;
                    host.superclass.state = UIGestureRecognizerStatePossible;
                }
                _ => {}
            }
            continue;
        }

        if env.objc.class_is_subclass_of(class, pan_class) {
            let touch = event_touch_for_view(env, event, view);
            let Some(touch) = touch else { continue };
            let location = touch_location(env, touch);
            let timestamp = env.objc.borrow::<UITouchHostObject>(touch).timestamp;
            match delivered_phase {
                UITouchPhaseBegan => {
                    let host = env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(recognizer);
                    host.initial_location = Some(location);
                    host.previous_location = location;
                    host.previous_timestamp = timestamp;
                    host.translation = CGPoint { x: 0.0, y: 0.0 };
                    host.velocity = CGPoint { x: 0.0, y: 0.0 };
                    host.superclass.state = UIGestureRecognizerStatePossible;
                }
                UITouchPhaseMoved | UITouchPhaseEnded => {
                    let should_send = {
                        let host = env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(recognizer);
                        let Some(initial) = host.initial_location else { continue };
                        host.translation = CGPoint {
                            x: location.x - initial.x,
                            y: location.y - initial.y,
                        };
                        let elapsed = timestamp - host.previous_timestamp;
                        host.velocity = if elapsed > 0.0 {
                            CGPoint {
                                x: (location.x - host.previous_location.x) / elapsed as CGFloat,
                                y: (location.y - host.previous_location.y) / elapsed as CGFloat,
                            }
                        } else {
                            CGPoint { x: 0.0, y: 0.0 }
                        };
                        host.previous_location = location;
                        host.previous_timestamp = timestamp;
                        if delivered_phase == UITouchPhaseEnded {
                            let active = matches!(host.superclass.state,
                                UIGestureRecognizerStateBegan | UIGestureRecognizerStateChanged);
                            host.superclass.state = if active {
                                UIGestureRecognizerStateEnded
                            } else {
                                UIGestureRecognizerStateFailed
                            };
                            active
                        } else {
                            let distance_squared = host.translation.x * host.translation.x
                                + host.translation.y * host.translation.y;
                            if host.superclass.state == UIGestureRecognizerStatePossible
                                && distance_squared >= 100.0
                            {
                                host.superclass.state = UIGestureRecognizerStateBegan;
                                true
                            } else if matches!(host.superclass.state,
                                UIGestureRecognizerStateBegan | UIGestureRecognizerStateChanged)
                            {
                                host.superclass.state = UIGestureRecognizerStateChanged;
                                true
                            } else { false }
                        }
                    };
                    if should_send { send_actions(env, recognizer); }
                    if delivered_phase == UITouchPhaseEnded {
                        let host = env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(recognizer);
                        host.initial_location = None;
                        host.velocity = CGPoint { x: 0.0, y: 0.0 };
                        host.superclass.state = UIGestureRecognizerStatePossible;
                    }
                }
                _ => {}
            }
            continue;
        }

        if !env.objc.class_is_subclass_of(class, pinch_class) { continue; }

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

pub(super) fn process_touches(
    env: &mut Environment,
    hit_view: id,
    event: id,
    delivered_phase: UITouchPhase,
) {
    let mut recognizer_view = hit_view;
    while recognizer_view != nil {
        process_recognizers_for_view(env, recognizer_view, event, delivered_phase);
        recognizer_view = msg![env; recognizer_view superview];
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

@implementation UITapGestureRecognizer: UIGestureRecognizer

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UITapGestureRecognizerHostObject>::default(), &mut env.mem)
}
- (NSUInteger)numberOfTapsRequired {
    env.objc.borrow::<UITapGestureRecognizerHostObject>(this).number_of_taps_required
}
- (())setNumberOfTapsRequired:(NSUInteger)value {
    env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(this).number_of_taps_required = value;
}
- (NSUInteger)numberOfTouchesRequired {
    env.objc.borrow::<UITapGestureRecognizerHostObject>(this).number_of_touches_required
}
- (())setNumberOfTouchesRequired:(NSUInteger)value {
    env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(this).number_of_touches_required = value;
}
- (())dealloc {
    let _ = std::mem::take(env.objc.borrow_mut::<UITapGestureRecognizerHostObject>(this));
    msg_super![env; this dealloc]
}
@end

@implementation UIPanGestureRecognizer: UIGestureRecognizer

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UIPanGestureRecognizerHostObject>::default(), &mut env.mem)
}
- (NSUInteger)minimumNumberOfTouches {
    env.objc.borrow::<UIPanGestureRecognizerHostObject>(this).minimum_number_of_touches
}
- (())setMinimumNumberOfTouches:(NSUInteger)value {
    env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(this).minimum_number_of_touches = value;
}
- (NSUInteger)maximumNumberOfTouches {
    env.objc.borrow::<UIPanGestureRecognizerHostObject>(this).maximum_number_of_touches
}
- (())setMaximumNumberOfTouches:(NSUInteger)value {
    env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(this).maximum_number_of_touches = value;
}
- (CGPoint)translationInView:(id)view {
    let translation = env.objc.borrow::<UIPanGestureRecognizerHostObject>(this).translation;
    if view == nil { translation } else {
        let zero = CGPoint { x: 0.0, y: 0.0 };
        let start: CGPoint = msg![env; view convertPoint:zero fromView:nil];
        let end: CGPoint = msg![env; view convertPoint:translation fromView:nil];
        CGPoint { x: end.x - start.x, y: end.y - start.y }
    }
}
- (())setTranslation:(CGPoint)translation inView:(id)view {
    let translation = if view == nil { translation } else {
        let zero = CGPoint { x: 0.0, y: 0.0 };
        let start: CGPoint = msg![env; view convertPoint:zero toView:nil];
        let end: CGPoint = msg![env; view convertPoint:translation toView:nil];
        CGPoint { x: end.x - start.x, y: end.y - start.y }
    };
    env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(this).translation = translation;
}
- (CGPoint)velocityInView:(id)_view {
    env.objc.borrow::<UIPanGestureRecognizerHostObject>(this).velocity
}
- (())dealloc {
    let _ = std::mem::take(env.objc.borrow_mut::<UIPanGestureRecognizerHostObject>(this));
    msg_super![env; this dealloc]
}
@end

};
