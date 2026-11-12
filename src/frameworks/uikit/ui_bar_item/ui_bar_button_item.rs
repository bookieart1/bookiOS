/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIBarButtonItem`.

use super::UIBarItemHostObject;
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_string::{get_static_str, to_rust_string};
use crate::frameworks::foundation::NSInteger;
use crate::frameworks::uikit::ui_font::UITextAlignmentCenter;
use crate::frameworks::uikit::ui_view::ui_control::UIControlEventTouchUpInside;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr, SEL,
};

pub type UIBarButtonItemStyle = NSInteger;
pub const UIBarButtonItemStylePlain: UIBarButtonItemStyle = 0;
pub const UIBarButtonItemStyleBordered: UIBarButtonItemStyle = 1;
pub const UIBarButtonItemStyleDone: UIBarButtonItemStyle = 2;

pub type UIBarButtonSystemItem = NSInteger;
pub const UIBarButtonSystemItemDone: UIBarButtonSystemItem = 0;
pub const UIBarButtonSystemItemCancel: UIBarButtonSystemItem = 1;
pub const UIBarButtonSystemItemFlexibleSpace: UIBarButtonSystemItem = 5;
pub const UIBarButtonSystemItemFixedSpace: UIBarButtonSystemItem = 6;

pub struct UIBarButtonItemHostObject {
    superclass: UIBarItemHostObject,
    pub style: UIBarButtonItemStyle,
    pub target: id,
    pub action: Option<SEL>,
    pub system_item: Option<UIBarButtonSystemItem>,
    /// Internal UIControl used for rendering and touch handling in UIToolbar.
    pub custom_view: id,
    /// Internal UILabel within custom_view, exposed for sizing queries.
    pub label: id,
    pub width: CGFloat,
}

impl_HostObject_with_superclass!(UIBarButtonItemHostObject);

impl Default for UIBarButtonItemHostObject {
    fn default() -> Self {
        Self {
            superclass: Default::default(),
            style: UIBarButtonItemStylePlain,
            target: nil,
            action: None,
            system_item: None,
            custom_view: nil,
            label: nil,
            width: 0.0,
        }
    }
}

fn make_label(env: &mut crate::Environment, title: id) -> id {
    // As on iPhone OS 3: white bold text, drawn over the button background
    // that UIToolbar draws.
    let font: id = msg_class![env; UIFont boldSystemFontOfSize:12_f32];
    let title_color: id = msg_class![env; UIColor whiteColor];
    let item_bg_color: id = msg_class![env; UIColor clearColor];
    let init_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 0.0,
            height: 44.0,
        },
    };
    let label: id = msg_class![env; UILabel new];
    let label: id = msg![env; label initWithFrame:init_frame];
    () = msg![env; label setTextAlignment:UITextAlignmentCenter];
    () = msg![env; label setText:title];
    () = msg![env; label setTextColor:title_color];
    () = msg![env; label setFont:font];
    () = msg![env; label setBackgroundColor:item_bg_color];
    label
}

fn make_custom_view(
    env: &mut crate::Environment,
    label: id,
    target: id,
    action: Option<SEL>,
) -> id {
    let init_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 0.0,
            height: 44.0,
        },
    };
    let control: id = msg_class![env; UIControl alloc];
    let control: id = msg![env; control initWithFrame:init_frame];
    if label != nil {
        () = msg![env; control addSubview:label];
    }
    if target != nil {
        if let Some(action) = action {
            () = msg![env; control addTarget:target action:action forControlEvents:UIControlEventTouchUpInside];
        }
    }
    control
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIBarButtonItem: UIBarItem

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UIBarButtonItemHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCoder:(id)coder {
    let this: id = msg_super![env; this init];
    let title_key = get_static_str(env, "UITitle");
    let style_key = get_static_str(env, "UIStyle");
    let custom_view_key = get_static_str(env, "UICustomView");
    let width_key = get_static_str(env, "UIWidth");

    let title: id = if msg![env; coder containsValueForKey:title_key] {
        msg![env; coder decodeObjectForKey:title_key]
    } else {
        nil
    };
    let style: UIBarButtonItemStyle = if msg![env; coder containsValueForKey:style_key] {
        msg![env; coder decodeIntegerForKey:style_key]
    } else {
        UIBarButtonItemStylePlain
    };
    let decoded_custom: id = if msg![env; coder containsValueForKey:custom_view_key] {
        msg![env; coder decodeObjectForKey:custom_view_key]
    } else {
        nil
    };
    let width: CGFloat = if msg![env; coder containsValueForKey:width_key] {
        msg![env; coder decodeFloatForKey:width_key]
    } else {
        0.0
    };

    let (label, custom_view) = if decoded_custom != nil {
        (nil, decoded_custom)
    } else if title != nil {
        let label = make_label(env, title);
        // A decoded item may have a title but no target/action connection.
        // UIKit still creates the item; it does not require a fabricated SEL.
        (label, make_custom_view(env, label, nil, None))
    } else {
        (nil, nil)
    };
    if title != nil {
        retain(env, title);
    }
    if custom_view != nil {
        retain(env, custom_view);
    }
    let host = env.objc.borrow_mut::<UIBarButtonItemHostObject>(this);
    host.superclass.title = title;
    host.style = style;
    host.custom_view = custom_view;
    host.label = label;
    host.width = width;
    this
}

- (id)initWithCustomView:(id)customView {
    log_dbg!(
        "[(UIBarButtonItem*){:?} initWithCustomView:{:?}]",
        this,
        customView
    );

    {
        let host = env.objc.borrow_mut::<UIBarButtonItemHostObject>(this);
        host.custom_view = customView;
    }

    if customView != nil {
        retain(env, customView);
    }

    msg_super![env; this init]
}

- (id)initWithTitle:(id)title
                style:(UIBarButtonItemStyle)style
                target:(id)target
                action:(SEL)action
{
    log_dbg!(
        "[(UIBarButtonItem*){:?} initWithTitle:{:?} style:{:?} target:{:?} action:{:?}]",
        this,
        to_rust_string(env, title),
        style,
        target,
        action
    );

    let label = make_label(env, title);
    let custom_view = make_custom_view(env, label, target, Some(action));

    {
        let host = env.objc.borrow_mut::<UIBarButtonItemHostObject>(this);
        host.superclass.title = title;
        host.style = style;
        host.target = target;
        host.action = Some(action);
        host.custom_view = custom_view;
        host.label = label;
    }

    if title != nil {
        retain(env, title);
    }
    if target != nil {
        retain(env, target);
    }
    retain(env, custom_view);

    msg_super![env; this init]
}

- (id)initWithBarButtonSystemItem:(UIBarButtonSystemItem)system_item
                            target:(id)target
                            action:(SEL)action
{
    log_dbg!(
        "[(UIBarButtonItem*){:?} initWithBarButtonSystemItem:{} target:{:?} action:{:?}]",
        this,
        system_item,
        target,
        action
    );

    let (title, label, style) = match system_item {
        UIBarButtonSystemItemDone => {
            let title = get_static_str(env, "Done");
            let label = make_label(env, title);
            (title, label, UIBarButtonItemStyleDone)
        }

        UIBarButtonSystemItemCancel => {
            let title = get_static_str(env, "Cancel");
            let label = make_label(env, title);
            (title, label, UIBarButtonItemStylePlain)
        }

        UIBarButtonSystemItemFlexibleSpace
        | UIBarButtonSystemItemFixedSpace => {
            (nil, nil, UIBarButtonItemStylePlain)
        }

        _ => unimplemented!(
            "UIBarButtonSystemItem {} rendering not implemented",
            system_item
        ),
    };

    let custom_view = make_custom_view(env, label, target, Some(action));

    {
        let host = env.objc.borrow_mut::<UIBarButtonItemHostObject>(this);
        host.superclass.title = title;
        host.style = style;
        host.system_item = Some(system_item);
        host.target = target;
        host.action = Some(action);
        host.custom_view = custom_view;
        host.label = label;
    }

    if title != nil {
        retain(env, title);
    }
    if target != nil {
        retain(env, target);
    }
    retain(env, custom_view);

    msg_super![env; this init]
}

- (id)customView {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).custom_view
}

- (id)label {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).label
}

- (id)title {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).superclass.title
}

- (())setTitle:(id)title {
    let label = env.objc.borrow::<UIBarButtonItemHostObject>(this).label;
    let old_title = env.objc.borrow::<UIBarButtonItemHostObject>(this).superclass.title;
    retain(env, title);
    release(env, old_title);
    env.objc.borrow_mut::<UIBarButtonItemHostObject>(this).superclass.title = title;
    if label != nil {
        () = msg![env; label setText:title];
    }
}

- (UIBarButtonItemStyle)style {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).style
}

- (())setStyle:(UIBarButtonItemStyle)style {
    match style {
        UIBarButtonItemStylePlain
        | UIBarButtonItemStyleBordered
        | UIBarButtonItemStyleDone => {}
        _ => unimplemented!("UIBarButtonItemStyle {}", style),
    }
    env.objc.borrow_mut::<UIBarButtonItemHostObject>(this).style = style;
}

- (id)target {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).target
}

- (())setTarget:(id)target {
    let old_target = env.objc.borrow::<UIBarButtonItemHostObject>(this).target;
    retain(env, target);
    env.objc.borrow_mut::<UIBarButtonItemHostObject>(this).target = target;
    release(env, old_target);
}

- (CGFloat)width {
    env.objc.borrow::<UIBarButtonItemHostObject>(this).width
}

- (())setWidth:(CGFloat)width {
    env.objc.borrow_mut::<UIBarButtonItemHostObject>(this).width = width;
}

- (CGSize)sizeThatFits:(CGSize)size {
    let host = env.objc.borrow::<UIBarButtonItemHostObject>(this);
    let label = host.label;
    let custom_view = host.custom_view;
    if label != nil {
        let label_size: CGSize = msg![env; label sizeThatFits:size];
        // 10pt either side of the title, as on iPhone OS 3.
        CGSize {
            width: label_size.width + 20.0,
            height: size.height,
        }
    } else if custom_view != nil {
        msg![env; custom_view sizeThatFits:size]
    } else {
        CGSize { width: 0.0, height: size.height }
    }
}

- (SEL)action {
    match env.objc.borrow::<UIBarButtonItemHostObject>(this).action {
        Some(sel) => sel,
        None => {
            log!("Warning: UIBarButtonItem has no action set!");
            env.objc.register_host_selector(
                "undefinedSelector".to_owned(),
                &mut env.mem,
            )
        }
    }
}

- (())setAction:(SEL)action {
    env.objc.borrow_mut::<UIBarButtonItemHostObject>(this).action = Some(action);
}

- (())addTarget:(id)target
         action:(SEL)action
forControlEvents:(crate::frameworks::foundation::NSUInteger)events {
    // Some iPhone OS nib archives reconnect a decoded UIBarButtonItem through
    // this UIControl-shaped selector. The item still has the ordinary single
    // target/action contract; its internal control receives the event mask.
    () = msg![env; this setTarget:target];
    () = msg![env; this setAction:action];
    let custom_view = env.objc.borrow::<UIBarButtonItemHostObject>(this).custom_view;
    if custom_view != nil {
        let control_class = env.objc.get_known_class("UIControl", &mut env.mem);
        let is_control: bool = msg![env; custom_view isKindOfClass:control_class];
        if is_control {
            () = msg![env; custom_view addTarget:target action:action forControlEvents:events];
        }
    }
}

- (())dealloc {
    let UIBarButtonItemHostObject {
        superclass,
        style: _,
        target,
        action: _,
        system_item: _,
        custom_view,
        label,
        width: _,
    } = std::mem::take(env.objc.borrow_mut(this));

    log_dbg!(
        "dealloc [(UIBarButtonItem*){:?} title {:?}, target {:?}, custom_view {:?}, label {:?}]",
        this, superclass.title, target, custom_view, label
    );

    release(env, superclass.title);
    release(env, target);
    release(env, custom_view);
    msg_super![env; this dealloc]
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_style_target_action_connection_round_trips() {
        let mut env = crate::Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let item: id = msg_class![env; UIBarButtonItem new];
        let target: id = msg_class![env; NSObject new];
        let action = env
            .objc
            .register_host_selector("performAction:".to_owned(), &mut env.mem);
        () = msg![env; item addTarget:target
                              action:action
                    forControlEvents:UIControlEventTouchUpInside];
        let stored_target: id = msg![env; item target];
        let stored_action: SEL = msg![env; item action];
        assert_eq!(stored_target, target);
        assert_eq!(stored_action, action);
        release(env, target);
        release(env, item);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn development_255_cancel_system_item_has_historical_title_and_plain_style() {
        let mut env = crate::Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let target: id = msg_class![env; NSObject new];
        let action = env
            .objc
            .register_host_selector("cancel:".to_owned(), &mut env.mem);
        let item: id = msg_class![env; UIBarButtonItem alloc];
        let item: id = msg![env; item initWithBarButtonSystemItem:1_i32
                                           target:target
                                           action:action];
        let title: id = msg![env; item title];
        let style: UIBarButtonItemStyle = msg![env; item style];
        assert_eq!(to_rust_string(env, title), "Cancel");
        assert_eq!(style, UIBarButtonItemStylePlain);
        release(env, item);
        release(env, target);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
