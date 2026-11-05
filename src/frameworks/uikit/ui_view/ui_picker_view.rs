/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIPickerView`.

use crate::frameworks::foundation::NSInteger;
use crate::impl_HostObject_with_superclass;
use crate::objc::{id, msg, nil, objc_classes, ClassExports, NSZonePtr, SEL};

#[derive(Default)]
struct UIPickerViewHostObject {
    superclass: super::UIViewHostObject,
    delegate: id,
    data_source: id,
    shows_selection_indicator: bool,
    selected_rows: Vec<NSInteger>,
}
impl_HostObject_with_superclass!(UIPickerViewHostObject);

fn component_count(env: &mut crate::Environment, this: id) -> NSInteger {
    let data_source = env.objc.borrow::<UIPickerViewHostObject>(this).data_source;
    if data_source == nil {
        return 0;
    }
    let selector: SEL = env
        .objc
        .register_host_selector("numberOfComponentsInPickerView:".to_string(), &mut env.mem);
    if !msg![env; data_source respondsToSelector:selector] {
        return 0;
    }
    crate::objc::msg_send(env, (data_source, selector, this))
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIPickerView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UIPickerViewHostObject>::default(), &mut env.mem)
}

- (())setShowsSelectionIndicator:(bool)shows {
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).shows_selection_indicator = shows;
}
- (bool)showsSelectionIndicator {
    env.objc.borrow::<UIPickerViewHostObject>(this).shows_selection_indicator
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UIPickerViewHostObject>(this).delegate
}
- (())setDataSource:(id)data_source {
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).data_source = data_source;
    () = msg![env; this reloadAllComponents];
}
- (id)dataSource {
    env.objc.borrow::<UIPickerViewHostObject>(this).data_source
}
- (NSInteger)numberOfComponents {
    component_count(env, this)
}
- (NSInteger)numberOfRowsInComponent:(NSInteger)component {
    if component < 0 || component >= component_count(env, this) {
        return 0;
    }
    let data_source = env.objc.borrow::<UIPickerViewHostObject>(this).data_source;
    let selector: SEL = env.objc.register_host_selector(
        "pickerView:numberOfRowsInComponent:".to_string(), &mut env.mem);
    if !msg![env; data_source respondsToSelector:selector] {
        return 0;
    }
    crate::objc::msg_send(env, (data_source, selector, this, component))
}
- (())reloadAllComponents {
    let count = usize::try_from(component_count(env, this).max(0)).unwrap();
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).selected_rows.resize(count, 0);
    for component in 0..count {
        let row_count: NSInteger = msg![env; this numberOfRowsInComponent:(component as NSInteger)];
        let rows = &mut env.objc.borrow_mut::<UIPickerViewHostObject>(this).selected_rows;
        rows[component] = if row_count <= 0 { -1 } else { rows[component].clamp(0, row_count - 1) };
    }
}
- (())reloadComponent:(NSInteger)component {
    if component < 0 {
        return;
    }
    let row_count: NSInteger = msg![env; this numberOfRowsInComponent:component];
    let rows = &mut env.objc.borrow_mut::<UIPickerViewHostObject>(this).selected_rows;
    if let Some(row) = rows.get_mut(component as usize) {
        *row = if row_count <= 0 { -1 } else { (*row).clamp(0, row_count - 1) };
    }
}
- (())selectRow:(NSInteger)row inComponent:(NSInteger)component animated:(bool)_animated {
    if component < 0 || row < 0 {
        return;
    }
    let count: NSInteger = msg![env; this numberOfRowsInComponent:component];
    if row >= count {
        return;
    }
    if let Some(selected) = env.objc.borrow_mut::<UIPickerViewHostObject>(this)
        .selected_rows.get_mut(component as usize) {
        *selected = row;
    }
}
- (NSInteger)selectedRowInComponent:(NSInteger)component {
    if component < 0 {
        return -1;
    }
    env.objc.borrow::<UIPickerViewHostObject>(this).selected_rows
        .get(component as usize).copied().unwrap_or(-1)
}

@end

};
