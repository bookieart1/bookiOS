/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CADisplayLink`

use crate::frameworks::core_animation::CACurrentMediaTime;
use crate::frameworks::foundation::ns_run_loop::NSRunLoopMode;
use crate::frameworks::foundation::ns_timer::set_time_interval;
use crate::frameworks::foundation::{NSInteger, NSTimeInterval};
use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, nil, objc_classes, release, retain, ClassExports,
    HostObject, NSZonePtr, SEL,
};

#[derive(Default)]
struct CADisplayLinkHostObject {
    target: id,
    selector: Option<SEL>,
    /// Weak reference. The timer retains the display link (as its target),
    /// so the timer necessarily outlives the display link. After `invalidate`,
    /// this pointer must not be used.
    ns_timer: id,
    paused: bool,
    timestamp: NSTimeInterval,
    frame_interval: NSInteger,
}
impl HostObject for CADisplayLinkHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation CADisplayLink: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::new(CADisplayLinkHostObject::default()), &mut env.mem)
}

+ (id)displayLinkWithTarget:(id)target selector:(SEL)sel {
    let display_link: id = msg![env; this new];
    // Because timer will pass itself as a second arg in ns_timer:handle_timer,
    // we need to use a re-direction: first fire the timer on the display link,
    // then call the original selector, passing the link as a second argument!
    let redirect_sel: SEL = env.objc.lookup_selector("_touchHLE_displayLinkTimerDidFire:").unwrap();
    let ns_timer = msg_class![env; NSTimer timerWithTimeInterval:(1.0/60.0)
                     target:display_link
                   selector:redirect_sel
                   userInfo:nil
                    repeats:true];
    retain(env, target);
    let host_object = env.objc.borrow_mut::<CADisplayLinkHostObject>(display_link);
    host_object.target = target;
    host_object.selector = Some(sel);
    host_object.ns_timer = ns_timer;
    host_object.frame_interval = 1;
    let diagnostic_detail = format!("create target,selector={},timer,display-link", sel.as_str(&env.mem));
    env.diagnostic_campaign_event(
        "display-link",
        &diagnostic_detail,
        [target.to_bits(), 0, ns_timer.to_bits(), display_link.to_bits(), 0, 0, 0, 0],
    );
    log_dbg!("[CADisplayLink displayLinkWithTarget:{:?} selector:{}] => {:?}", target, sel.as_str(&env.mem), display_link);
    autorelease(env, display_link)
}

- (bool)isPaused {
    env.objc.borrow::<CADisplayLinkHostObject>(this).paused
}
- (())setPaused:(bool)paused {
    env.objc.borrow_mut::<CADisplayLinkHostObject>(this).paused = paused;
}

- (NSTimeInterval)timestamp {
    env.objc.borrow::<CADisplayLinkHostObject>(this).timestamp
}

- (NSTimeInterval)duration {
    1.0 / 60.0
}

- (())setFrameInterval:(NSInteger)frameInterval {
    log_dbg!("[(CADisplayLink*){:?} setFrameInterval:{}]", this, frameInterval);
    assert!(frameInterval >= 1);
    let interval = frameInterval as f64 / 60.0;
    let ns_timer = env.objc.borrow::<CADisplayLinkHostObject>(this).ns_timer;
    set_time_interval(env, ns_timer, interval);
    env.objc.borrow_mut::<CADisplayLinkHostObject>(this).frame_interval = frameInterval;
}

- (NSInteger)frameInterval {
    env.objc.borrow::<CADisplayLinkHostObject>(this).frame_interval
}

- (())addToRunLoop:(id)run_loop forMode:(NSRunLoopMode)mode {
    log_dbg!("[(CADisplayLink*){:?} addToRunLoop:{:?} forMode:{:?}]", this, run_loop, mode);
    let ns_timer = env.objc.borrow::<CADisplayLinkHostObject>(this).ns_timer;
    env.diagnostic_campaign_event(
        "display-link",
        "schedule display-link,run-loop,mode,timer",
        [this.to_bits(), run_loop.to_bits(), mode.to_bits(), ns_timer.to_bits(), 0, 0, 0, 0],
    );
    () = msg![env; run_loop addTimer:ns_timer forMode:mode];
}

- (())invalidate {
    log_dbg!("[(CADisplayLink*){:?} invalidate]", this);
    let ns_timer = env.objc.borrow::<CADisplayLinkHostObject>(this).ns_timer;
    () = msg![env; ns_timer invalidate];
}

- (())dealloc {
    let &CADisplayLinkHostObject { target, .. } = env.objc.borrow(this);
    release(env, target);
    env.objc.dealloc_object(this, &mut env.mem);
}

- (())_touchHLE_displayLinkTimerDidFire:(id)timer { // NSTimer *
    let &CADisplayLinkHostObject {
        target,
        selector,
        ns_timer,
        paused,
        ..
    } = env.objc.borrow::<CADisplayLinkHostObject>(this);
    assert_eq!(ns_timer, timer);
    if paused {
        env.diagnostic_campaign_event(
            "display-link",
            "timer-fired-while-paused",
            [this.to_bits(), target.to_bits(), timer.to_bits(), 0, 0, 0, 0, 0],
        );
        // This could be improved, as we're still running the timer,
        // but just not passing the actual call.
        return;
    }
    let timestamp = CACurrentMediaTime(env);
    let diagnostic_detail = format!("timer-fire target,selector={},timer,display-link", selector.unwrap().as_str(&env.mem));
    env.diagnostic_campaign_event(
        "display-link",
        &diagnostic_detail,
        [target.to_bits(), 0, timer.to_bits(), this.to_bits(), timestamp.to_bits() as u32, (timestamp.to_bits() >> 32) as u32, 0, 0],
    );
    env.objc.borrow_mut::<CADisplayLinkHostObject>(this).timestamp = timestamp;
    // Signature is `- (void) selector:(CADisplayLink *)sender;`
    () = msg_send(env, (target, selector.unwrap(), this));
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn development_256_display_link_exposes_default_and_updated_frame_interval() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let target: id = msg_class![env; NSObject new];
        let selector = env.objc.lookup_selector("description").unwrap();
        let link: id = msg_class![env; CADisplayLink displayLinkWithTarget:target selector:selector];
        let initial: NSInteger = msg![env; link frameInterval];
        assert_eq!(initial, 1);
        () = msg![env; link setFrameInterval:3i32];
        let updated: NSInteger = msg![env; link frameInterval];
        assert_eq!(updated, 3);
        release(env, pool);
        release(env, target);
    }
}
