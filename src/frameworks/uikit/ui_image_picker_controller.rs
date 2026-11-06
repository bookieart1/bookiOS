/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIImagePickerController`

use crate::frameworks::foundation::NSInteger;
use crate::objc::{id, msg_super, objc_classes, ClassExports};
use std::collections::HashMap;

type UIImagePickerControllerSourceType = NSInteger;
const UIIMAGE_PICKER_CONTROLLER_SOURCE_TYPE_PHOTO_LIBRARY: UIImagePickerControllerSourceType = 0;

#[derive(Default)]
pub struct State {
    source_types: HashMap<id, UIImagePickerControllerSourceType>,
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// TODO: should extend UINavigationController, which extends
//       UIViewController.
@implementation UIImagePickerController: UIViewController

+ (bool)isSourceTypeAvailable:(UIImagePickerControllerSourceType)_type {
    // iDared does not expose a host camera or photo-library service. The
    // sourceType property remains real state even when none of those services
    // are available.
    false
}

- (UIImagePickerControllerSourceType)sourceType {
    env.framework_state
        .uikit
        .ui_image_picker_controller
        .source_types
        .get(&this)
        .copied()
        .unwrap_or(UIIMAGE_PICKER_CONTROLLER_SOURCE_TYPE_PHOTO_LIBRARY)
}

- (())setSourceType:(UIImagePickerControllerSourceType)source_type {
    env.framework_state
        .uikit
        .ui_image_picker_controller
        .source_types
        .insert(this, source_type);
}

- (())setDelegate:(id)_delegate {
    // TODO
}

- (())dealloc {
    env.framework_state
        .uikit
        .ui_image_picker_controller
        .source_types
        .remove(&this);
    msg_super![env; this dealloc]
}

@end

};

#[cfg(test)]
mod dev_239_tests {
    use super::*;
    use crate::objc::{msg, msg_class, release};
    use crate::Environment;

    #[test]
    fn dev_239_image_picker_source_type_has_default_and_round_trips() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let picker: id = msg_class![env; UIImagePickerController new];

        let initial: UIImagePickerControllerSourceType = msg![env; picker sourceType];
        assert_eq!(
            initial,
            UIIMAGE_PICKER_CONTROLLER_SOURCE_TYPE_PHOTO_LIBRARY
        );
        let _: () = msg![env; picker setSourceType:2];
        let selected: UIImagePickerControllerSourceType = msg![env; picker sourceType];
        assert_eq!(selected, 2);
        let available: bool =
            msg_class![env; UIImagePickerController isSourceTypeAvailable:2];
        assert!(!available);

        release(env, picker);
        assert!(env
            .framework_state
            .uikit
            .ui_image_picker_controller
            .source_types
            .is_empty());
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
