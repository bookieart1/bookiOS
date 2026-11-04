/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal synchronous `NSOperation` and `NSOperationQueue` support.
//!
//! The real classes can execute work concurrently. Running operations
//! synchronously is deterministic and provides the same observable ordering
//! for older apps that use a queue only to defer small pieces of work.

use super::{ns_array, NSInteger, NSUInteger};
use crate::libc::dispatch::{invoke_block_synchronously, BlockLiteral};
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_send_no_type_checking, nil, objc_classes, release, retain,
    ClassExports, HostObject, NSZonePtr, SEL,
};
use crate::Environment;

#[derive(Default)]
pub struct State {
    main_queue: Option<id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut State {
        &mut env.framework_state.foundation.ns_operation
    }
}

struct NSOperationHostObject {
    target: id,
    selector: Option<SEL>,
    object: id,
    invocation: id,
    cancelled: bool,
    executing: bool,
    finished: bool,
    queue_priority: NSInteger,
}
impl HostObject for NSOperationHostObject {}

struct NSOperationQueueHostObject {
    suspended: bool,
    max_concurrent_operation_count: NSInteger,
    name: id,
    pending: Vec<id>,
}
impl HostObject for NSOperationQueueHostObject {}

fn run_operation(env: &mut Environment, operation: id) {
    retain(env, operation);
    let _: () = msg![env; operation start];
    release(env, operation);
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSOperation: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSOperationHostObject {
        target: nil,
        selector: None,
        object: nil,
        invocation: nil,
        cancelled: false,
        executing: false,
        finished: false,
        queue_priority: 0,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())start {
    if env.objc.borrow::<NSOperationHostObject>(this).finished {
        return;
    }
    if env.objc.borrow::<NSOperationHostObject>(this).cancelled {
        env.objc.borrow_mut::<NSOperationHostObject>(this).finished = true;
        return;
    }
    env.objc.borrow_mut::<NSOperationHostObject>(this).executing = true;
    let _: () = msg![env; this main];
    let host_object = env.objc.borrow_mut::<NSOperationHostObject>(this);
    host_object.executing = false;
    host_object.finished = true;
}

- (())main {}

- (())cancel {
    env.objc.borrow_mut::<NSOperationHostObject>(this).cancelled = true;
}
- (bool)isCancelled {
    env.objc.borrow::<NSOperationHostObject>(this).cancelled
}
- (bool)isExecuting {
    env.objc.borrow::<NSOperationHostObject>(this).executing
}
- (bool)isFinished {
    env.objc.borrow::<NSOperationHostObject>(this).finished
}
- (bool)isConcurrent { false }
- (bool)isReady { true }
- (())waitUntilFinished {}

- (NSInteger)queuePriority {
    env.objc.borrow::<NSOperationHostObject>(this).queue_priority
}
- (())setQueuePriority:(NSInteger)priority {
    env.objc.borrow_mut::<NSOperationHostObject>(this).queue_priority = priority;
}

- (id)dependencies {
    let dependencies = ns_array::from_vec(env, Vec::new());
    autorelease(env, dependencies)
}
- (())addDependency:(id)_operation {}
- (())removeDependency:(id)_operation {}

- (())dealloc {
    let host_object = env.objc.borrow::<NSOperationHostObject>(this);
    let target = host_object.target;
    let object = host_object.object;
    let invocation = host_object.invocation;
    release(env, target);
    release(env, object);
    release(env, invocation);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation NSInvocationOperation: NSOperation

- (id)initWithInvocation:(id)invocation {
    assert_ne!(invocation, nil, "NSInvocationOperation requires an invocation");
    retain(env, invocation);
    env.objc.borrow_mut::<NSOperationHostObject>(this).invocation = invocation;
    this
}

- (id)initWithTarget:(id)target selector:(SEL)selector object:(id)object {
    retain(env, target);
    retain(env, object);
    let host_object = env.objc.borrow_mut::<NSOperationHostObject>(this);
    host_object.target = target;
    host_object.selector = Some(selector);
    host_object.object = object;
    this
}

- (())main {
    let host_object = env.objc.borrow::<NSOperationHostObject>(this);
    let invocation = host_object.invocation;
    let target = host_object.target;
    let selector = host_object.selector;
    let object = host_object.object;
    if invocation != nil {
        let _: () = msg![env; invocation invoke];
    } else if target != nil {
        if let Some(selector) = selector {
            let _: () = msg_send_no_type_checking(env, (target, selector, object));
        }
    }
}

- (id)invocation {
    env.objc.borrow::<NSOperationHostObject>(this).invocation
}

- (id)result { nil }

@end

@implementation NSOperationQueue: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSOperationQueueHostObject {
        suspended: false,
        max_concurrent_operation_count: -1,
        name: nil,
        pending: Vec::new(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)mainQueue {
    if let Some(queue) = State::get(env).main_queue {
        return queue;
    }
    let queue: id = msg![env; this new];
    State::get(env).main_queue = Some(queue);
    queue
}

+ (id)currentQueue {
    msg![env; this mainQueue]
}

- (())addOperation:(id)operation {
    if env.objc.borrow::<NSOperationQueueHostObject>(this).suspended {
        retain(env, operation);
        env.objc
            .borrow_mut::<NSOperationQueueHostObject>(this)
            .pending
            .push(operation);
    } else {
        run_operation(env, operation);
    }
}

- (())addOperationWithBlock:(MutPtr<BlockLiteral>)block {
    // Blocks passed to this convenience method may live on the guest stack.
    // Until iDared has a general Block_copy implementation, executing before
    // returning is the lifetime-safe subset of the queue contract and matches
    // this class's existing deterministic synchronous operation model.
    log_once!("NSOperationQueue addOperationWithBlock: executes synchronously");
    invoke_block_synchronously(env, block);
}

- (())addOperations:(id)operations waitUntilFinished:(bool)_wait {
    let count: NSUInteger = msg![env; operations count];
    for index in 0..count {
        let operation: id = msg![env; operations objectAtIndex:index];
        let _: () = msg![env; this addOperation:operation];
    }
}

- (())cancelAllOperations {
    let pending = env.objc.borrow::<NSOperationQueueHostObject>(this).pending.clone();
    for operation in pending {
        let _: () = msg![env; operation cancel];
    }
}

- (id)operations {
    let pending = env.objc.borrow::<NSOperationQueueHostObject>(this).pending.clone();
    for &operation in &pending {
        retain(env, operation);
    }
    let operations = ns_array::from_vec(env, pending);
    autorelease(env, operations)
}

- (NSUInteger)operationCount {
    env.objc.borrow::<NSOperationQueueHostObject>(this).pending.len() as NSUInteger
}

- (NSInteger)maxConcurrentOperationCount {
    env.objc.borrow::<NSOperationQueueHostObject>(this).max_concurrent_operation_count
}
- (())setMaxConcurrentOperationCount:(NSInteger)count {
    env.objc.borrow_mut::<NSOperationQueueHostObject>(this).max_concurrent_operation_count = count;
}

- (bool)isSuspended {
    env.objc.borrow::<NSOperationQueueHostObject>(this).suspended
}
- (())setSuspended:(bool)suspended {
    let was_suspended = env.objc.borrow::<NSOperationQueueHostObject>(this).suspended;
    env.objc.borrow_mut::<NSOperationQueueHostObject>(this).suspended = suspended;
    if was_suspended && !suspended {
        let pending = std::mem::take(
            &mut env.objc.borrow_mut::<NSOperationQueueHostObject>(this).pending,
        );
        for operation in pending {
            run_operation(env, operation);
            release(env, operation);
        }
    }
}

- (())waitUntilAllOperationsAreFinished {
    let _: () = msg![env; this setSuspended:false];
}

- (id)name {
    env.objc.borrow::<NSOperationQueueHostObject>(this).name
}
- (())setName:(id)name {
    let copied_name: id = msg![env; name copy];
    let old_name = env.objc.borrow::<NSOperationQueueHostObject>(this).name;
    env.objc.borrow_mut::<NSOperationQueueHostObject>(this).name = copied_name;
    release(env, old_name);
}

- (())dealloc {
    let host_object = env.objc.borrow::<NSOperationQueueHostObject>(this);
    let name = host_object.name;
    let pending = host_object.pending.clone();
    release(env, name);
    for operation in pending {
        release(env, operation);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::msg_class;

    #[test]
    fn invocation_operation_retains_and_invokes_its_invocation() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let target: id = msg_class![env; NSObject new];
        let selector = env
            .objc
            .register_host_selector("description".to_owned(), &mut env.mem);
        let signature: id = msg![env; target methodSignatureForSelector:selector];
        let invocation: id = msg_class![env; NSInvocation invocationWithMethodSignature:signature];
        () = msg![env; invocation setTarget:target];
        () = msg![env; invocation setSelector:selector];
        let operation: id = msg_class![env; NSInvocationOperation alloc];
        let operation: id = msg![env; operation initWithInvocation:invocation];
        let stored: id = msg![env; operation invocation];
        assert_eq!(stored, invocation);
        () = msg![env; operation start];
        assert!(msg![env; operation isFinished]);
        release(env, operation);
        release(env, target);
    }
}
