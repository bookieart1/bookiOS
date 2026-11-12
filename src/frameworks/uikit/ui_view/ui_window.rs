/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIWindow`.
//!
//! Useful resources:
//! - [Technical Q&A QA1588: Automatic orientation support for iPhone and iPad apps](https://developer.apple.com/library/archive/qa/qa1588/_index.html)
//! - [Technical Q&A QA1688: Why won't my UIViewController rotate with the device?](https://developer.apple.com/library/archive/qa/qa1688/_index.html)

use super::UIViewHostObject;
use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::core_graphics::cg_affine_transform::CGAffineTransform;
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect};
use crate::frameworks::foundation::ns_string;
use crate::frameworks::uikit::ui_application::{
    UIInterfaceOrientation,
    UIInterfaceOrientationLandscapeLeft, UIInterfaceOrientationLandscapeRight,
    UIInterfaceOrientationPortrait, UIInterfaceOrientationPortraitUpsideDown,
};
use crate::objc::{
    id, msg, msg_class, msg_super, nil, objc_classes, release, retain, ClassExports,
};

#[derive(Default)]
pub struct State {
    /// List of visible windows for internal purposes. Non-retaining!
    ///
    /// This is public because Core Animation also uses it.
    pub windows: Vec<id>,
    /// The most recent window which received `makeKeyAndVisible` message.
    /// Non-retaining!
    pub key_window: Option<id>,
    /// Window levels per window (UIWindowLevel).
    /// Non-retaining!
    pub window_levels: Vec<(id, CGFloat)>,
    /// Strong root-view-controller property for each window.
    pub root_view_controllers: Vec<(id, id)>,
    /// Bounded launch-time evidence for games which create a window but never
    /// produce a visible frame. This is diagnostic only.
    diagnostic_transition_count: u8,
}

fn choose_initial_controller_orientation(
    current: UIInterfaceOrientation,
    supports: [bool; 4],
) -> Option<UIInterfaceOrientation> {
    let supports_orientation = |orientation: UIInterfaceOrientation| {
        let index = orientation.checked_sub(1).and_then(|value| usize::try_from(value).ok());
        index.and_then(|index| supports.get(index)).copied().unwrap_or(false)
    };
    if supports_orientation(current) {
        return Some(current);
    }
    [
        UIInterfaceOrientationLandscapeLeft,
        UIInterfaceOrientationLandscapeRight,
        UIInterfaceOrientationPortrait,
        UIInterfaceOrientationPortraitUpsideDown,
    ]
    .into_iter()
    .find(|&orientation| supports_orientation(orientation))
}

fn log_startup_window_transition(
    env: &mut crate::Environment,
    event: &'static str,
    window: id,
    related: id,
) {
    const LIMIT: u8 = 24;
    let state = &mut env.framework_state.uikit.ui_view.ui_window;
    if state.diagnostic_transition_count >= LIMIT {
        return;
    }
    state.diagnostic_transition_count += 1;
    log!(
        "bounded UIWindow startup diagnostic #{}/{}: event={} window={:?} related={:?} key-window={:?} known-windows={}",
        state.diagnostic_transition_count,
        LIMIT,
        event,
        window,
        related,
        state.key_window,
        state.windows.len(),
    );
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIWindow: UIView

// TODO: more?

- (id)initWithFrame:(CGRect)frame {
    let this = msg_super![env; this initWithFrame:frame];
    // Undocumented: windows seem to be hidden by default on iOS, unlike views.
    // Super call to bypass the overriden setter on this class, which would post
    // a notification.
    () = msg_super![env; this setHidden:true];

    let state = &mut env.framework_state.uikit.ui_view.ui_window;
    state.windows.push(this);
    // Default to normal window level (0.0 on iOS)
    state.window_levels.push((this, 0.0));
    state.root_view_controllers.push((this, nil));

    log_dbg!(
        "New window: {:?}. New list of all windows: {:?}",
        this,
        state.windows,
    );
    log_startup_window_transition(env, "initWithFrame", this, nil);

    this
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    // Unlike windows made in code, a nib's window is only hidden if the nib
    // says so (UIHidden), which Interface Builder does unless it's "Visible
    // at Launch". Some games never make their window visible themselves.
    let this = msg_super![env; this initWithCoder:coder];

    let state = &mut env.framework_state.uikit.ui_view.ui_window;
    state.windows.push(this);
    state.window_levels.push((this, 0.0));
    state.root_view_controllers.push((this, nil));

    log_dbg!(
        "New window: {:?}. New list of all windows: {:?}",
        this,
        state.windows,
    );
    log_startup_window_transition(env, "initWithCoder", this, coder);

    this
}

- (())dealloc {
    let root = {
        let state = &mut env.framework_state.uikit.ui_view.ui_window;

        if let Some(key_window) = state.key_window {
            if key_window == this {
                state.key_window = None;
            }
        }

        let list = &mut state.windows;
        let idx = list.iter().position(|&w| w == this).unwrap();
        list.remove(idx);

        // Remove window level entry for this window
        state.window_levels.retain(|(w, _)| *w != this);

        let root = state
            .root_view_controllers
            .iter()
            .find(|(window, _)| *window == this)
            .map_or(nil, |(_, root)| *root);
        state
            .root_view_controllers
            .retain(|(window, _)| *window != this);

        log_dbg!(
            "Deallocating window {:?}. New list of all windows: {:?}",
            this,
            state.windows,
        );
        root
    };
    release(env, root);
    msg_super![env; this dealloc]
}

- (())setHidden:(bool)is_hidden {
    () = msg_super![env; this setHidden:is_hidden];

    // TODO: post UIWindowDidBecomeVisibleNotification,
    //            UIWindowDidBecomeHiddenNotification
    log_dbg!("[(UIWindow*){:?} setHidden:{:?}]", this, is_hidden);
    log_startup_window_transition(
        env,
        if is_hidden { "setHidden:YES" } else { "setHidden:NO" },
        this,
        nil,
    );
}

- (())makeKeyWindow {
    if let Some(previous) = env.framework_state.uikit.ui_view.ui_window.key_window {
        if previous != this {
            () = msg![env; previous resignKeyWindow];
        }
    }
    env.framework_state.uikit.ui_view.ui_window.key_window = Some(this);
    log_startup_window_transition(env, "makeKeyWindow", this, nil);

    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let notif_name = ns_string::get_static_str(env, UIWindowDidBecomeKeyNotification);
    () = msg![env; center postNotificationName:notif_name object:this userInfo:nil];
}

- (())resignKeyWindow {
    if env.framework_state.uikit.ui_view.ui_window.key_window != Some(this) {
        return;
    }
    env.framework_state.uikit.ui_view.ui_window.key_window = None;
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let notif_name = ns_string::get_static_str(env, UIWindowDidResignKeyNotification);
    () = msg![env; center postNotificationName:notif_name object:this userInfo:nil];
}

- (bool)isKeyWindow {
    env.framework_state.uikit.ui_view.ui_window.key_window == Some(this)
}

- (())makeKeyAndVisible {
    // TODO: We don't currently have send any non-touch events to windows,
    // so there's no meaning in it yet.

    // FIXME: This should also bump the window to the top of the list.

    () = msg![env; this makeKeyWindow];

    // TODO: post UIWindowDidBecomeVisibleNotification
    () = msg![env; this setHidden:false];
}

- (id)rootViewController {
    env.framework_state
        .uikit
        .ui_view
        .ui_window
        .root_view_controllers
        .iter()
        .find(|(window, _)| *window == this)
        .map_or(nil, |(_, controller)| *controller)
}

- (())setRootViewController:(id)controller {
    let old = msg![env; this rootViewController];
    if old == controller {
        return;
    }

    retain(env, controller);
    let entry = env
        .framework_state
        .uikit
        .ui_view
        .ui_window
        .root_view_controllers
        .iter_mut()
        .find(|(window, _)| *window == this)
        .unwrap();
    entry.1 = controller;

    if old != nil {
        let old_view: id = msg![env; old view];
        () = msg![env; old_view removeFromSuperview];
    }
    if controller != nil {
        let view: id = msg![env; controller view];
        let bounds: CGRect = msg![env; this bounds];
        () = msg![env; view setFrame:bounds];
        () = msg![env; this addSubview:view];
    }
    release(env, old);
    log_startup_window_transition(env, "setRootViewController", this, controller);
}

// We only model the single main screen
- (id)screen {
    msg_class![env; UIScreen mainScreen]
}
- (())setScreen:(id)screen {
    log_dbg!("[(UIWindow*){:?} setScreen:{:?}]", this, screen);
}

// UIResponder implementation
// From the Apple UIView docs regarding [UIResponder nextResponder]:
// "UIWindow returns the application object."
- (id)nextResponder {
    msg_class![env; UIApplication sharedApplication]
}

- (())addSubview:(id)view {
    log_dbg!("[(UIWindow*){:?} addSubview:{:?}] => ()", this, view);

    if view == nil {
        () = msg_super![env; this addSubview:view];
        return;
    }

    let view_controller = env.objc.borrow::<UIViewHostObject>(view).view_controller;
    if env.bundle.bundle_identifier() == "com.majesticsoftware.blot" {
        let window_frame: CGRect = msg![env; this frame];
        let view_frame: CGRect = msg![env; view frame];
        let view_bounds: CGRect = msg![env; view bounds];
        let view_transform: CGAffineTransform = msg![env; view transform];
        log!(
            "Blot display diagnostic: UIWindow addSubview view={view:?}, window_frame={window_frame:?}, view_frame={view_frame:?}, view_bounds={view_bounds:?}, view_transform={view_transform:?}, view_controller={view_controller:?}"
        );
    }

    if view_controller == nil {
        () = msg_super![env; this addSubview:view];
        return;
    }

    // Below we treat a special case of adding view controller's view
    // to a window, in order to generate display related notifications

    if env.objc.borrow::<UIViewHostObject>(this).subviews.contains(&view) {
        // For the case of existing view hidden by another view,
        // we need to delay a below sequence up until obstructions are removed
        log!("TODO: case of existing view hidden by another view for sending view[Will,Did]Appear");
    }

    let vc = view_controller;
    () = msg![env; vc viewWillAppear:false];
    () = msg_super![env; this addSubview:view];
    () = msg![env; vc viewDidAppear:false];

    // Support initial auto-rotation. UIKit starts launch layout in portrait,
    // then, on iOS 5 and below, asks the visible controller which interface
    // orientations it accepts before showing the window. The Info.plist
    // orientation remains a launch hint, not the controller's final answer.
    // FIXME: It's unclear when and where this auto-rotation is supposed to
    //        happen. It must have something to do with mounting the view
    //        controller to a window, so we do it here. QA1688 (see top of file)
    //        mentions a breaking behaviour change in iOS 6 that makes
    //        auto-rotation rely on rootViewController (a property only found in
    //        iOS 6), so the current implementation is specific to iOS <= 5.
    // FIXME: Are we supposed to notify the view somehow of the rotation?
    // FIXME: The device orientation stored on env.window can come from one of
    //        three places (user/default options, setStatusBarOrientation: etc,
    //        Info.plist UIInterfaceOrientation etc). It's not clear if these
    //        are really equivalent and should all trigger autorotation.
    let current_orientation = match env.window.as_ref().unwrap().current_rotation() {
        crate::window::DeviceOrientation::Portrait => UIInterfaceOrientationPortrait,
        crate::window::DeviceOrientation::PortraitUpsideDown => UIInterfaceOrientationPortraitUpsideDown,
        crate::window::DeviceOrientation::LandscapeLeft => UIInterfaceOrientationLandscapeLeft,
        crate::window::DeviceOrientation::LandscapeRight => UIInterfaceOrientationLandscapeRight,
    };
    let supports = [
        msg![env; vc shouldAutorotateToInterfaceOrientation:UIInterfaceOrientationPortrait],
        msg![env; vc shouldAutorotateToInterfaceOrientation:UIInterfaceOrientationPortraitUpsideDown],
        msg![env; vc shouldAutorotateToInterfaceOrientation:UIInterfaceOrientationLandscapeRight],
        msg![env; vc shouldAutorotateToInterfaceOrientation:UIInterfaceOrientationLandscapeLeft],
    ];
    if let Some(orientation) =
        choose_initial_controller_orientation(current_orientation, supports)
    {
        if orientation != current_orientation {
            let application: id = msg_class![env; UIApplication sharedApplication];
            () = msg![env; application setStatusBarOrientation:orientation];
        }
        if orientation != UIInterfaceOrientationPortrait {
            log_dbg!("App requested autorotation; applying orientation transform to view {:?}.", view);
            let transform = match orientation {
                UIInterfaceOrientationPortraitUpsideDown => CGAffineTransform::make_rotation(-std::f32::consts::PI),
                UIInterfaceOrientationLandscapeLeft => CGAffineTransform::make_rotation(-std::f32::consts::FRAC_PI_2),
                UIInterfaceOrientationLandscapeRight => CGAffineTransform::make_rotation(std::f32::consts::FRAC_PI_2),
                _ => unimplemented!(),
            };

            let window_frame: CGRect = msg![env; this frame];
            log_dbg!("Window frame: {window_frame:?}");
            let view_frame: CGRect = msg![env; view frame];
            log_dbg!("Old view frame: {view_frame:?}");

            () = msg![env; view setTransform:transform];

            // Re-apply the view's old frame to compensate for the rotation
            // effectively offseting its center position and changing the size.
            // FIXME: I have no idea if this is how this should be solved, but
            //        it works for DMC4 Refrain at least.

            let view_frame: CGRect = msg![env; view frame];
            log_dbg!("Old view frame after transform: {view_frame:?}");

            () = msg![env; view setFrame:window_frame];

            let view_frame: CGRect = msg![env; view frame];
            log_dbg!("New view frame after re-applying old view frame: {view_frame:?}");
        }
    }
}

- (CGPoint)convertPoint:(CGPoint)point
             fromWindow:(id)other { // UIWindow*
    let this_layer: id = msg![env; this layer];
    // Resolves to nil if other is nil.
    let other_layer: id = msg![env; other layer];
    msg![env; this_layer convertPoint:point fromLayer:other_layer]
}
- (CGPoint)convertPoint:(CGPoint)point
               toWindow:(id)other { // UIWindow*
    let this_layer: id = msg![env; this layer];
    // Resolves to nil if other is nil.
    let other_layer: id = msg![env; other layer];
    msg![env; this_layer convertPoint:point toLayer:other_layer]
}

- (CGFloat)windowLevel {
    let state = &env.framework_state.uikit.ui_view.ui_window;
    if let Some((_, level)) = state.window_levels.iter().rev().find(|(w, _)| *w == this) {
        *level
    } else {
        // Default: normal level
        0.0
    }
}

- (())setWindowLevel:(CGFloat)level {
    let state = &mut env.framework_state.uikit.ui_view.ui_window;

    if let Some((_, stored_level)) = state.window_levels.iter_mut().find(|(w, _)| *w == this) {
        *stored_level = level;
    } else {
        state.window_levels.push((this, level));
    }

    log_dbg!("[(UIWindow*){:?} setWindowLevel:{}]", this, level);

    // TODO: re-order windows based on windowLevel for proper z-ordering.
}

@end

};

/// Window life-cycle notifications
/// TODO: more notifications
const UIWindowDidBecomeKeyNotification: &str = "UIWindowDidBecomeKeyNotification";
const UIWindowDidResignKeyNotification: &str = "UIWindowDidResignKeyNotification";
/// Keyboard notifications
/// TODO: more keyboard notifications
pub const UIKeyboardWillShowNotification: &str = "UIKeyboardWillShowNotification";
pub const UIKeyboardDidShowNotification: &str = "UIKeyboardDidShowNotification";
pub const UIKeyboardWillHideNotification: &str = "UIKeyboardWillHideNotification";
pub const UIKeyboardDidHideNotification: &str = "UIKeyboardDidHideNotification";
pub const UIKeyboardBoundsUserInfoKey: &str = "UIKeyboardBoundsUserInfoKey";
pub const UIKeyboardAnimationDurationUserInfoKey: &str =
    "UIKeyboardAnimationDurationUserInfoKey";
pub const UIKeyboardAnimationCurveUserInfoKey: &str = "UIKeyboardAnimationCurveUserInfoKey";

pub const CONSTANTS: ConstantExports = &[
    (
        "_UIWindowDidBecomeKeyNotification",
        HostConstant::NSString(UIWindowDidBecomeKeyNotification),
    ),
    (
        "_UIWindowDidResignKeyNotification",
        HostConstant::NSString(UIWindowDidResignKeyNotification),
    ),
    (
        "_UIKeyboardWillShowNotification",
        HostConstant::NSString(UIKeyboardWillShowNotification),
    ),
    (
        "_UIKeyboardDidShowNotification",
        HostConstant::NSString(UIKeyboardDidShowNotification),
    ),
    (
        "_UIKeyboardWillHideNotification",
        HostConstant::NSString(UIKeyboardWillHideNotification),
    ),
    (
        "_UIKeyboardDidHideNotification",
        HostConstant::NSString(UIKeyboardDidHideNotification),
    ),
    (
        "_UIKeyboardBoundsUserInfoKey",
        HostConstant::NSString(UIKeyboardBoundsUserInfoKey),
    ),
    (
        "_UIKeyboardAnimationDurationUserInfoKey",
        HostConstant::NSString(UIKeyboardAnimationDurationUserInfoKey),
    ),
    (
        "_UIKeyboardAnimationCurveUserInfoKey",
        HostConstant::NSString(UIKeyboardAnimationCurveUserInfoKey),
    ),
    (
        "_UIWindowLevelNormal",
        HostConstant::Custom(|env| env.mem.alloc_and_write(0.0f32).cast().cast_const()),
    ),
    (
        "_UIWindowLevelStatusBar",
        HostConstant::Custom(|env| env.mem.alloc_and_write(1000.0f32).cast().cast_const()),
    ),
    (
        "_UIWindowLevelAlert",
        HostConstant::Custom(|env| env.mem.alloc_and_write(2000.0f32).cast().cast_const()),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_255_landscape_only_controller_selects_landscape_at_launch() {
        assert_eq!(
            choose_initial_controller_orientation(
                UIInterfaceOrientationPortrait,
                [false, false, true, true],
            ),
            Some(UIInterfaceOrientationLandscapeLeft)
        );
    }
}
