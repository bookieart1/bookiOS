/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded offline `GKAchievement` behavior.

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::{ns_string, NSInteger};
use crate::mem::{MutPtr, MutVoidPtr, SafeRead};
use crate::objc::{id, msg_class, nil, objc_classes, ClassExports};
use crate::Environment;

const GK_ERROR_DOMAIN: &str = "GKErrorDomain";
const GK_ERROR_NOT_AUTHENTICATED: NSInteger = 6;

#[repr(C)]
struct BlockLiteral {
    _isa: MutVoidPtr,
    _flags: i32,
    _reserved: i32,
    invoke: GuestFunction,
    _descriptor: MutVoidPtr,
}
unsafe impl SafeRead for BlockLiteral {}

fn not_authenticated_error(env: &mut Environment) -> id {
    let domain = ns_string::get_static_str(env, GK_ERROR_DOMAIN);
    msg_class![env; NSError errorWithDomain:domain
                              code:GK_ERROR_NOT_AUTHENTICATED
                          userInfo:nil]
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation GKAchievement: NSObject

+ (())loadAchievementsWithCompletionHandler:(MutPtr<BlockLiteral>)completion_handler {
    if completion_handler.is_null() {
        return;
    }
    // touchHLE does not authenticate a Game Center local player. Native
    // GameKit therefore completes this optional service request with
    // GKErrorNotAuthenticated rather than inventing achievement data.
    let error = not_authenticated_error(env);
    let invoke = env.mem.read(completion_handler).invoke;
    assert_ne!(invoke.addr_without_thumb_bit(), 0);
    () = invoke.call_from_host(env, (completion_handler.cast_void(), nil, error));
}

@end

};

pub const CONSTANTS: ConstantExports = &[(
    "_GKErrorDomain",
    HostConstant::NSString(GK_ERROR_DOMAIN),
)];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::msg;

    #[test]
    fn development_255_gamekit_offline_achievement_error_contract() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let error = not_authenticated_error(env);
        let code: NSInteger = msg![env; error code];
        let domain: id = msg![env; error domain];
        assert_eq!(code, GK_ERROR_NOT_AUTHENTICATED);
        assert_eq!(ns_string::to_rust_string(env, domain), GK_ERROR_DOMAIN);
    }
}
