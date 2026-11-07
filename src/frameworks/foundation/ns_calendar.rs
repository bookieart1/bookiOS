/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSCalendar` and `NSDateComponents`.
//!
//! This currently models the Gregorian calendar in GMT. That is sufficient
//! for the common early-iOS uses of extracting date fields, constructing a
//! date, and adding date components.

use super::{ns_string, NSInteger, NSTimeInterval, NSUInteger};
use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::core_foundation::time::{
    absolute_time_from_gregorian, add_gregorian_units, CFAbsoluteTimeGetGregorianDate,
    CFGregorianUnits,
};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};

pub const NSEraCalendarUnit: NSUInteger = 1 << 1;
pub const NSYearCalendarUnit: NSUInteger = 1 << 2;
pub const NSMonthCalendarUnit: NSUInteger = 1 << 3;
pub const NSDayCalendarUnit: NSUInteger = 1 << 4;
pub const NSHourCalendarUnit: NSUInteger = 1 << 5;
pub const NSMinuteCalendarUnit: NSUInteger = 1 << 6;
pub const NSSecondCalendarUnit: NSUInteger = 1 << 7;
pub const NSWeekCalendarUnit: NSUInteger = 1 << 8;
pub const NSWeekdayCalendarUnit: NSUInteger = 1 << 9;
pub const NSWeekdayOrdinalCalendarUnit: NSUInteger = 1 << 10;
pub const NSQuarterCalendarUnit: NSUInteger = 1 << 11;
pub const NSWeekOfMonthCalendarUnit: NSUInteger = 1 << 12;
pub const NSWeekOfYearCalendarUnit: NSUInteger = 1 << 13;
pub const NSYearForWeekOfYearCalendarUnit: NSUInteger = 1 << 14;
pub const NSCalendarCalendarUnit: NSUInteger = 1 << 20;
pub const NSTimeZoneCalendarUnit: NSUInteger = 1 << 21;

const NS_UNDEFINED_DATE_COMPONENT: NSInteger = NSInteger::MAX;
const NS_GREGORIAN_CALENDAR: &str = "gregorian";
const NS_BUDDHIST_CALENDAR: &str = "buddhist";
const NS_CHINESE_CALENDAR: &str = "chinese";
const NS_HEBREW_CALENDAR: &str = "hebrew";
const NS_ISLAMIC_CALENDAR: &str = "islamic";
const NS_ISLAMIC_CIVIL_CALENDAR: &str = "islamic-civil";
const NS_JAPANESE_CALENDAR: &str = "japanese";
const NS_PERSIAN_CALENDAR: &str = "persian";
const NS_REPUBLIC_OF_CHINA_CALENDAR: &str = "republic-of-china";
const NS_INDIAN_CALENDAR: &str = "indian";
const NS_ISO_8601_CALENDAR: &str = "iso8601";

// These are identifier strings, not claims that NSCalendar implements each
// calendar's arithmetic. Several Unity/Mono-era games import the complete
// historical identifier table even though they only compare or pass the
// identifiers through. Leaving the symbols unresolved turns an ordinary data
// lookup into a null-page fault before the app can make that distinction.
pub const CONSTANTS: ConstantExports = &[
    (
        "_NSGregorianCalendar",
        HostConstant::NSString(NS_GREGORIAN_CALENDAR),
    ),
    (
        "_NSBuddhistCalendar",
        HostConstant::NSString(NS_BUDDHIST_CALENDAR),
    ),
    (
        "_NSChineseCalendar",
        HostConstant::NSString(NS_CHINESE_CALENDAR),
    ),
    (
        "_NSHebrewCalendar",
        HostConstant::NSString(NS_HEBREW_CALENDAR),
    ),
    (
        "_NSIslamicCalendar",
        HostConstant::NSString(NS_ISLAMIC_CALENDAR),
    ),
    (
        "_NSIslamicCivilCalendar",
        HostConstant::NSString(NS_ISLAMIC_CIVIL_CALENDAR),
    ),
    (
        "_NSJapaneseCalendar",
        HostConstant::NSString(NS_JAPANESE_CALENDAR),
    ),
    (
        "_NSPersianCalendar",
        HostConstant::NSString(NS_PERSIAN_CALENDAR),
    ),
    (
        "_NSRepublicOfChinaCalendar",
        HostConstant::NSString(NS_REPUBLIC_OF_CHINA_CALENDAR),
    ),
    (
        "_NSIndianCalendar",
        HostConstant::NSString(NS_INDIAN_CALENDAR),
    ),
    (
        "_NSISO8601Calendar",
        HostConstant::NSString(NS_ISO_8601_CALENDAR),
    ),
];

#[derive(Default)]
pub struct State {
    current_calendar: Option<id>,
}

struct NSCalendarHostObject {
    identifier: id,
    time_zone: id,
    locale: id,
}
impl HostObject for NSCalendarHostObject {}

#[derive(Clone)]
struct NSDateComponentsHostObject {
    calendar: id,
    time_zone: id,
    era: NSInteger,
    year: NSInteger,
    month: NSInteger,
    day: NSInteger,
    hour: NSInteger,
    minute: NSInteger,
    second: NSInteger,
    nanosecond: NSInteger,
    weekday: NSInteger,
    weekday_ordinal: NSInteger,
    quarter: NSInteger,
    week: NSInteger,
    week_of_month: NSInteger,
    week_of_year: NSInteger,
    year_for_week_of_year: NSInteger,
    leap_month: bool,
}
impl HostObject for NSDateComponentsHostObject {}

impl Default for NSDateComponentsHostObject {
    fn default() -> Self {
        Self {
            calendar: nil,
            time_zone: nil,
            era: NS_UNDEFINED_DATE_COMPONENT,
            year: NS_UNDEFINED_DATE_COMPONENT,
            month: NS_UNDEFINED_DATE_COMPONENT,
            day: NS_UNDEFINED_DATE_COMPONENT,
            hour: NS_UNDEFINED_DATE_COMPONENT,
            minute: NS_UNDEFINED_DATE_COMPONENT,
            second: NS_UNDEFINED_DATE_COMPONENT,
            nanosecond: NS_UNDEFINED_DATE_COMPONENT,
            weekday: NS_UNDEFINED_DATE_COMPONENT,
            weekday_ordinal: NS_UNDEFINED_DATE_COMPONENT,
            quarter: NS_UNDEFINED_DATE_COMPONENT,
            week: NS_UNDEFINED_DATE_COMPONENT,
            week_of_month: NS_UNDEFINED_DATE_COMPONENT,
            week_of_year: NS_UNDEFINED_DATE_COMPONENT,
            year_for_week_of_year: NS_UNDEFINED_DATE_COMPONENT,
            leap_month: false,
        }
    }
}

fn component_or_zero(value: NSInteger) -> i32 {
    if value == NS_UNDEFINED_DATE_COMPONENT {
        0
    } else {
        value
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSCalendar: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSCalendarHostObject {
        identifier: nil,
        time_zone: nil,
        locale: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)currentCalendar {
    if let Some(calendar) = env.framework_state.foundation.ns_calendar.current_calendar {
        calendar
    } else {
        let calendar: id = msg![env; this alloc];
        let calendar: id = msg![env; calendar init];
        env.framework_state.foundation.ns_calendar.current_calendar = Some(calendar);
        calendar
    }
}

+ (id)autoupdatingCurrentCalendar {
    msg![env; this currentCalendar]
}

- (id)init {
    let identifier = ns_string::get_static_str(env, NS_GREGORIAN_CALENDAR);
    retain(env, identifier);
    env.objc.borrow_mut::<NSCalendarHostObject>(this).identifier = identifier;
    this
}

- (id)initWithCalendarIdentifier:(id)identifier {
    assert_ne!(identifier, nil);
    retain(env, identifier);
    let old = env.objc.borrow::<NSCalendarHostObject>(this).identifier;
    env.objc.borrow_mut::<NSCalendarHostObject>(this).identifier = identifier;
    release(env, old);
    this
}

- (())dealloc {
    let host_object = env.objc.borrow::<NSCalendarHostObject>(this);
    let (identifier, time_zone, locale) =
        (host_object.identifier, host_object.time_zone, host_object.locale);
    release(env, identifier);
    release(env, time_zone);
    release(env, locale);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)calendarIdentifier {
    env.objc.borrow::<NSCalendarHostObject>(this).identifier
}

- (id)timeZone {
    env.objc.borrow::<NSCalendarHostObject>(this).time_zone
}

- (())setTimeZone:(id)time_zone {
    retain(env, time_zone);
    let old = env.objc.borrow::<NSCalendarHostObject>(this).time_zone;
    env.objc.borrow_mut::<NSCalendarHostObject>(this).time_zone = time_zone;
    release(env, old);
}

- (id)locale {
    env.objc.borrow::<NSCalendarHostObject>(this).locale
}

- (())setLocale:(id)locale {
    retain(env, locale);
    let old = env.objc.borrow::<NSCalendarHostObject>(this).locale;
    env.objc.borrow_mut::<NSCalendarHostObject>(this).locale = locale;
    release(env, old);
}

- (id)components:(NSUInteger)unit_flags fromDate:(id)date {
    let time_interval: NSTimeInterval = msg![env; date timeIntervalSinceReferenceDate];
    let gregorian = CFAbsoluteTimeGetGregorianDate(env, time_interval, nil);
    let class = env.objc.get_known_class("NSDateComponents", &mut env.mem);
    let components: id = msg![env; class alloc];
    let components: id = msg![env; components init];

    if unit_flags & NSEraCalendarUnit != 0 {
        let _: () = msg![env; components setEra:(1 as NSInteger)];
    }
    if unit_flags & NSYearCalendarUnit != 0 {
        let _: () = msg![env; components setYear:(gregorian.year as NSInteger)];
    }
    if unit_flags & NSMonthCalendarUnit != 0 {
        let _: () = msg![env; components setMonth:(gregorian.month as NSInteger)];
    }
    if unit_flags & NSDayCalendarUnit != 0 {
        let _: () = msg![env; components setDay:(gregorian.day as NSInteger)];
    }
    if unit_flags & NSHourCalendarUnit != 0 {
        let _: () = msg![env; components setHour:(gregorian.hours as NSInteger)];
    }
    if unit_flags & NSMinuteCalendarUnit != 0 {
        let _: () = msg![env; components setMinute:(gregorian.minutes as NSInteger)];
    }
    if unit_flags & NSSecondCalendarUnit != 0 {
        let _: () = msg![env; components setSecond:(gregorian.seconds as NSInteger)];
    }
    if unit_flags & NSWeekdayCalendarUnit != 0 {
        // 2001-01-01, the reference date, was a Monday. Cocoa numbers
        // Sunday as 1 through Saturday as 7.
        let days = (time_interval / 86_400.0).floor() as i64;
        let weekday = ((days + 1).rem_euclid(7) + 1) as NSInteger;
        let _: () = msg![env; components setWeekday:weekday];
    }
    if unit_flags & NSCalendarCalendarUnit != 0 {
        let _: () = msg![env; components setCalendar:this];
    }
    if unit_flags & NSTimeZoneCalendarUnit != 0 {
        let time_zone = env.objc.borrow::<NSCalendarHostObject>(this).time_zone;
        let _: () = msg![env; components setTimeZone:time_zone];
    }

    autorelease(env, components)
}

- (id)dateFromComponents:(id)components {
    let values = env
        .objc
        .borrow::<NSDateComponentsHostObject>(components)
        .clone();
    let year = if values.year == NS_UNDEFINED_DATE_COMPONENT {
        1
    } else {
        values.year
    };
    let month = if values.month == NS_UNDEFINED_DATE_COMPONENT {
        1
    } else {
        values.month
    };
    let day = if values.day == NS_UNDEFINED_DATE_COMPONENT {
        1
    } else {
        values.day
    };
    let second = component_or_zero(values.second) as f64
        + component_or_zero(values.nanosecond) as f64 / 1_000_000_000.0;
    let time_interval = absolute_time_from_gregorian(
        year as i64,
        month as i64,
        day as i64,
        component_or_zero(values.hour) as i64,
        component_or_zero(values.minute) as i64,
        second,
    );
    msg_class![env; NSDate dateWithTimeIntervalSinceReferenceDate:time_interval]
}

- (id)dateByAddingComponents:(id)components
                       toDate:(id)date
                       options:(NSUInteger)_options {
    let values = env
        .objc
        .borrow::<NSDateComponentsHostObject>(components)
        .clone();
    let time_interval: NSTimeInterval = msg![env; date timeIntervalSinceReferenceDate];
    let units = CFGregorianUnits {
        years: component_or_zero(values.year),
        months: component_or_zero(values.month),
        days: component_or_zero(values.day) + 7 * component_or_zero(values.week),
        hours: component_or_zero(values.hour),
        minutes: component_or_zero(values.minute),
        seconds: component_or_zero(values.second) as f64
            + component_or_zero(values.nanosecond) as f64 / 1_000_000_000.0,
    };
    let result = add_gregorian_units(time_interval, units);
    msg_class![env; NSDate dateWithTimeIntervalSinceReferenceDate:result]
}

// Foundation ordinals are one-based. This models the Gregorian/GMT calendar
// already used by the rest of this implementation and covers the historical
// unit/container pairs used by early iPhone applications. Unsupported pairs
// return NSNotFound rather than inventing a value.
- (NSUInteger)ordinalityOfUnit:(NSUInteger)smaller
                         inUnit:(NSUInteger)larger
                        forDate:(id)date {
    if date == nil {
        return NSUInteger::MAX;
    }
    let time_interval: NSTimeInterval = msg![env; date timeIntervalSinceReferenceDate];
    let gregorian = CFAbsoluteTimeGetGregorianDate(env, time_interval, nil);
    let ordinal: Option<u64> = match (smaller, larger) {
        (NSMonthCalendarUnit, NSYearCalendarUnit) => Some(gregorian.month as u64),
        (NSDayCalendarUnit, NSMonthCalendarUnit) => Some(gregorian.day as u64),
        (NSDayCalendarUnit, NSYearCalendarUnit) => {
            let year_start = absolute_time_from_gregorian(
                i64::from(gregorian.year), 1, 1, 0, 0, 0.0,
            );
            Some(((time_interval - year_start) / 86_400.0).floor() as u64 + 1)
        }
        (NSHourCalendarUnit, NSDayCalendarUnit) => Some(gregorian.hours as u64 + 1),
        (NSMinuteCalendarUnit, NSHourCalendarUnit) => Some(gregorian.minutes as u64 + 1),
        (NSSecondCalendarUnit, NSMinuteCalendarUnit) => {
            Some(gregorian.seconds.floor() as u64 + 1)
        }
        (NSWeekdayCalendarUnit, NSWeekCalendarUnit) => {
            let days = (time_interval / 86_400.0).floor() as i64;
            Some(((days + 1).rem_euclid(7) + 1) as u64)
        }
        _ => None,
    };
    env.diagnostic_campaign_event(
        "calendar",
        "ordinalityOfUnit:inUnit:forDate: calendar,smaller,larger,date,ordinal",
        [this.to_bits(), smaller, larger, date.to_bits(), ordinal.unwrap_or(u64::MAX) as u32, 0, 0, 0],
    );
    ordinal
        .and_then(|value| NSUInteger::try_from(value).ok())
        .unwrap_or(NSUInteger::MAX)
}

@end

@implementation NSDateComponents: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::<NSDateComponentsHostObject>::default(),
        &mut env.mem,
    )
}

- (id)init {
    this
}

- (())dealloc {
    let host_object = env.objc.borrow::<NSDateComponentsHostObject>(this);
    let (calendar, time_zone) = (host_object.calendar, host_object.time_zone);
    release(env, calendar);
    release(env, time_zone);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)copyWithZone:(NSZonePtr)_zone {
    let copy = env
        .objc
        .borrow::<NSDateComponentsHostObject>(this)
        .clone();
    retain(env, copy.calendar);
    retain(env, copy.time_zone);
    let class = env.objc.get_known_class("NSDateComponents", &mut env.mem);
    env.objc.alloc_object(class, Box::new(copy), &mut env.mem)
}

- (id)calendar { env.objc.borrow::<NSDateComponentsHostObject>(this).calendar }
- (())setCalendar:(id)value {
    retain(env, value);
    let old = env.objc.borrow::<NSDateComponentsHostObject>(this).calendar;
    env.objc.borrow_mut::<NSDateComponentsHostObject>(this).calendar = value;
    release(env, old);
}
- (id)timeZone { env.objc.borrow::<NSDateComponentsHostObject>(this).time_zone }
- (())setTimeZone:(id)value {
    retain(env, value);
    let old = env.objc.borrow::<NSDateComponentsHostObject>(this).time_zone;
    env.objc.borrow_mut::<NSDateComponentsHostObject>(this).time_zone = value;
    release(env, old);
}

- (NSInteger)era { env.objc.borrow::<NSDateComponentsHostObject>(this).era }
- (())setEra:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).era = value }
- (NSInteger)year { env.objc.borrow::<NSDateComponentsHostObject>(this).year }
- (())setYear:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).year = value }
- (NSInteger)month { env.objc.borrow::<NSDateComponentsHostObject>(this).month }
- (())setMonth:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).month = value }
- (NSInteger)day { env.objc.borrow::<NSDateComponentsHostObject>(this).day }
- (())setDay:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).day = value }
- (NSInteger)hour { env.objc.borrow::<NSDateComponentsHostObject>(this).hour }
- (())setHour:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).hour = value }
- (NSInteger)minute { env.objc.borrow::<NSDateComponentsHostObject>(this).minute }
- (())setMinute:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).minute = value }
- (NSInteger)second { env.objc.borrow::<NSDateComponentsHostObject>(this).second }
- (())setSecond:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).second = value }
- (NSInteger)nanosecond { env.objc.borrow::<NSDateComponentsHostObject>(this).nanosecond }
- (())setNanosecond:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).nanosecond = value }
- (NSInteger)weekday { env.objc.borrow::<NSDateComponentsHostObject>(this).weekday }
- (())setWeekday:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).weekday = value }
- (NSInteger)weekdayOrdinal { env.objc.borrow::<NSDateComponentsHostObject>(this).weekday_ordinal }
- (())setWeekdayOrdinal:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).weekday_ordinal = value }
- (NSInteger)quarter { env.objc.borrow::<NSDateComponentsHostObject>(this).quarter }
- (())setQuarter:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).quarter = value }
- (NSInteger)week { env.objc.borrow::<NSDateComponentsHostObject>(this).week }
- (())setWeek:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).week = value }
- (NSInteger)weekOfMonth { env.objc.borrow::<NSDateComponentsHostObject>(this).week_of_month }
- (())setWeekOfMonth:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).week_of_month = value }
- (NSInteger)weekOfYear { env.objc.borrow::<NSDateComponentsHostObject>(this).week_of_year }
- (())setWeekOfYear:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).week_of_year = value }
- (NSInteger)yearForWeekOfYear { env.objc.borrow::<NSDateComponentsHostObject>(this).year_for_week_of_year }
- (())setYearForWeekOfYear:(NSInteger)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).year_for_week_of_year = value }
- (bool)isLeapMonth { env.objc.borrow::<NSDateComponentsHostObject>(this).leap_month }
- (())setLeapMonth:(bool)value { env.objc.borrow_mut::<NSDateComponentsHostObject>(this).leap_month = value }

- (NSInteger)valueForComponent:(NSUInteger)unit {
    let host = env.objc.borrow::<NSDateComponentsHostObject>(this);
    match unit {
        NSEraCalendarUnit => host.era,
        NSYearCalendarUnit => host.year,
        NSMonthCalendarUnit => host.month,
        NSDayCalendarUnit => host.day,
        NSHourCalendarUnit => host.hour,
        NSMinuteCalendarUnit => host.minute,
        NSSecondCalendarUnit => host.second,
        NSWeekCalendarUnit => host.week,
        NSWeekdayCalendarUnit => host.weekday,
        NSWeekdayOrdinalCalendarUnit => host.weekday_ordinal,
        NSQuarterCalendarUnit => host.quarter,
        NSWeekOfMonthCalendarUnit => host.week_of_month,
        NSWeekOfYearCalendarUnit => host.week_of_year,
        NSYearForWeekOfYearCalendarUnit => host.year_for_week_of_year,
        _ => NS_UNDEFINED_DATE_COMPONENT,
    }
}

- (())setValue:(NSInteger)value forComponent:(NSUInteger)unit {
    let host = env.objc.borrow_mut::<NSDateComponentsHostObject>(this);
    match unit {
        NSEraCalendarUnit => host.era = value,
        NSYearCalendarUnit => host.year = value,
        NSMonthCalendarUnit => host.month = value,
        NSDayCalendarUnit => host.day = value,
        NSHourCalendarUnit => host.hour = value,
        NSMinuteCalendarUnit => host.minute = value,
        NSSecondCalendarUnit => host.second = value,
        NSWeekCalendarUnit => host.week = value,
        NSWeekdayCalendarUnit => host.weekday = value,
        NSWeekdayOrdinalCalendarUnit => host.weekday_ordinal = value,
        NSQuarterCalendarUnit => host.quarter = value,
        NSWeekOfMonthCalendarUnit => host.week_of_month = value,
        NSWeekOfYearCalendarUnit => host.week_of_year = value,
        NSYearForWeekOfYearCalendarUnit => host.year_for_week_of_year = value,
        _ => {
            log!("Warning: unknown NSCalendarUnit {:#x}", unit);
        }
    }
}

@end

};

#[cfg(test)]
mod constant_tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn calendar_identifiers_export_historical_string_values() {
        for (symbol, expected) in [
            ("_NSGregorianCalendar", "gregorian"),
            ("_NSBuddhistCalendar", "buddhist"),
            ("_NSChineseCalendar", "chinese"),
            ("_NSHebrewCalendar", "hebrew"),
            ("_NSIslamicCalendar", "islamic"),
            ("_NSIslamicCivilCalendar", "islamic-civil"),
            ("_NSJapaneseCalendar", "japanese"),
            ("_NSPersianCalendar", "persian"),
            ("_NSRepublicOfChinaCalendar", "republic-of-china"),
            ("_NSIndianCalendar", "indian"),
            ("_NSISO8601Calendar", "iso8601"),
        ] {
            let (_, constant) = CONSTANTS
                .iter()
                .find(|(candidate, _)| *candidate == symbol)
                .unwrap();
            let HostConstant::NSString(actual) = constant else {
                panic!("{symbol} was not exported as NSString");
            };
            assert_eq!(*actual, expected);
        }
    }

    #[test]
    fn dev_248_gregorian_ordinality_is_one_based_and_handles_leap_years() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let leap_year_march = absolute_time_from_gregorian(2024, 3, 1, 0, 0, 0.0);
        let date: id = msg_class![env; NSDate dateWithTimeIntervalSinceReferenceDate:
            leap_year_march];
        let calendar: id = msg_class![env; NSCalendar currentCalendar];
        let day: NSUInteger = msg![env; calendar ordinalityOfUnit:NSDayCalendarUnit
                                                           inUnit:NSYearCalendarUnit
                                                          forDate:date];
        let month: NSUInteger = msg![env; calendar ordinalityOfUnit:NSMonthCalendarUnit
                                                             inUnit:NSYearCalendarUnit
                                                            forDate:date];
        assert_eq!(day, 61);
        assert_eq!(month, 3);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
