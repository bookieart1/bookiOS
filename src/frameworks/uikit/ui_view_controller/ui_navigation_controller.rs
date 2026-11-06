/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UINavigationController`.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::{ns_array, NSUInteger};
use crate::objc::{
    autorelease, id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil,
    objc_classes, release, retain, ClassExports, NSZonePtr, SEL,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn controller_observes_container_and_stable_navigation_bar_state() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let root: id = msg_class![env; UIViewController new];
        let nav: id = msg_class![env; UINavigationController alloc];
        super::super::set_navigation_controller(env, root, nav);
        let observed: id = msg![env; root navigationController];
        assert_eq!(observed, nav);

        // Avoid forcing UIView screen/bundle loading in this synthetic host
        // fixture. Supplying an allocated bar directly still exercises the
        // controller's stable ownership and the bar's item-stack contract.
        let bar: id = msg_class![env; UINavigationBar alloc];
        env.objc
            .borrow_mut::<UINavigationControllerHostObject>(nav)
            .navigation_bar = bar;
        let first_bar: id = msg![env; nav navigationBar];
        let second_bar: id = msg![env; nav navigationBar];
        assert_eq!(first_bar, second_bar);
        let root_item: id = msg![env; root navigationItem];
        () = msg![env; first_bar pushNavigationItem:root_item animated:false];
        let top_item: id = msg![env; first_bar topItem];
        assert_eq!(top_item, root_item);

        let child: id = msg_class![env; UIViewController new];
        super::super::set_navigation_controller(env, child, nav);
        let observed: id = msg![env; child navigationController];
        assert_eq!(observed, nav);
        super::super::set_navigation_controller(env, child, nil);
        let observed: id = msg![env; child navigationController];
        assert_eq!(observed, nil);

        super::super::set_navigation_controller(env, root, nil);
        release(env, child);
        release(env, root);
        // `bar` intentionally skipped UIView initialization above, so let the
        // synthetic environment discard it with `nav` instead of invoking
        // UIView's registered-view teardown contract on an unregistered view.
    }
}

// TODO: navigation bar and toolbar
// TODO: animations

#[derive(Default)]
struct UINavigationControllerHostObject {
    superclass: super::UIViewControllerHostObject,
    /// something implementing UINavigationControllerDelegate
    delegate: id,
    /// Navigation stack of view controllers, non-retaining
    /// (we explicitly retain/release on push/pop messages)
    navigation_stack: Vec<id>,
    /// Stable UINavigationBar owned by the controller.
    navigation_bar: id,
    /// Whether the controller's navigation bar is hidden. The navigation bar
    /// itself is not rendered yet, but callers can still observe and update the
    /// public property consistently.
    navigation_bar_hidden: bool,
}
impl_HostObject_with_superclass!(UINavigationControllerHostObject);

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UINavigationController: UIViewController

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UINavigationControllerHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCoder:(id)coder {
    let this: id = msg_super![env; this initWithCoder:coder];

    // Older UIKit nib archives store the navigation bar and controller stack
    // beside the UIViewController's optional UIView. Reconstruct that public
    // state instead of treating a missing UIView as an empty controller.
    let navigation_bar_key = get_static_str(env, "UINavigationBar");
    let navigation_bar: id = msg![env; coder decodeObjectForKey:navigation_bar_key];
    if navigation_bar != nil {
        retain(env, navigation_bar);
        let old = env
            .objc
            .borrow::<UINavigationControllerHostObject>(this)
            .navigation_bar;
        release(env, old);
        env.objc
            .borrow_mut::<UINavigationControllerHostObject>(this)
            .navigation_bar = navigation_bar;
    }

    let view_controllers_key = get_static_str(env, "UIViewControllers");
    let child_view_controllers_key = get_static_str(env, "UIChildViewControllers");
    let mut controllers: id = msg![env; coder decodeObjectForKey:view_controllers_key];
    if controllers == nil {
        controllers = msg![env; coder decodeObjectForKey:child_view_controllers_key];
    }
    if controllers != nil {
        let count: NSUInteger = msg![env; controllers count];
        if count > 0 {
            () = msg![env; this setViewControllers:controllers animated:false];
        }
    }

    this
}

- (id)initWithRootViewController:(id)root_vc { // UIViewController *
    () = msg![env; this pushViewController:root_vc animated:false];
    this
}

- (())dealloc {
    let (stack, navigation_bar) = {
        let host = env.objc.borrow_mut::<UINavigationControllerHostObject>(this);
        (std::mem::take(&mut host.navigation_stack), host.navigation_bar)
    };
    for controller in stack {
        super::set_navigation_controller(env, controller, nil);
        release(env, controller);
    }
    release(env, navigation_bar);
    msg_super![env; this dealloc]
}

// weak/non-retaining
- (())setDelegate:(id)delegate { // something implementing UINavigationControllerDelegate
    log_dbg!("[(UINavigationController*){:?} setDelegate:{:?}]", this, delegate);
    let host_object = env.objc.borrow_mut::<UINavigationControllerHostObject>(this);
    host_object.delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UINavigationControllerHostObject>(this).delegate
}

- (())pushViewController:(id)view_controller // UIViewController *
                animated:(bool)_animated {
    let stack = &mut env.objc.borrow_mut::<UINavigationControllerHostObject>(this).navigation_stack;
    assert!(!stack.contains(&view_controller));
    stack.push(view_controller);
    retain(env, view_controller);
    super::set_navigation_controller(env, view_controller, this);

    let navigation_bar: id = msg![env; this navigationBar];
    let navigation_item: id = msg![env; view_controller navigationItem];
    () = msg![env; navigation_bar pushNavigationItem:navigation_item animated:false];

    // UIKit has loaded the destination view (and therefore delivered its
    // one-time viewDidLoad) before telling the navigation delegate that the
    // controller is about to be shown. Older applications use that ordering
    // to prepare appearance state from outlets populated in viewDidLoad.
    let vc_view: id = msg![env; view_controller view];

    let delegate = env.objc.borrow::<UINavigationControllerHostObject>(this).delegate;
    let sel: SEL = env
        .objc
        .register_host_selector(
            "navigationController:willShowViewController:animated:".to_string(),
            &mut env.mem
        );
    let responds: bool = msg![env; delegate respondsToSelector:sel];
    if responds {
        () = msg![env; delegate navigationController:this willShowViewController:view_controller animated:false];
    }
    let self_view: id = msg![env; this view];
    // TODO: animations
    () = msg![env; view_controller viewWillAppear:false];
    () = msg![env; self_view addSubview:vc_view];
    () = msg![env; view_controller viewDidAppear:false];
    let sel: SEL = env
        .objc
        .register_host_selector(
            "navigationController:didShowViewController:animated:".to_string(),
            &mut env.mem
        );
    let responds: bool  = msg![env; delegate respondsToSelector:sel];
    if responds {
        () = msg![env; delegate navigationController:this didShowViewController:view_controller animated:false];
    }
}

- (id)topViewController {
    if let Some(top_vc) = env.objc.borrow::<UINavigationControllerHostObject>(this).navigation_stack.last() {
        *top_vc
    } else {
        nil
    }
}
- (id)visibleViewController {
    msg![env; this topViewController]
}

- (id)popViewControllerAnimated:(bool)_animated {
    let controller = {
        let stack = &mut env
            .objc
            .borrow_mut::<UINavigationControllerHostObject>(this)
            .navigation_stack;
        if stack.len() <= 1 {
            return nil;
        }
        stack.pop().unwrap()
    };
    let view = env
        .objc
        .borrow::<super::UIViewControllerHostObject>(controller)
        .view;
    if view != nil {
        () = msg![env; controller viewWillDisappear:false];
        () = msg![env; view removeFromSuperview];
        () = msg![env; controller viewDidDisappear:false];
    }
    super::set_navigation_controller(env, controller, nil);
    let navigation_bar: id = msg![env; this navigationBar];
    let _: id = msg![env; navigation_bar popNavigationItemAnimated:false];
    autorelease(env, controller)
}

- (id)viewControllers {
    let vcs = env.objc.borrow::<UINavigationControllerHostObject>(this).navigation_stack.to_vec();
    for vc in &vcs {
        retain(env, *vc);
    }
    let res = ns_array::from_vec(env, vcs);
    autorelease(env, res)
}
- (())setViewControllers:(id)controllers { // NSArray *
    msg![env; this setViewControllers:controllers animated:false]
}

- (())setViewControllers:(id)controllers // NSArray *
                animated:(bool)animated {
    assert!(!animated);

    // Clean existing view controllers
    let self_view = env.objc.borrow::<UINavigationControllerHostObject>(this).superclass.view;
    let mut stack = std::mem::take(&mut env.objc.borrow_mut::<UINavigationControllerHostObject>(this).navigation_stack);
    // TODO: shall we drain in reverse order? does it matter?
    for controller in stack.drain(..) {
        let vc_view = env.objc.borrow::<super::UIViewControllerHostObject>(controller).view;
        let vc_view_superview = msg![env; vc_view superview];
        assert_eq!(self_view, vc_view_superview);
        // TODO: view{Will,Did}Disappear: messages for vc?
        () = msg![env; vc_view removeFromSuperview];

        super::set_navigation_controller(env, controller, nil);
        release(env, controller);
    }
    let navigation_bar: id = msg![env; this navigationBar];
    () = msg![env; navigation_bar setItems:nil animated:false];

    let mut tmp_stack: Vec<id> = Vec::new();
    let count: NSUInteger = msg![env; controllers count];
    // TODO: zero count
    assert!(count > 0);
    for i in 0..(count - 1) {
        let next: id = msg![env; controllers objectAtIndex:i];
        tmp_stack.push(next);
        retain(env, next);
        super::set_navigation_controller(env, next, this);
        let item: id = msg![env; next navigationItem];
        () = msg![env; navigation_bar pushNavigationItem:item animated:false];
    }
    env.objc.borrow_mut::<UINavigationControllerHostObject>(this).navigation_stack = tmp_stack;

    // The n-1 element in the controllers array is special and need to be pushed
    // TODO: double check this behavior
    let last_vc: id = msg![env; controllers objectAtIndex:(count - 1)];
    () = msg![env; this pushViewController:last_vc animated:animated];
}

- (id)navigationBar {
    let existing = env
        .objc
        .borrow::<UINavigationControllerHostObject>(this)
        .navigation_bar;
    if existing != nil {
        return existing;
    }
    let bar: id = msg_class![env; UINavigationBar alloc];
    let frame = CGRect::default();
    let bar: id = msg![env; bar initWithFrame:frame];
    env.objc
        .borrow_mut::<UINavigationControllerHostObject>(this)
        .navigation_bar = bar;
    bar
}
- (bool)isNavigationBarHidden {
    env.objc.borrow::<UINavigationControllerHostObject>(this).navigation_bar_hidden
}
- (())setNavigationBarHidden:(bool)hidden {
    env.objc.borrow_mut::<UINavigationControllerHostObject>(this).navigation_bar_hidden = hidden;
}
- (())setNavigationBarHidden:(bool)hidden
                     animated:(bool)_animated {
    // Bar animations are not implemented. The final visibility state and
    // observable property value are applied synchronously.
    () = msg![env; this setNavigationBarHidden:hidden];
}

@end

};
