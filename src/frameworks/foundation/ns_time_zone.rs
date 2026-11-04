/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSTimeZone`.

use crate::frameworks::foundation::{ns_string, NSInteger, NSUInteger};
use crate::objc::{autorelease, id, nil, release, retain, ClassExports, HostObject, NSZonePtr};
use crate::{msg, objc_classes};

#[derive(Default)]
pub struct State {
    system_time_zone: Option<id>,
}

struct NSTimeZoneHostObject {
    // NSString*
    time_zone: id,
}
impl HostObject for NSTimeZoneHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSTimeZone: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSTimeZoneHostObject {
        time_zone: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)timeZoneWithName:(id)tz_name {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithName:tz_name];
    autorelease(env, new)
}

+ (id)timeZoneWithAbbreviation:(id)abbreviation {
    if abbreviation == nil {
        return nil;
    }
    // touchHLE currently models all time zones as GMT, but returning a stable
    // zone object here matches Foundation's object-level contract and lets
    // callers proceed without depending on the host device's time zone data.
    msg![env; this timeZoneWithName:abbreviation]
}

+ (id)localTimeZone {
    // According to docs, `localTimeZone` is not cached in contrast to
    // `systemTimeZone`
    let gmt_tz_name: id = ns_string::get_static_str(env, "GMT");
    msg![env; this timeZoneWithName:gmt_tz_name]
}

+ (id)systemTimeZone {
    if let Some(system_time_zone) = env.framework_state.foundation.ns_time_zone.system_time_zone {
        system_time_zone
    } else {
        let new: id = msg![env; this alloc];
        let gmt_tz_name: id = ns_string::get_static_str(env, "GMT");
        let new: id = msg![env; new initWithName:gmt_tz_name];
        env.framework_state.foundation.ns_time_zone.system_time_zone = Some(new);
        new
    }
}

+ (id)defaultTimeZone {
    // TODO: implement setting a default time zone
    msg![env; this systemTimeZone]
}

- (())dealloc {
    let tz_name = env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone;
    release(env, tz_name);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithName:(id)tz_name { // NSString *
    assert_ne!(tz_name, nil);
    retain(env, tz_name);
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone = tz_name;
    this
}

- (id)name {
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone
}

- (id)abbreviation {
    // TODO: support zone abbreviations
    ns_string::get_static_str(env, "GMT")
}

- (id)localizedName:(NSUInteger)style locale:(id)_locale {
    // touchHLE currently models every NSTimeZone as GMT. All six historical
    // NSTimeZoneNameStyle values therefore have the same correct display name
    // in this deliberately limited model. Unknown styles are invalid.
    if style <= 5 {
        ns_string::get_static_str(env, "GMT")
    } else {
        nil
    }
}

- (NSInteger)secondsFromGMT {
    // TODO: respect timezone
    0
}

- (bool)isDaylightSavingTime {
    // The currently modelled zones are fixed-offset GMT zones and therefore
    // never enter daylight-saving time.
    false
}

- (bool)isDaylightSavingTimeForDate:(id)_date {
    false
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::msg_class;
    use crate::Environment;

    #[test]
    fn localized_name_supports_all_historical_styles() {
        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let time_zone: id = msg_class![env; NSTimeZone systemTimeZone];

        for style in 0u32..=5 {
            let name: id = msg![env; time_zone localizedName:style locale:nil];
            assert_eq!(ns_string::to_rust_string(env, name), "GMT");
        }

        let invalid: id = msg![env; time_zone localizedName:6u32 locale:nil];
        assert_eq!(invalid, nil);
        assert!(!msg![env; time_zone isDaylightSavingTime]);
    }
}
