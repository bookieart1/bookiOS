/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The Core Data framework.
//!
//! Only class identity is modeled initially. This is enough for inherited
//! NSObject allocation to behave correctly while keeping every unsupported
//! persistence operation visible instead of pretending that a store exists.

use crate::dyld::HostDylib;
use crate::objc::{objc_classes, ClassExports};

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/CoreData.framework/CoreData",
    aliases: &[],
    class_exports: &[CLASSES],
    constant_exports: &[],
    function_exports: &[],
};

const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPersistentStoreCoordinator: NSObject
@end

};

#[cfg(test)]
mod tests {
    use crate::objc::{id, msg_class, release};
    use crate::Environment;

    #[test]
    fn persistent_store_coordinator_has_real_class_identity_and_allocation() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let object: id = msg_class![&mut env; NSPersistentStoreCoordinator new];
        assert!(!object.is_null());
        release(&mut env, object);
    }
}
