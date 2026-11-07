/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIDevice`.

use crate::dyld::ConstantExports;
use crate::dyld::HostConstant;
use crate::environment::Environment;
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::{ns_string, NSInteger};
use crate::msg_class;
use crate::objc::{id, msg, nil, objc_classes, ClassExports, NSZonePtr, TrivialHostObject};
use crate::window::{get_battery_status, BatteryState, DeviceFamily, DeviceOrientation};
use std::time::{Duration, Instant};

pub const UIDeviceOrientationDidChangeNotification: &str =
    "UIDeviceOrientationDidChangeNotification";
pub const UIDeviceBatteryStateDidChangeNotification: &str =
    "UIDeviceBatteryStateDidChangeNotification";

pub type UIDeviceOrientation = NSInteger;
#[allow(dead_code)]
pub const UIDeviceOrientationUnknown: UIDeviceOrientation = 0;
pub const UIDeviceOrientationPortrait: UIDeviceOrientation = 1;
pub const UIDeviceOrientationPortraitUpsideDown: UIDeviceOrientation = 2;
pub const UIDeviceOrientationLandscapeLeft: UIDeviceOrientation = 3;
pub const UIDeviceOrientationLandscapeRight: UIDeviceOrientation = 4;
#[allow(dead_code)]
pub const UIDeviceOrientationFaceUp: UIDeviceOrientation = 5;
#[allow(dead_code)]
pub const UIDeviceOrientationFaceDown: UIDeviceOrientation = 6;

pub type UIDeviceBatteryState = NSInteger;
pub const UIDeviceBatteryStateUnknown: UIDeviceBatteryState = 0;
pub const UIDeviceBatteryStateUnplugged: UIDeviceBatteryState = 1;
pub const UIDeviceBatteryStateCharging: UIDeviceBatteryState = 2;
pub const UIDeviceBatteryStateFull: UIDeviceBatteryState = 3;

type UIUserInterfaceIdiom = NSInteger;
#[allow(dead_code)]
const UIUserInterfaceIdiomUnspecified: UIUserInterfaceIdiom = -1;
const UIUserInterfaceIdiomPhone: UIUserInterfaceIdiom = 0;
const UIUserInterfaceIdiomPad: UIUserInterfaceIdiom = 1;

#[derive(Default)]
pub struct State {
    current_device: Option<id>,
    orientation_generation_count: usize,
    physical_orientation: Option<UIDeviceOrientation>,
    orientation_candidate: Option<UIDeviceOrientation>,
    orientation_candidate_since: Option<Instant>,
    orientation_due_by: Option<Instant>,
    battery_monitoring_enabled: bool,
    last_battery_state: Option<UIDeviceBatteryState>,
    battery_due_by: Option<Instant>,
}
impl State {
    pub fn is_generating_device_orientation_notifications(&self) -> bool {
        self.orientation_generation_count > 0
    }
}

const ORIENTATION_SAMPLE_INTERVAL: Duration = Duration::from_millis(50);
const ORIENTATION_STABILITY_INTERVAL: Duration = Duration::from_millis(100);
const ORIENTATION_MINIMUM_GRAVITY: f32 = 0.5;
const BATTERY_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

fn interface_orientation(orientation: DeviceOrientation) -> UIDeviceOrientation {
    match orientation {
        DeviceOrientation::Portrait => UIDeviceOrientationPortrait,
        DeviceOrientation::PortraitUpsideDown => UIDeviceOrientationPortraitUpsideDown,
        DeviceOrientation::LandscapeLeft => UIDeviceOrientationLandscapeLeft,
        DeviceOrientation::LandscapeRight => UIDeviceOrientationLandscapeRight,
    }
}

/// Infer the physical device orientation from UIAcceleration-compatible
/// gravity values. The dominant axis is used only when the reading is strong
/// enough to reject free-fall/noisy samples.
fn orientation_from_acceleration(x: f32, y: f32, z: f32) -> Option<UIDeviceOrientation> {
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return None;
    }
    let (abs_x, abs_y, abs_z) = (x.abs(), y.abs(), z.abs());
    let strongest = abs_x.max(abs_y).max(abs_z);
    if strongest < ORIENTATION_MINIMUM_GRAVITY {
        return None;
    }
    if abs_z >= abs_x && abs_z >= abs_y {
        Some(if z < 0.0 {
            UIDeviceOrientationFaceUp
        } else {
            UIDeviceOrientationFaceDown
        })
    } else if abs_x >= abs_y {
        Some(if x < 0.0 {
            UIDeviceOrientationLandscapeLeft
        } else {
            UIDeviceOrientationLandscapeRight
        })
    } else {
        Some(if y < 0.0 {
            UIDeviceOrientationPortrait
        } else {
            UIDeviceOrientationPortraitUpsideDown
        })
    }
}

pub const CONSTANTS: ConstantExports = &[
    (
        "_UIDeviceOrientationDidChangeNotification",
        HostConstant::NSString(UIDeviceOrientationDidChangeNotification),
    ),
    (
        "_UIDeviceBatteryStateDidChangeNotification",
        HostConstant::NSString(UIDeviceBatteryStateDidChangeNotification),
    ),
];

fn host_battery_state() -> UIDeviceBatteryState {
    match get_battery_status().1 {
        BatteryState::Unknown => UIDeviceBatteryStateUnknown,
        BatteryState::OnBattery => UIDeviceBatteryStateUnplugged,
        BatteryState::NoBattery | BatteryState::Charging => UIDeviceBatteryStateCharging,
        BatteryState::Full => UIDeviceBatteryStateFull,
    }
}

fn record_battery_state(
    env: &mut Environment,
    current: UIDeviceBatteryState,
) -> UIDeviceBatteryState {
    let (enabled, changed) = {
        let state = &mut env.framework_state.uikit.ui_device;
        let changed = state.last_battery_state.is_some_and(|old| old != current);
        state.last_battery_state = Some(current);
        (state.battery_monitoring_enabled, changed)
    };
    if enabled && changed {
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        let name = get_static_str(env, UIDeviceBatteryStateDidChangeNotification);
        let device = env
            .framework_state
            .uikit
            .ui_device
            .current_device
            .unwrap_or(nil);
        () = msg![env; center postNotificationName:name object:device];
    }
    current
}

fn observe_battery_state(env: &mut Environment) -> UIDeviceBatteryState {
    record_battery_state(env, host_battery_state())
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIDevice: NSObject

+ (id)currentDevice {
    if let Some(device) = env.framework_state.uikit.ui_device.current_device {
        device
    } else {
        let new = msg_class![env; _touchHLE_UIDevice_Static alloc];
        env.framework_state.uikit.ui_device.current_device = Some(new);
        new
    }
}

- (())beginGeneratingDeviceOrientationNotifications {
    log_dbg!("[UIDevice beginGeneratingDeviceOrientationNotifications]");
    let state = &mut env.framework_state.uikit.ui_device;
    state.orientation_generation_count = state.orientation_generation_count.saturating_add(1);
    if state.orientation_generation_count == 1 {
        state.orientation_due_by = Some(Instant::now());
    }
}
- (())endGeneratingDeviceOrientationNotifications {
    log_dbg!("[UIDevice endGeneratingDeviceOrientationNotifications]");
    let state = &mut env.framework_state.uikit.ui_device;
    state.orientation_generation_count = state.orientation_generation_count.saturating_sub(1);
    if state.orientation_generation_count == 0 {
        state.orientation_candidate = None;
        state.orientation_candidate_since = None;
        state.orientation_due_by = None;
    }
}
- (bool)isGeneratingDeviceOrientationNotifications {
    let res = env.framework_state.uikit.ui_device.is_generating_device_orientation_notifications();
    log_dbg!("[UIDevice isGeneratingDeviceOrientationNotifications] -> {}", res);
    res
}

- (id)model {
    let model = match env.options.device_family {
        Some(DeviceFamily::iPad) => "iPad",
        _ => "iPhone",
    };
    ns_string::get_static_str(env, model)
}
- (id)localizedModel {
    // TODO: localization
    msg![env; this model]
}

- (id)name {
    msg![env; this model]
}

- (id)systemName {
    ns_string::get_static_str(env, "iPhone OS")
}

// NSString
- (id)systemVersion {
    // The original iPad came with iPhone OS 3.2.
    let version = match env.options.device_family {
        Some(DeviceFamily::iPad) => "3.2",
        _ => "2.0",
    };
    ns_string::get_static_str(env, version)
}

- (id)uniqueIdentifier {
    // Aspen Simulator returns (null) here
    // A device unique identifier must be 40 characters long
    ns_string::get_static_str(env, "touchHLEdevice..........................")
}

- (bool)isMultitaskingSupported {
    false
}

- (UIDeviceOrientation)orientation {
    let fallback = interface_orientation(env.window().current_rotation());
    env.framework_state
        .uikit
        .ui_device
        .physical_orientation
        .unwrap_or(fallback)
}
- (())setOrientation:(UIDeviceOrientation)orientation {
    let prev_orientation = env.window().current_rotation();
    env.on_parent_stack_in_coroutine(|window, _| {window.rotate_device(match orientation {
        UIDeviceOrientationPortrait => DeviceOrientation::Portrait,
        UIDeviceOrientationPortraitUpsideDown => DeviceOrientation::PortraitUpsideDown,
        UIDeviceOrientationLandscapeLeft => DeviceOrientation::LandscapeLeft,
        UIDeviceOrientationLandscapeRight => DeviceOrientation::LandscapeRight,
        _ => unimplemented!("Orientation {} not handled yet", orientation),
    })});
    if prev_orientation != env.window().current_rotation() {
        let state = &mut env.framework_state.uikit.ui_device;
        state.physical_orientation = Some(orientation);
        state.orientation_candidate = None;
        state.orientation_candidate_since = None;
        generate_device_orientation_notification(env);
    }
}

- (bool)isBatteryMonitoringEnabled {
    env.framework_state.uikit.ui_device.battery_monitoring_enabled
}
- (())setBatteryMonitoringEnabled:(bool)enabled {
    let current = host_battery_state();
    let state = &mut env.framework_state.uikit.ui_device;
    state.battery_monitoring_enabled = enabled;
    state.last_battery_state = enabled.then_some(current);
    state.battery_due_by = enabled.then_some(Instant::now() + BATTERY_SAMPLE_INTERVAL);
}
- (f32)batteryLevel {
    let pct = get_battery_status().0;
    if pct < 0 {
        log_dbg!("batteryLevel percentage could not be determined, returning 100% for compatibility");
        return 1.0
    }
    pct as f32 / 100.0 // narrow down to 0.0 - 1.0
}
- (UIDeviceBatteryState)batteryState {
    observe_battery_state(env)
}

- (UIUserInterfaceIdiom)userInterfaceIdiom {
    match env.window().device_family() {
        DeviceFamily::iPhone => UIUserInterfaceIdiomPhone,
        DeviceFamily::iPad => UIUserInterfaceIdiomPad,
    }
}

@end

// Private static implementation of UIDevice, used for the current device
@implementation _touchHLE_UIDevice_Static: UIDevice

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_static_object(
        this,
        Box::new(TrivialHostObject),
        &mut env.mem
    )
}

- (id) retain { this }
- (()) release {}
- (id) autorelease { this }

@end

};

/// For use by `NSRunLoop`: sample the real/simulated gravity source while an
/// app has requested physical orientation notifications. Interface orientation
/// remains independent; only UIDevice's physical orientation changes here.
pub(super) fn handle_device_orientation(env: &mut Environment) -> Option<Instant> {
    let now = Instant::now();
    let battery_due = {
        let state = &mut env.framework_state.uikit.ui_device;
        if !state.battery_monitoring_enabled {
            state.battery_due_by = None;
            None
        } else if state.battery_due_by.is_none_or(|due_by| due_by <= now) {
            state.battery_due_by = Some(now + BATTERY_SAMPLE_INTERVAL);
            Some(true)
        } else {
            Some(false)
        }
    };
    if battery_due == Some(true) {
        observe_battery_state(env);
    }
    let battery_due_by = env.framework_state.uikit.ui_device.battery_due_by;
    {
        let state = &mut env.framework_state.uikit.ui_device;
        if !state.is_generating_device_orientation_notifications() {
            return battery_due_by;
        }
        if state.orientation_due_by.is_some_and(|due_by| due_by > now) {
            return match (state.orientation_due_by, battery_due_by) {
                (Some(orientation), Some(battery)) => Some(orientation.min(battery)),
                (orientation, None) => orientation,
                (None, battery) => battery,
            };
        }
        state.orientation_due_by = Some(now + ORIENTATION_SAMPLE_INTERVAL);
    }

    let (x, y, z) = env.window().get_acceleration(&env.options);
    let observed = orientation_from_acceleration(x, y, z);
    let fallback = interface_orientation(env.window().current_rotation());
    let mut changed_to = None;

    if let Some(observed) = observed {
        let state = &mut env.framework_state.uikit.ui_device;
        match state.physical_orientation {
            None => {
                if observed != fallback {
                    changed_to = Some(observed);
                }
                state.physical_orientation = Some(observed);
                state.orientation_candidate = None;
                state.orientation_candidate_since = None;
            }
            Some(current) if current == observed => {
                state.orientation_candidate = None;
                state.orientation_candidate_since = None;
            }
            Some(_) if state.orientation_candidate == Some(observed) => {
                let stable = state.orientation_candidate_since.is_some_and(|since| {
                    now.duration_since(since) >= ORIENTATION_STABILITY_INTERVAL
                });
                if stable {
                    state.physical_orientation = Some(observed);
                    state.orientation_candidate = None;
                    state.orientation_candidate_since = None;
                    changed_to = Some(observed);
                }
            }
            Some(_) => {
                state.orientation_candidate = Some(observed);
                state.orientation_candidate_since = Some(now);
            }
        }
    }

    if let Some(orientation) = changed_to {
        log!("UIDevice physical orientation changed to {}.", orientation);
        generate_device_orientation_notification(env);
    }
    match (
        env.framework_state.uikit.ui_device.orientation_due_by,
        battery_due_by,
    ) {
        (Some(orientation), Some(battery)) => Some(orientation.min(battery)),
        (orientation, None) => orientation,
        (None, battery) => battery,
    }
}

pub fn generate_device_orientation_notification(env: &mut Environment) {
    let center: id = msg_class![env; NSNotificationCenter defaultCenter];
    let name = get_static_str(env, UIDeviceOrientationDidChangeNotification);
    let device: id = msg_class![env; UIDevice currentDevice];
    let _: () = msg![env; center postNotificationName:name object:device];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_orientation_uses_dominant_gravity_axis() {
        assert_eq!(
            orientation_from_acceleration(0.0, -1.0, 0.0),
            Some(UIDeviceOrientationPortrait)
        );
        assert_eq!(
            orientation_from_acceleration(0.0, 1.0, 0.0),
            Some(UIDeviceOrientationPortraitUpsideDown)
        );
        assert_eq!(
            orientation_from_acceleration(-1.0, 0.0, 0.0),
            Some(UIDeviceOrientationLandscapeLeft)
        );
        assert_eq!(
            orientation_from_acceleration(1.0, 0.0, 0.0),
            Some(UIDeviceOrientationLandscapeRight)
        );
        assert_eq!(
            orientation_from_acceleration(0.0, 0.0, -1.0),
            Some(UIDeviceOrientationFaceUp)
        );
        assert_eq!(
            orientation_from_acceleration(0.0, 0.0, 1.0),
            Some(UIDeviceOrientationFaceDown)
        );
    }

    #[test]
    fn physical_orientation_rejects_weak_or_invalid_samples() {
        assert_eq!(orientation_from_acceleration(0.1, 0.2, 0.3), None);
        assert_eq!(orientation_from_acceleration(f32::NAN, 1.0, 0.0), None);
    }

    #[test]
    fn dev_245_battery_notification_posts_only_on_monitored_state_change() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let device: id = msg_class![env; UIDevice currentDevice];
        let notifications: id = msg_class![env; NSMutableArray new];
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        let selector = env.objc.lookup_selector("addObject:").unwrap();
        let name = get_static_str(env, UIDeviceBatteryStateDidChangeNotification);
        () = msg![env; center addObserver:notifications selector:selector name:name object:device];
        {
            let state = &mut env.framework_state.uikit.ui_device;
            state.battery_monitoring_enabled = true;
            state.last_battery_state = Some(UIDeviceBatteryStateUnplugged);
        }
        record_battery_state(env, UIDeviceBatteryStateCharging);
        record_battery_state(env, UIDeviceBatteryStateCharging);
        let count: u32 = msg![env; notifications count];
        assert_eq!(count, 1);
        env.framework_state
            .uikit
            .ui_device
            .battery_monitoring_enabled = false;
        record_battery_state(env, UIDeviceBatteryStateFull);
        let count: u32 = msg![env; notifications count];
        assert_eq!(count, 1);
        () = msg![env; center removeObserver:notifications];
        crate::objc::release(env, notifications);
        crate::objc::release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
