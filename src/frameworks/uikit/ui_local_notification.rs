/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UILocalNotification`.

use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::{ns_string::get_static_str, NSInteger};
use crate::objc::{id, msg, msg_super, objc_classes, release, ClassExports, HostObject, NSZonePtr};

#[derive(Default)]
struct UILocalNotificationHostObject {
    fire_date: id,
    time_zone: id,
    alert_body: id,
    alert_action: id,
    alert_launch_image: id,
    sound_name: id,
    application_icon_badge_number: NSInteger,
}
impl HostObject for UILocalNotificationHostObject {}

fn set_copied(
    env: &mut crate::Environment,
    this: id,
    field: fn(&mut UILocalNotificationHostObject) -> &mut id,
    value: id,
) {
    let value: id = msg![env; value copy];
    let old = std::mem::replace(field(env.objc.borrow_mut(this)), value);
    release(env, old);
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UILocalNotification: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UILocalNotificationHostObject>::default(), &mut env.mem)
}

- (())dealloc {
    let host = std::mem::take(env.objc.borrow_mut::<UILocalNotificationHostObject>(this));
    for value in [host.fire_date, host.time_zone, host.alert_body, host.alert_action,
                  host.alert_launch_image, host.sound_name] {
        release(env, value);
    }
    msg_super![env; this dealloc]
}

- (())setFireDate:(id)date { // NSDate *
    set_copied(env, this, |host| &mut host.fire_date, date);
}
- (id)fireDate { env.objc.borrow::<UILocalNotificationHostObject>(this).fire_date }
- (())setTimeZone:(id)time_zone { // NSTimeZone *
    set_copied(env, this, |host| &mut host.time_zone, time_zone);
}
- (id)timeZone { env.objc.borrow::<UILocalNotificationHostObject>(this).time_zone }
- (())setAlertBody:(id)body { // NSString *
    set_copied(env, this, |host| &mut host.alert_body, body);
}
- (id)alertBody { env.objc.borrow::<UILocalNotificationHostObject>(this).alert_body }
- (())setAlertAction:(id)action { // NSString *
    set_copied(env, this, |host| &mut host.alert_action, action);
}
- (id)alertAction { env.objc.borrow::<UILocalNotificationHostObject>(this).alert_action }
- (())setAlertLaunchImage:(id)image { // NSString *
    set_copied(env, this, |host| &mut host.alert_launch_image, image);
}
- (id)alertLaunchImage { env.objc.borrow::<UILocalNotificationHostObject>(this).alert_launch_image }
- (())setSoundName:(id)name { // NSString *
    set_copied(env, this, |host| &mut host.sound_name, name);
}
- (id)soundName { env.objc.borrow::<UILocalNotificationHostObject>(this).sound_name }
- (())setApplicationIconBadgeNumber:(NSInteger)badge_number {
    env.objc.borrow_mut::<UILocalNotificationHostObject>(this).application_icon_badge_number = badge_number;
}
- (NSInteger)applicationIconBadgeNumber {
    env.objc.borrow::<UILocalNotificationHostObject>(this).application_icon_badge_number
}

@end

};

pub const CONSTANTS: ConstantExports = &[(
    "_UILocalNotificationDefaultSoundName",
    HostConstant::Custom(|env| get_static_str(env, "default").cast().cast_const()),
)];
