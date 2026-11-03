/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Historical `UINavigationBar` state contract.
//!
//! Layout, drawing and animated transitions remain out of scope. This keeps
//! the public stack and property semantics available to applications without
//! claiming that a native-looking navigation bar is rendered.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::{ns_array, NSInteger, NSUInteger};
use crate::objc::{
    autorelease, id, impl_HostObject_with_superclass, msg, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr,
};

#[derive(Default)]
struct UINavigationBarHostObject {
    superclass: super::UIViewHostObject,
    items: Vec<id>,
    delegate: id,
    bar_style: NSInteger,
    tint_color: id,
    translucent: bool,
}
impl_HostObject_with_superclass!(UINavigationBarHostObject);

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UINavigationBar: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UINavigationBarHostObject>::default(), &mut env.mem)
}

- (id)initWithFrame:(CGRect)frame {
    msg_super![env; this initWithFrame:frame]
}

- (id)initWithCoder:(id)coder {
    msg_super![env; this initWithCoder:coder]
}

- (id)delegate {
    env.objc.borrow::<UINavigationBarHostObject>(this).delegate
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UINavigationBarHostObject>(this).delegate = delegate;
}

- (id)items {
    let items = env.objc.borrow::<UINavigationBarHostObject>(this).items.clone();
    for item in &items {
        retain(env, *item);
    }
    let array = ns_array::from_vec(env, items);
    autorelease(env, array)
}
- (id)topItem {
    env.objc
        .borrow::<UINavigationBarHostObject>(this)
        .items
        .last()
        .copied()
        .unwrap_or(nil)
}
- (id)backItem {
    let items = &env.objc.borrow::<UINavigationBarHostObject>(this).items;
    if items.len() >= 2 {
        items[items.len() - 2]
    } else {
        nil
    }
}

- (())setItems:(id)items {
    () = msg![env; this setItems:items animated:false];
}
- (())setItems:(id)items animated:(bool)_animated {
    let count: NSUInteger = msg![env; items count];
    let mut new_items = Vec::with_capacity(count as usize);
    for index in 0..count {
        let item: id = msg![env; items objectAtIndex:index];
        retain(env, item);
        new_items.push(item);
    }
    let old_items = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationBarHostObject>(this).items,
        new_items,
    );
    for item in old_items {
        release(env, item);
    }
}

- (())pushNavigationItem:(id)item animated:(bool)_animated {
    retain(env, item);
    env.objc.borrow_mut::<UINavigationBarHostObject>(this).items.push(item);
}
- (id)popNavigationItemAnimated:(bool)_animated {
    let item = env
        .objc
        .borrow_mut::<UINavigationBarHostObject>(this)
        .items
        .pop()
        .unwrap_or(nil);
    if item != nil {
        autorelease(env, item);
    }
    item
}

- (NSInteger)barStyle {
    env.objc.borrow::<UINavigationBarHostObject>(this).bar_style
}
- (())setBarStyle:(NSInteger)style {
    env.objc.borrow_mut::<UINavigationBarHostObject>(this).bar_style = style;
}
- (id)tintColor {
    env.objc.borrow::<UINavigationBarHostObject>(this).tint_color
}
- (())setTintColor:(id)color {
    retain(env, color);
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<UINavigationBarHostObject>(this).tint_color,
        color,
    );
    release(env, old);
}
- (bool)isTranslucent {
    env.objc.borrow::<UINavigationBarHostObject>(this).translucent
}
- (())setTranslucent:(bool)translucent {
    env.objc.borrow_mut::<UINavigationBarHostObject>(this).translucent = translucent;
}

- (())dealloc {
    let (items, tint_color) = {
        let host = env.objc.borrow_mut::<UINavigationBarHostObject>(this);
        (std::mem::take(&mut host.items), host.tint_color)
    };
    for item in items {
        release(env, item);
    }
    release(env, tint_color);
    msg_super![env; this dealloc]
}

@end

};
