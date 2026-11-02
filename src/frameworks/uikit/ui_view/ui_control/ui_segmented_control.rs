/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UISegmentedControl`.

use super::{send_actions, UIControlEventValueChanged};
use crate::frameworks::core_graphics::{CGPoint, CGRect};
use crate::frameworks::foundation::{NSInteger, NSUInteger};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, Class, ClassExports, NSZonePtr,
};
use crate::Environment;

const UISegmentedControlNoSegment: NSInteger = -1;

struct Segment {
    title: id,
    image: id,
    enabled: bool,
    width: f32,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            title: nil,
            image: nil,
            enabled: true,
            width: 0.0,
        }
    }
}

pub struct UISegmentedControlHostObject {
    superclass: super::UIControlHostObject,
    segments: Vec<Segment>,
    selected_segment_index: NSInteger,
    momentary: bool,
    style: NSInteger,
}
impl_HostObject_with_superclass!(UISegmentedControlHostObject);

impl Default for UISegmentedControlHostObject {
    fn default() -> Self {
        Self {
            superclass: Default::default(),
            segments: Vec::new(),
            selected_segment_index: UISegmentedControlNoSegment,
            momentary: false,
            style: 0,
        }
    }
}

fn segment_for_item(env: &mut Environment, item: id) -> Segment {
    assert_ne!(item, nil);
    let string_class: Class = msg_class![env; NSString class];
    let image_class: Class = msg_class![env; UIImage class];
    if msg![env; item isKindOfClass:string_class] {
        Segment {
            title: retain(env, item),
            ..Default::default()
        }
    } else if msg![env; item isKindOfClass:image_class] {
        Segment {
            image: retain(env, item),
            ..Default::default()
        }
    } else {
        panic!(
            "UISegmentedControl item {:?} is neither an NSString nor UIImage",
            item
        );
    }
}

fn release_segment(env: &mut Environment, segment: Segment) {
    release(env, segment.title);
    release(env, segment.image);
}

fn checked_index(host: &UISegmentedControlHostObject, index: NSUInteger) -> usize {
    let index = usize::try_from(index).unwrap();
    assert!(index < host.segments.len());
    index
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UISegmentedControl: UIControl

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UISegmentedControlHostObject>::default(), &mut env.mem)
}

- (id)initWithFrame:(CGRect)frame {
    msg_super![env; this initWithFrame:frame]
}

- (id)initWithItems:(id)items { // NSArray<NSString* or UIImage*>*
    let this: id = msg_super![env; this initWithFrame:CGRect::default()];
    if items == nil {
        return this;
    }
    let count: NSUInteger = msg![env; items count];
    let mut segments = Vec::with_capacity(usize::try_from(count).unwrap());
    for index in 0..count {
        let item: id = msg![env; items objectAtIndex:index];
        segments.push(segment_for_item(env, item));
    }
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).segments = segments;
    this
}

// Archived segment subviews remain a rendering limitation, but the control
// now has valid bounded state and accessors.
- (id)initWithCoder:(id)coder {
    msg_super![env; this initWithCoder:coder]
}

- (())dealloc {
    let host = std::mem::take(env.objc.borrow_mut::<UISegmentedControlHostObject>(this));
    for segment in host.segments {
        release_segment(env, segment);
    }
    msg_super![env; this dealloc]
}

- (NSUInteger)numberOfSegments {
    env.objc.borrow::<UISegmentedControlHostObject>(this).segments.len().try_into().unwrap()
}

- (())insertSegmentWithTitle:(id)title atIndex:(NSUInteger)index animated:(bool)_animated {
    let len = env.objc.borrow::<UISegmentedControlHostObject>(this).segments.len();
    let index = usize::try_from(index).unwrap();
    assert!(index <= len);
    let segment = Segment { title: retain(env, title), ..Default::default() };
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).segments.insert(index, segment);
}

- (())insertSegmentWithImage:(id)image atIndex:(NSUInteger)index animated:(bool)_animated {
    let len = env.objc.borrow::<UISegmentedControlHostObject>(this).segments.len();
    let index = usize::try_from(index).unwrap();
    assert!(index <= len);
    let segment = Segment { image: retain(env, image), ..Default::default() };
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).segments.insert(index, segment);
}

- (())removeSegmentAtIndex:(NSUInteger)index animated:(bool)_animated {
    let index = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        checked_index(host, index)
    };
    let host = env.objc.borrow_mut::<UISegmentedControlHostObject>(this);
    let segment = host.segments.remove(index);
    if host.selected_segment_index == index as NSInteger {
        host.selected_segment_index = UISegmentedControlNoSegment;
    } else if host.selected_segment_index > index as NSInteger {
        host.selected_segment_index -= 1;
    }
    release_segment(env, segment);
}

- (())removeAllSegments {
    let host = env.objc.borrow_mut::<UISegmentedControlHostObject>(this);
    let segments = std::mem::take(&mut host.segments);
    host.selected_segment_index = UISegmentedControlNoSegment;
    for segment in segments {
        release_segment(env, segment);
    }
}

- (id)titleForSegmentAtIndex:(NSUInteger)index {
    let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
    host.segments[checked_index(host, index)].title
}

- (())setTitle:(id)title forSegmentAtIndex:(NSUInteger)index {
    let index = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        checked_index(host, index)
    };
    retain(env, title);
    let host = env.objc.borrow_mut::<UISegmentedControlHostObject>(this);
    let old_title = std::mem::replace(&mut host.segments[index].title, title);
    let old_image = std::mem::replace(&mut host.segments[index].image, nil);
    release(env, old_title);
    release(env, old_image);
}

- (id)imageForSegmentAtIndex:(NSUInteger)index {
    let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
    host.segments[checked_index(host, index)].image
}

- (())setImage:(id)image forSegmentAtIndex:(NSUInteger)index {
    let index = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        checked_index(host, index)
    };
    retain(env, image);
    let host = env.objc.borrow_mut::<UISegmentedControlHostObject>(this);
    let old_image = std::mem::replace(&mut host.segments[index].image, image);
    let old_title = std::mem::replace(&mut host.segments[index].title, nil);
    release(env, old_image);
    release(env, old_title);
}

- (NSInteger)selectedSegmentIndex {
    env.objc.borrow::<UISegmentedControlHostObject>(this).selected_segment_index
}

- (())setSelectedSegmentIndex:(NSInteger)index {
    let count = env.objc.borrow::<UISegmentedControlHostObject>(this).segments.len() as NSInteger;
    assert!(index == UISegmentedControlNoSegment || (0..count).contains(&index));
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).selected_segment_index = index;
}

- (bool)isMomentary {
    env.objc.borrow::<UISegmentedControlHostObject>(this).momentary
}

- (())setMomentary:(bool)momentary {
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).momentary = momentary;
}

- (NSInteger)segmentedControlStyle {
    env.objc.borrow::<UISegmentedControlHostObject>(this).style
}

- (())setSegmentedControlStyle:(NSInteger)style {
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).style = style;
}

- (bool)isEnabledForSegmentAtIndex:(NSUInteger)index {
    let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
    host.segments[checked_index(host, index)].enabled
}

- (())setEnabled:(bool)enabled forSegmentAtIndex:(NSUInteger)index {
    let index = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        checked_index(host, index)
    };
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).segments[index].enabled = enabled;
}

- (f32)widthForSegmentAtIndex:(NSUInteger)index {
    let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
    host.segments[checked_index(host, index)].width
}

- (())setWidth:(f32)width forSegmentAtIndex:(NSUInteger)index {
    let index = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        checked_index(host, index)
    };
    env.objc.borrow_mut::<UISegmentedControlHostObject>(this).segments[index].width = width.max(0.0);
}

- (())endTrackingWithTouch:(id)touch withEvent:(id)event {
    let bounds: CGRect = msg![env; this bounds];
    let point: CGPoint = msg![env; touch locationInView:this];
    let (count, momentary) = {
        let host = env.objc.borrow::<UISegmentedControlHostObject>(this);
        (host.segments.len(), host.momentary)
    };
    if count > 0 && bounds.size.width > 0.0 &&
        point.x >= bounds.origin.x && point.x < bounds.origin.x + bounds.size.width {
        let relative = (point.x - bounds.origin.x) / bounds.size.width;
        let index = ((relative * count as f32) as usize).min(count - 1);
        if env.objc.borrow::<UISegmentedControlHostObject>(this).segments[index].enabled {
            env.objc.borrow_mut::<UISegmentedControlHostObject>(this).selected_segment_index = index as NSInteger;
            send_actions(env, this, event, UIControlEventValueChanged);
            if momentary {
                env.objc.borrow_mut::<UISegmentedControlHostObject>(this).selected_segment_index = UISegmentedControlNoSegment;
            }
        }
    }
    () = msg_super![env; this endTrackingWithTouch:touch withEvent:event];
}

@end

// Undocumented class used by UISegmentedControl
@implementation UISegment: UIControl

- (id)initWithFrame:(CGRect)frame {
    log!("[(UISegment*){:?} initWithFrame:{:?}] Attempted to initialize undocumented class from outside of the NIB.", this, frame);
    unreachable!()
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    log!("[(UISegment*){:?} initWithCoder:{:?}] TODO: UISegment archive rendering remains unsupported.", this, coder);
    msg_super![env; this initWithCoder:coder]
}

// TODO: all of it

@end

};
