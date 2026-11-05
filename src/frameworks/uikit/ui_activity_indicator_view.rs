/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIActivityIndicatorView`.

use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, objc_classes, todo_objc_setter, ClassExports,
    NSZonePtr,
};

type UIActivityIndicatorViewStyle = NSInteger;

pub struct UIActivityIndicatorViewHostObject {
    superclass: super::ui_view::UIViewHostObject,
    animating: bool,
    hides_when_stopped: bool,
}
impl_HostObject_with_superclass!(UIActivityIndicatorViewHostObject);

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIActivityIndicatorView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIActivityIndicatorViewHostObject {
        superclass: Default::default(),
        animating: false,
        hides_when_stopped: true,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithActivityIndicatorStyle:(UIActivityIndicatorViewStyle)_style {
    // The historical default is to hide a stopped indicator.
    let initialized: id = msg![env; this init];
    if initialized != crate::objc::nil {
        () = msg![env; initialized setHidden:true];
    }
    initialized
}

- (())setActivityIndicatorViewStyle:(UIActivityIndicatorViewStyle)style {
    todo_objc_setter!(this, style);
}

- (())startAnimating {
    log!("TODO: [(UIActivityIndicatorView *){:?} startAnimating]", this);
    env.objc.borrow_mut::<UIActivityIndicatorViewHostObject>(this).animating = true;
    () = msg![env; this setHidden:false];
}
- (())stopAnimating {
    log!("TODO: [(UIActivityIndicatorView *){:?} stopAnimating]", this);
    let hides = {
        let host = env.objc.borrow_mut::<UIActivityIndicatorViewHostObject>(this);
        host.animating = false;
        host.hides_when_stopped
    };
    if hides {
        () = msg![env; this setHidden:true];
    }
}

- (bool)isAnimating {
    env.objc.borrow::<UIActivityIndicatorViewHostObject>(this).animating
}

- (bool)hidesWhenStopped {
    env.objc.borrow::<UIActivityIndicatorViewHostObject>(this).hides_when_stopped
}

- (())setHidesWhenStopped:(bool)hides {
    let should_hide = {
        let host = env.objc.borrow_mut::<UIActivityIndicatorViewHostObject>(this);
        host.hides_when_stopped = hides;
        hides && !host.animating
    };
    () = msg![env; this setHidden:should_hide];
}

@end

};

#[cfg(test)]
mod tests {
    use crate::objc::{id, msg, msg_class, release};
    use crate::Environment;

    #[test]
    fn hides_when_stopped_defaults_true_and_controls_visibility() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let indicator: id = msg_class![env; UIActivityIndicatorView alloc];
        let indicator: id = msg![env; indicator initWithActivityIndicatorStyle:0i32];
        assert!(msg![env; indicator hidesWhenStopped]);
        assert!(msg![env; indicator isHidden]);

        () = msg![env; indicator startAnimating];
        assert!(!msg![env; indicator isHidden]);
        () = msg![env; indicator stopAnimating];
        assert!(msg![env; indicator isHidden]);

        () = msg![env; indicator setHidesWhenStopped:false];
        assert!(!msg![env; indicator hidesWhenStopped]);
        assert!(!msg![env; indicator isHidden]);
        release(env, indicator);
    }
}
