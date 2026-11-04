/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableViewCell`.

use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr,
};

type UITableViewCellStyle = NSInteger;
type UITableViewCellAccessoryType = NSInteger;
type UITableViewCellSelectionStyle = NSInteger;

struct UITableViewCellHostObject {
    superclass: super::UIViewHostObject,
    style: UITableViewCellStyle,
    reuse_identifier: id,
    content_view: id,
    text_label: id,
    detail_text_label: id,
    image_view: id,
    accessory_view: id,
    background_view: id,
    selected_background_view: id,
    accessory_type: UITableViewCellAccessoryType,
    selection_style: UITableViewCellSelectionStyle,
    selected: bool,
    highlighted: bool,
    editing: bool,
}
impl_HostObject_with_superclass!(UITableViewCellHostObject);

impl Default for UITableViewCellHostObject {
    fn default() -> Self {
        Self {
            superclass: Default::default(),
            style: 0,
            reuse_identifier: nil,
            content_view: nil,
            text_label: nil,
            detail_text_label: nil,
            image_view: nil,
            accessory_view: nil,
            background_view: nil,
            selected_background_view: nil,
            accessory_type: 0,
            selection_style: 1,
            selected: false,
            highlighted: false,
            editing: false,
        }
    }
}

fn init_common(env: &mut crate::Environment, this: id, reuse_identifier: id) -> id {
    let reuse_identifier: id = msg![env; reuse_identifier copy];
    let content_view: id = msg_class![env; UIView new];
    let text_label: id = msg_class![env; UILabel new];
    let detail_text_label: id = msg_class![env; UILabel new];
    let image_view: id = msg_class![env; UIImageView new];

    () = msg![env; content_view addSubview:image_view];
    () = msg![env; content_view addSubview:text_label];
    () = msg![env; content_view addSubview:detail_text_label];
    () = msg![env; this addSubview:content_view];

    let host = env.objc.borrow_mut::<UITableViewCellHostObject>(this);
    host.reuse_identifier = reuse_identifier;
    host.content_view = content_view;
    host.text_label = text_label;
    host.detail_text_label = detail_text_label;
    host.image_view = image_view;
    this
}

fn replace_retained_view(
    env: &mut crate::Environment,
    this: id,
    new_view: id,
    select: impl FnOnce(&mut UITableViewCellHostObject) -> &mut id,
) {
    retain(env, new_view);
    let old = {
        let host = env.objc.borrow_mut::<UITableViewCellHostObject>(this);
        std::mem::replace(select(host), new_view)
    };
    release(env, old);
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableViewCell: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UITableViewCellHostObject>::default(), &mut env.mem)
}

- (id)initWithStyle:(UITableViewCellStyle)style reuseIdentifier:(id)reuse_identifier {
    let this: id = msg_super![env; this initWithFrame:(CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize { width: 320.0, height: 44.0 },
    })];
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).style = style;
    init_common(env, this, reuse_identifier)
}

- (id)initWithFrame:(CGRect)frame reuseIdentifier:(id)reuse_identifier {
    let this: id = msg_super![env; this initWithFrame:frame];
    init_common(env, this, reuse_identifier)
}

- (id)initWithCoder:(id)coder {
    let this: id = msg_super![env; this initWithCoder:coder];
    init_common(env, this, nil)
}

- (())dealloc {
    let UITableViewCellHostObject {
        superclass: _,
        style: _,
        reuse_identifier,
        content_view,
        text_label,
        detail_text_label,
        image_view,
        accessory_view,
        background_view,
        selected_background_view,
        accessory_type: _,
        selection_style: _,
        selected: _,
        highlighted: _,
        editing: _,
    } = std::mem::take(env.objc.borrow_mut(this));
    release(env, reuse_identifier);
    release(env, content_view);
    release(env, text_label);
    release(env, detail_text_label);
    release(env, image_view);
    release(env, accessory_view);
    release(env, background_view);
    release(env, selected_background_view);
    msg_super![env; this dealloc]
}

- (id)reuseIdentifier {
    env.objc.borrow::<UITableViewCellHostObject>(this).reuse_identifier
}
- (id)contentView {
    env.objc.borrow::<UITableViewCellHostObject>(this).content_view
}
- (id)textLabel {
    env.objc.borrow::<UITableViewCellHostObject>(this).text_label
}
- (id)detailTextLabel {
    env.objc.borrow::<UITableViewCellHostObject>(this).detail_text_label
}
- (id)imageView {
    env.objc.borrow::<UITableViewCellHostObject>(this).image_view
}

- (id)text {
    let label = env.objc.borrow::<UITableViewCellHostObject>(this).text_label;
    msg![env; label text]
}
- (())setText:(id)text {
    let label = env.objc.borrow::<UITableViewCellHostObject>(this).text_label;
    () = msg![env; label setText:text];
}
- (id)image {
    let image_view = env.objc.borrow::<UITableViewCellHostObject>(this).image_view;
    msg![env; image_view image]
}
- (())setImage:(id)image {
    let image_view = env.objc.borrow::<UITableViewCellHostObject>(this).image_view;
    () = msg![env; image_view setImage:image];
}

- (UITableViewCellAccessoryType)accessoryType {
    env.objc.borrow::<UITableViewCellHostObject>(this).accessory_type
}
- (())setAccessoryType:(UITableViewCellAccessoryType)accessory_type {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).accessory_type = accessory_type;
}
- (id)accessoryView {
    env.objc.borrow::<UITableViewCellHostObject>(this).accessory_view
}
- (())setAccessoryView:(id)view {
    replace_retained_view(env, this, view, |host| &mut host.accessory_view);
}
- (id)backgroundView {
    env.objc.borrow::<UITableViewCellHostObject>(this).background_view
}
- (())setBackgroundView:(id)view {
    replace_retained_view(env, this, view, |host| &mut host.background_view);
}
- (id)selectedBackgroundView {
    env.objc.borrow::<UITableViewCellHostObject>(this).selected_background_view
}
- (())setSelectedBackgroundView:(id)view {
    replace_retained_view(env, this, view, |host| &mut host.selected_background_view);
}

- (UITableViewCellSelectionStyle)selectionStyle {
    env.objc.borrow::<UITableViewCellHostObject>(this).selection_style
}
- (())setSelectionStyle:(UITableViewCellSelectionStyle)style {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).selection_style = style;
}
- (bool)isSelected {
    env.objc.borrow::<UITableViewCellHostObject>(this).selected
}
- (())setSelected:(bool)selected animated:(bool)_animated {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).selected = selected;
}
- (bool)isHighlighted {
    env.objc.borrow::<UITableViewCellHostObject>(this).highlighted
}
- (())setHighlighted:(bool)highlighted animated:(bool)_animated {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).highlighted = highlighted;
}
- (bool)isEditing {
    env.objc.borrow::<UITableViewCellHostObject>(this).editing
}
- (())setEditing:(bool)editing animated:(bool)_animated {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).editing = editing;
}

- (())prepareForReuse {
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).selected = false;
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).highlighted = false;
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).editing = false;
}

- (())layoutSubviews {
    let bounds: CGRect = msg![env; this bounds];
    let (content_view, text_label, detail_text_label, image_view, accessory_view) = {
        let host = env.objc.borrow::<UITableViewCellHostObject>(this);
        (host.content_view, host.text_label, host.detail_text_label, host.image_view, host.accessory_view)
    };
    () = msg![env; content_view setFrame:bounds];
    let image_width: CGFloat = 40.0;
    () = msg![env; image_view setFrame:(CGRect {
        origin: CGPoint { x: 4.0, y: 4.0 },
        size: CGSize { width: image_width - 8.0, height: (bounds.size.height - 8.0).max(0.0) },
    })];
    let accessory_width = if accessory_view == nil { 0.0 } else { 40.0 };
    let label_frame = CGRect {
        origin: CGPoint { x: image_width, y: 0.0 },
        size: CGSize {
            width: (bounds.size.width - image_width - accessory_width).max(0.0),
            height: bounds.size.height / 2.0,
        },
    };
    () = msg![env; text_label setFrame:label_frame];
    () = msg![env; detail_text_label setFrame:(CGRect {
        origin: CGPoint { x: label_frame.origin.x, y: label_frame.size.height },
        size: label_frame.size,
    })];
    if accessory_view != nil {
        () = msg![env; accessory_view setFrame:(CGRect {
            origin: CGPoint { x: bounds.size.width - accessory_width, y: 0.0 },
            size: CGSize { width: accessory_width, height: bounds.size.height },
        })];
    }
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;
    use crate::Environment;

    #[test]
    fn cell_keeps_identity_state_and_standard_subviews() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let reuse = ns_string::from_rust_string(env, "Cell".to_owned());
        let cell: id = msg_class![env; UITableViewCell alloc];
        let cell: id = msg![env; cell initWithStyle:0i32 reuseIdentifier:reuse];
        let actual: id = msg![env; cell reuseIdentifier];
        assert_eq!(ns_string::to_rust_string(env, actual), "Cell");
        let content_view: id = msg![env; cell contentView];
        let text_label: id = msg![env; cell textLabel];
        let image_view: id = msg![env; cell imageView];
        assert_ne!(content_view, nil);
        assert_ne!(text_label, nil);
        assert_ne!(image_view, nil);
        () = msg![env; cell setSelected:true animated:false];
        let selected: bool = msg![env; cell isSelected];
        assert!(selected);
        () = msg![env; cell setEditing:true animated:true];
        let editing: bool = msg![env; cell isEditing];
        assert!(editing);
        () = msg![env; cell prepareForReuse];
        let editing: bool = msg![env; cell isEditing];
        let selected: bool = msg![env; cell isSelected];
        assert!(!editing);
        assert!(!selected);
        release(env, cell);
        release(env, reuse);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
