/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `MPMediaPickerController`.

use crate::frameworks::foundation::NSUInteger;
use crate::objc::{id, msg, msg_super, nil, objc_classes, release, ClassExports};
use crate::Environment;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    pickers: HashMap<id, PickerState>,
}

#[derive(Default)]
struct PickerState {
    media_types: NSUInteger,
    delegate: id,
    allows_multiple: bool,
    prompt: id,
}

fn state(env: &mut Environment, picker: id) -> &mut PickerState {
    env.framework_state
        .media_player
        .media_picker_controller
        .pickers
        .entry(picker)
        .or_default()
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation MPMediaPickerController: UIViewController

- (id)initWithMediaTypes:(NSUInteger)media_types {
    let picker: id = msg_super![env; this initWithNibName:nil bundle:nil];
    state(env, picker).media_types = media_types;
    env.diagnostic_media_event("media-picker-init", picker, nil, media_types);
    picker
}
- (NSUInteger)mediaTypes {
    state(env, this).media_types
}
- (id)delegate {
    state(env, this).delegate
}
- (())setDelegate:(id)delegate {
    // MPMediaPickerController's delegate is historically assign/non-retaining.
    state(env, this).delegate = delegate;
}
- (bool)allowsPickingMultipleItems {
    state(env, this).allows_multiple
}
- (())setAllowsPickingMultipleItems:(bool)allows_multiple {
    state(env, this).allows_multiple = allows_multiple;
}
- (id)prompt {
    state(env, this).prompt
}
- (())setPrompt:(id)prompt {
    let old_prompt = state(env, this).prompt;
    let copied_prompt: id = msg![env; prompt copy];
    state(env, this).prompt = copied_prompt;
    release(env, old_prompt);
}
- (())dealloc {
    if let Some(picker) = env.framework_state
        .media_player
        .media_picker_controller
        .pickers
        .remove(&this)
    {
        release(env, picker.prompt);
    }
    msg_super![env; this dealloc]
}
@end

};

#[cfg(test)]
mod dev_233_tests {
    use super::*;

    #[test]
    fn media_picker_state_defaults_to_single_selection_without_a_delegate() {
        let picker = PickerState::default();
        assert_eq!(picker.media_types, 0);
        assert_eq!(picker.delegate, nil);
        assert!(!picker.allows_multiple);
        assert_eq!(picker.prompt, nil);
    }
}
