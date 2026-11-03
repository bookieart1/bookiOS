/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UINavigationItem` state owned by a `UIViewController`.
//!
//! This models the historical ownership and property contract. Rendering and
//! navigation-bar animations remain the responsibility of a future bar model.

use crate::objc::{
    id, msg, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};

#[derive(Default)]
struct UINavigationItemHostObject {
    title: id,
    prompt: id,
    title_view: id,
    left_bar_button_item: id,
    right_bar_button_item: id,
    back_bar_button_item: id,
    hides_back_button: bool,
}
impl HostObject for UINavigationItemHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UINavigationItem: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::<UINavigationItemHostObject>::default(),
        &mut env.mem,
    )
}

- (id)initWithTitle:(id)title {
    let title: id = msg![env; title copy];
    env.objc.borrow_mut::<UINavigationItemHostObject>(this).title = title;
    this
}

- (id)initWithCoder:(id)_coder {
    this
}

- (id)title {
    env.objc.borrow::<UINavigationItemHostObject>(this).title
}
- (())setTitle:(id)title {
    let title: id = msg![env; title copy];
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).title,
        title,
    );
    release(env, old);
}

- (id)prompt {
    env.objc.borrow::<UINavigationItemHostObject>(this).prompt
}
- (())setPrompt:(id)prompt {
    let prompt: id = msg![env; prompt copy];
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).prompt,
        prompt,
    );
    release(env, old);
}

- (id)titleView {
    env.objc.borrow::<UINavigationItemHostObject>(this).title_view
}
- (())setTitleView:(id)view {
    retain(env, view);
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).title_view,
        view,
    );
    release(env, old);
}

- (id)leftBarButtonItem {
    env.objc.borrow::<UINavigationItemHostObject>(this).left_bar_button_item
}
- (())setLeftBarButtonItem:(id)item {
    retain(env, item);
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).left_bar_button_item,
        item,
    );
    release(env, old);
}
- (())setLeftBarButtonItem:(id)item animated:(bool)_animated {
    () = msg![env; this setLeftBarButtonItem:item];
}

- (id)rightBarButtonItem {
    env.objc.borrow::<UINavigationItemHostObject>(this).right_bar_button_item
}
- (())setRightBarButtonItem:(id)item {
    retain(env, item);
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).right_bar_button_item,
        item,
    );
    release(env, old);
}
- (())setRightBarButtonItem:(id)item animated:(bool)_animated {
    () = msg![env; this setRightBarButtonItem:item];
}

- (id)backBarButtonItem {
    env.objc.borrow::<UINavigationItemHostObject>(this).back_bar_button_item
}
- (())setBackBarButtonItem:(id)item {
    retain(env, item);
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationItemHostObject>(this).back_bar_button_item,
        item,
    );
    release(env, old);
}

- (bool)hidesBackButton {
    env.objc.borrow::<UINavigationItemHostObject>(this).hides_back_button
}
- (())setHidesBackButton:(bool)hidden {
    env.objc
        .borrow_mut::<UINavigationItemHostObject>(this)
        .hides_back_button = hidden;
}
- (())setHidesBackButton:(bool)hidden animated:(bool)_animated {
    () = msg![env; this setHidesBackButton:hidden];
}

- (())dealloc {
    let &UINavigationItemHostObject {
        title,
        prompt,
        title_view,
        left_bar_button_item,
        right_bar_button_item,
        back_bar_button_item,
        ..
    } = env.objc.borrow(this);
    release(env, title);
    release(env, prompt);
    release(env, title_view);
    release(env, left_bar_button_item);
    release(env, right_bar_button_item);
    release(env, back_bar_button_item);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

};
