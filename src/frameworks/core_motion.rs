/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The Core Motion framework.

use crate::abi::impl_GuestRet_for_large_struct;
use crate::dyld::HostDylib;
use crate::frameworks::foundation::NSTimeInterval;
use crate::frameworks::uikit::ui_accelerometer::{
    sample_acceleration, DEFAULT_UPDATE_INTERVAL,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::{msg, msg_class, release};

    #[test]
    fn device_motion_interval_is_stable_without_claiming_motion_availability() {
        let mut env = crate::Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let manager: id = msg_class![env; CMMotionManager new];
        () = msg![env; manager setDeviceMotionUpdateInterval:0.25f64];
        let interval: NSTimeInterval = msg![env; manager deviceMotionUpdateInterval];
        assert_eq!(interval, 0.25);
        let available: bool = msg![env; manager isDeviceMotionAvailable];
        assert!(!available);
        () = msg![env; manager startDeviceMotionUpdates];
        let active: bool = msg![env; manager isDeviceMotionActive];
        let sample: id = msg![env; manager deviceMotion];
        assert!(!active);
        assert_eq!(sample, nil);
        () = msg![env; manager stopDeviceMotionUpdates];
        release(env, manager);
    }
}
use crate::mem::SafeRead;
use crate::objc::{
    id, impl_HostObject_with_superclass, nil, objc_classes, release, ClassExports, HostObject,
    NSZonePtr,
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C, packed)]
pub struct CMAcceleration {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
unsafe impl SafeRead for CMAcceleration {}
impl_GuestRet_for_large_struct!(CMAcceleration);

#[derive(Default)]
struct CMLogItemHostObject {
    timestamp: NSTimeInterval,
}
impl HostObject for CMLogItemHostObject {}

#[derive(Default)]
struct CMAccelerometerDataHostObject {
    superclass: CMLogItemHostObject,
    acceleration: CMAcceleration,
}
impl_HostObject_with_superclass!(CMAccelerometerDataHostObject);

struct CMMotionManagerHostObject {
    accelerometer_active: bool,
    accelerometer_update_interval: NSTimeInterval,
    /// Strong reference to the latest polling sample while active.
    accelerometer_data: id,
    device_motion_update_interval: NSTimeInterval,
    device_motion_active: bool,
}
impl Default for CMMotionManagerHostObject {
    fn default() -> Self {
        Self {
            accelerometer_active: false,
            accelerometer_update_interval: DEFAULT_UPDATE_INTERVAL,
            accelerometer_data: nil,
            device_motion_update_interval: DEFAULT_UPDATE_INTERVAL,
            device_motion_active: false,
        }
    }
}
impl HostObject for CMMotionManagerHostObject {}

pub const DYLIB: HostDylib = HostDylib {
    path: "/System/Library/Frameworks/CoreMotion.framework/CoreMotion",
    aliases: &[],
    class_exports: &[CLASSES],
    constant_exports: &[],
    function_exports: &[],
};

const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation CMLogItem: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<CMLogItemHostObject>::default(), &mut env.mem)
}

- (NSTimeInterval)timestamp {
    env.objc.borrow::<CMLogItemHostObject>(this).timestamp
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

@implementation CMAccelerometerData: CMLogItem

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::<CMAccelerometerDataHostObject>::default(),
        &mut env.mem,
    )
}

- (CMAcceleration)acceleration {
    env.objc.borrow::<CMAccelerometerDataHostObject>(this).acceleration
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

@implementation CMMotionManager: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::<CMMotionManagerHostObject>::default(),
        &mut env.mem,
    )
}

- (bool)isGyroAvailable {
    // It could make sense to implement gyroscope support at least for Android.
    log!("TODO: [(CMMotionManager *){:?} isGyroAvailable] -> false", this);
    false
}
- (bool)isDeviceMotionAvailable {
    log!("TODO: [(CMMotionManager *){:?} isDeviceMotionAvailable] -> false", this);
    // According to docs, this is functionally equivalent to `isGyroAvailable`
    // method. (All devices have accelerometer, but only some do have gyro).
    false
}

- (NSTimeInterval)deviceMotionUpdateInterval {
    env.objc
        .borrow::<CMMotionManagerHostObject>(this)
        .device_motion_update_interval
}

- (())setDeviceMotionUpdateInterval:(NSTimeInterval)interval {
    let interval = if interval <= 0.0 {
        DEFAULT_UPDATE_INTERVAL
    } else {
        interval.max(DEFAULT_UPDATE_INTERVAL)
    };
    env.objc
        .borrow_mut::<CMMotionManagerHostObject>(this)
        .device_motion_update_interval = interval;
}

- (())startDeviceMotionUpdates {
    // Device motion requires a gyroscope. This emulated device reports that
    // capability as unavailable, so starting cannot produce samples.
    env.objc
        .borrow_mut::<CMMotionManagerHostObject>(this)
        .device_motion_active = false;
}

- (())stopDeviceMotionUpdates {
    env.objc
        .borrow_mut::<CMMotionManagerHostObject>(this)
        .device_motion_active = false;
}

- (bool)isDeviceMotionActive {
    env.objc
        .borrow::<CMMotionManagerHostObject>(this)
        .device_motion_active
}

- (id)deviceMotion {
    nil
}
- (bool)isAccelerometerAvailable {
    // According to https://developer.apple.com/documentation/coremotion/getting-raw-accelerometer-events?language=objc,
    // every iOS device has an accelerometer, but on real hardware this method
    // can still return false if the device isn't ready to produce data yet.
    // Here we always return true since we don't model that readiness state.
    true
}

- (())startAccelerometerUpdates {
    let already_active = env
        .objc
        .borrow::<CMMotionManagerHostObject>(this)
        .accelerometer_active;
    if already_active {
        return;
    }

    if let Some(window) = env.window.as_ref() {
        window.print_accelerometer_notice(&env.options);
    }
    let data_class = env
        .objc
        .get_known_class("CMAccelerometerData", &mut env.mem);
    let data = env.objc.alloc_object(
        data_class,
        Box::<CMAccelerometerDataHostObject>::default(),
        &mut env.mem,
    );
    let host = env.objc.borrow_mut::<CMMotionManagerHostObject>(this);
    host.accelerometer_active = true;
    host.accelerometer_data = data;
}

- (())stopAccelerometerUpdates {
    let data = {
        let host = env.objc.borrow_mut::<CMMotionManagerHostObject>(this);
        host.accelerometer_active = false;
        std::mem::replace(&mut host.accelerometer_data, nil)
    };
    release(env, data);
}

- (bool)isAccelerometerActive {
    env.objc
        .borrow::<CMMotionManagerHostObject>(this)
        .accelerometer_active
}

- (NSTimeInterval)accelerometerUpdateInterval {
    env.objc
        .borrow::<CMMotionManagerHostObject>(this)
        .accelerometer_update_interval
}

- (())setAccelerometerUpdateInterval:(NSTimeInterval)interval {
    // A zero interval is documented as requesting the platform default. Clamp
    // invalid/high-frequency requests to the same 60 Hz floor as
    // UIAccelerometer.
    let interval = if interval <= 0.0 {
        DEFAULT_UPDATE_INTERVAL
    } else {
        interval.max(DEFAULT_UPDATE_INTERVAL)
    };
    env.objc
        .borrow_mut::<CMMotionManagerHostObject>(this)
        .accelerometer_update_interval = interval;
}

- (id)accelerometerData {
    let data = {
        let host = env.objc.borrow::<CMMotionManagerHostObject>(this);
        if !host.accelerometer_active {
            return nil;
        }
        host.accelerometer_data
    };
    let (x, y, z, timestamp) = sample_acceleration(env);
    *env.objc.borrow_mut::<CMAccelerometerDataHostObject>(data) =
        CMAccelerometerDataHostObject {
            superclass: CMLogItemHostObject { timestamp },
            acceleration: CMAcceleration { x, y, z },
        };
    data
}

- (())dealloc {
    let data = env
        .objc
        .borrow::<CMMotionManagerHostObject>(this)
        .accelerometer_data;
    release(env, data);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

};
