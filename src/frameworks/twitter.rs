/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The legacy Twitter framework.

use crate::dyld::HostDylib;
use crate::frameworks::foundation::NSUInteger;
use crate::objc::{id, msg, nil, objc_classes, ClassExports};

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

- (bool)setInitialText:(id)text {
    if text == nil {
        return true;
    }
    // Twitter's system composer accepted at most 140 UTF-16 code units in
    // this framework era.  There is no public getter and the host service is
    // unavailable, so validating the contract is sufficient without claiming
    // that a tweet can be sent.
    let length: NSUInteger = msg![env; text length];
    length <= 140
}

@end

};

#[cfg(test)]
mod tests {
    use crate::frameworks::foundation::ns_string;
    use crate::objc::{id, msg, msg_class, release};
    use crate::Environment;

    #[test]
    fn unavailable_twitter_service_reports_false() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let class: id = msg_class![env; TWTweetComposeViewController class];
        assert!(!class.is_null());
        let available: bool = msg_class![env; TWTweetComposeViewController canSendTweet];
        assert!(!available);

        let composer: id = msg_class![env; TWTweetComposeViewController new];
        let short = ns_string::from_rust_string(env, "hello".to_owned());
        let accepted: bool = msg![env; composer setInitialText:short];
        assert!(accepted);
        let long = ns_string::from_rust_string(env, "x".repeat(141));
        let accepted: bool = msg![env; composer setInitialText:long];
        assert!(!accepted);
        release(env, long);
        release(env, short);
        release(env, composer);
    }
}
