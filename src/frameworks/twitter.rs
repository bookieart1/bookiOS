/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The legacy Twitter framework.

use crate::dyld::HostDylib;
use crate::objc::{objc_classes, ClassExports};

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/Twitter.framework/Twitter",
    aliases: &[],
    class_exports: &[CLASSES],
    constant_exports: &[],
    function_exports: &[],
};

const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation TWTweetComposeViewController: UIViewController

// The host cannot provide the removed system Twitter account service. This is
// the historical availability query, so returning false is the truthful
// offline result and lets applications keep their own fallback path.
+ (bool)canSendTweet {
    false
}

@end

};

#[cfg(test)]
mod tests {
    use crate::objc::{id, msg_class};
    use crate::Environment;

    #[test]
    fn unavailable_twitter_service_reports_false() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let class: id = msg_class![&mut env; TWTweetComposeViewController class];
        assert!(!class.is_null());
        let available: bool = msg_class![&mut env; TWTweetComposeViewController canSendTweet];
        assert!(!available);
    }
}
