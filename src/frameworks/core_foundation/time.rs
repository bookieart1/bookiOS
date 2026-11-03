/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Time things including `CFAbsoluteTime`.

use crate::abi::GuestArg;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::frameworks::foundation::NSTimeInterval;
use crate::libc::time::{time_t, timestamp_to_calendar_date};
use crate::mem::SafeRead;
use crate::objc::nil;
use crate::{impl_GuestRet_for_large_struct, Environment};
use std::ops::Add;
use std::time::{Duration, SystemTime};

/// Seconds between Unix and Apple's epochs
pub const SECS_FROM_UNIX_TO_APPLE_EPOCHS: u64 = 978_307_200;

/// The absolute reference date is 1 Jan 2001 00:00:00 GMT
pub fn apple_epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH.add(Duration::from_secs(SECS_FROM_UNIX_TO_APPLE_EPOCHS))
}

pub type CFTimeInterval = NSTimeInterval;
pub type CFAbsoluteTime = CFTimeInterval;

// These are exported double values, not pointers to NSNumber or NSString.
// Apple CFDate.c defines the same offsets from the 2001 reference epoch.
pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFAbsoluteTimeIntervalSince1970",
        HostConstant::Custom(|env| {
            env.mem
                .alloc_and_write(SECS_FROM_UNIX_TO_APPLE_EPOCHS as f64)
                .cast_void()
                .cast_const()
        }),
    ),
    (
        "_kCFAbsoluteTimeIntervalSince1904",
        HostConstant::Custom(|env| {
            env.mem
                .alloc_and_write(3_061_152_000.0f64)
                .cast_void()
                .cast_const()
        }),
    ),
];

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
pub struct CFGregorianDate {
    pub year: i32,    // SInt32
    pub month: i8,    // SInt8
    pub day: i8,      // SInt8
    pub hours: i8,    // SInt8
    pub minutes: i8,  // SInt8
    pub seconds: f64, // double
}
unsafe impl SafeRead for CFGregorianDate {}
impl_GuestRet_for_large_struct!(CFGregorianDate);

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
pub struct CFGregorianUnits {
    pub years: i32,   // SInt32
    pub months: i32,  // SInt32
    pub days: i32,    // SInt32
    pub hours: i32,   // SInt32
    pub minutes: i32, // SInt32
    pub seconds: f64, // double
}
impl GuestArg for CFGregorianUnits {
    const REG_COUNT: usize = 7;

    fn from_regs(regs: &[u32]) -> Self {
        CFGregorianUnits {
            years: GuestArg::from_regs(&regs[0..1]),
            months: GuestArg::from_regs(&regs[1..2]),
            days: GuestArg::from_regs(&regs[2..3]),
            hours: GuestArg::from_regs(&regs[3..4]),
            minutes: GuestArg::from_regs(&regs[4..5]),
            seconds: GuestArg::from_regs(&regs[5..7]),
        }
    }
    fn to_regs(self, regs: &mut [u32]) {
        self.years.to_regs(&mut regs[0..1]);
        self.months.to_regs(&mut regs[1..2]);
        self.days.to_regs(&mut regs[2..3]);
        self.hours.to_regs(&mut regs[3..4]);
        self.minutes.to_regs(&mut regs[4..5]);
        self.seconds.to_regs(&mut regs[5..7]);
    }
}

/// Absolute time is measured in seconds relative to the absolute reference date
/// of Jan 1 2001 00:00:00 GMT.
fn CFAbsoluteTimeGetCurrent(_env: &mut Environment) -> CFAbsoluteTime {
    SystemTime::now()
        .duration_since(apple_epoch())
        .unwrap()
        .as_secs_f64()
}

type CFTimeZoneRef = CFTypeRef;

fn CFTimeZoneCopySystem(_env: &mut Environment) -> CFTimeZoneRef {
    // TODO: implement (nil seems to correspond to GMT)
    nil
}

pub fn CFAbsoluteTimeGetGregorianDate(
    _env: &mut Environment,
    at: CFAbsoluteTime,
    tz: CFTimeZoneRef,
) -> CFGregorianDate {
    assert!(tz.is_null());
    let time64 = apple_epoch()
        .add(Duration::from_secs_f64(at))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time = time64 as time_t;
    let tm = timestamp_to_calendar_date(time);
    CFGregorianDate {
        year: 1900 + tm.tm_year,
        month: (tm.tm_mon + 1) as i8,
        day: tm.tm_mday as i8,
        hours: tm.tm_hour as i8,
        minutes: tm.tm_min as i8,
        seconds: tm.tm_sec.into(),
    }
}

fn CFAbsoluteTimeGetDayOfWeek(env: &mut Environment, at: CFAbsoluteTime, tz: CFTimeZoneRef) -> i32 {
    assert!(tz.is_null());
    CFAbsoluteTimeGetGregorianDate(env, at, tz).day.into()
}

const DAY_SECONDS: f64 = 86400.0;
/// Days from 1970-01-01 to the absolute reference date.
const DAYS_FROM_UNIX_TO_APPLE_EPOCHS: i64 = 11323;

/// Days since the absolute reference date of a Gregorian date. A month of 13
/// is January of the next year.
fn days_from_date(year: i64, month: i64, day: i64) -> i64 {
    // Counts from March, so the leap day is at the end of the year.
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era - 719468 - DAYS_FROM_UNIX_TO_APPLE_EPOCHS
}

/// Convert Gregorian date components to an absolute time relative to Apple's
/// 2001 reference date. Foundation's calendar implementation uses this to
/// construct `NSDate` objects without exposing Core Foundation internals to
/// the guest.
pub(crate) fn absolute_time_from_gregorian(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: f64,
) -> CFAbsoluteTime {
    days_from_date(year, month, day) as f64 * DAY_SECONDS
        + hour as f64 * 3600.0
        + minute as f64 * 60.0
        + second
}

/// The Gregorian (year, month, day) of a day since the absolute reference
/// date. The inverse of [days_from_date].
fn date_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + DAYS_FROM_UNIX_TO_APPLE_EPOCHS + 719468;
    let era = days.div_euclid(146097);
    let day_of_era = days - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = (month_from_march + 2) % 12 + 1;
    let year = year_of_era + era * 400 + (month <= 2) as i64;
    (year, month, day)
}

fn CFAbsoluteTimeAddGregorianUnits(
    _env: &mut Environment,
    at: CFAbsoluteTime,
    tz: CFTimeZoneRef,
    units: CFGregorianUnits,
) -> CFAbsoluteTime {
    assert!(tz.is_null());
    add_gregorian_units(at, units)
}

/// Like Apple's: years and months move the date, keeping the time of day and
/// clamping the day to the new month's length. The rest are added as seconds.
pub(crate) fn add_gregorian_units(
    at: CFAbsoluteTime,
    units: CFGregorianUnits,
) -> CFAbsoluteTime {
    let days = (at / DAY_SECONDS).floor();
    let time_of_day = at - days * DAY_SECONDS;
    let (year, month, day) = date_from_days(days as i64);
    let months = year * 12 + month - 1 + i64::from(units.years) * 12 + i64::from(units.months);
    let (year, month) = (months.div_euclid(12), months.rem_euclid(12) + 1);
    let month_days = days_from_date(year, month + 1, 1) - days_from_date(year, month, 1);
    let days = days_from_date(year, month, day.min(month_days));
    days as f64 * DAY_SECONDS
        + time_of_day
        + f64::from(units.days) * DAY_SECONDS
        + f64::from(units.hours) * 3600.0
        + f64::from(units.minutes) * 60.0
        + units.seconds
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFAbsoluteTimeGetCurrent()),
    export_c_func!(CFTimeZoneCopySystem()),
    export_c_func!(CFAbsoluteTimeGetGregorianDate(_, _)),
    export_c_func!(CFAbsoluteTimeGetDayOfWeek(_, _)),
    export_c_func!(CFAbsoluteTimeAddGregorianUnits(_, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_time_constants_are_guest_doubles() {
        let mut env = Environment::new_fake_for_test();
        for ((_, constant), expected) in CONSTANTS.iter().zip([978_307_200.0f64, 3_061_152_000.0]) {
            let HostConstant::Custom(export) = constant else {
                panic!("expected data export")
            };
            let address = export(&mut env);
            assert_ne!(address.to_bits(), 0);
            assert_eq!(env.mem.read::<f64>(address.cast()), expected);
            assert_eq!(
                env.mem.get_bytes_fallible(address, 8).unwrap(),
                expected.to_le_bytes()
            );
            // Execute a guest load of both ABI words; this catches accidentally
            // exporting a pointer-to-value instead of the double itself.
            let code = env.mem.alloc(12).cast::<u32>();
            env.mem.write(code, 0xe592_0000u32); // ldr r0, [r2]
            env.mem.write(code + 1, 0xe592_1004u32); // ldr r1, [r2, #4]
            env.mem.write(code + 2, 0xef12_3456u32); // svc #0x123456
            env.cpu.regs_mut()[2] = address.to_bits();
            env.cpu
                .branch(crate::abi::GuestFunction::from_addr_with_thumb_bit(
                    code.to_bits(),
                ));
            let mut ticks = 16;
            assert!(matches!(
                env.cpu.run_or_step(&mut env.mem, Some(&mut ticks)),
                crate::cpu::CpuState::Svc(0x123456)
            ));
            let bits = u64::from(env.cpu.regs()[0]) | (u64::from(env.cpu.regs()[1]) << 32);
            assert_eq!(f64::from_bits(bits), expected);
        }
        assert_eq!(at(1970, 1, 1), -978_307_200.0);
        assert_eq!(at(1904, 1, 1), -3_061_152_000.0);
    }

    fn at(year: i64, month: i64, day: i64) -> CFAbsoluteTime {
        days_from_date(year, month, day) as f64 * DAY_SECONDS
    }

    #[test]
    fn test_days_from_date() {
        assert_eq!(days_from_date(2001, 1, 1), 0);
        assert_eq!(days_from_date(2000, 12, 31), -1);
        assert_eq!(days_from_date(2001, 13, 1), 365);
        assert_eq!(days_from_date(2004, 3, 1) - days_from_date(2004, 2, 1), 29);
        for days in -800_000..800_000 {
            let (year, month, day) = date_from_days(days);
            assert_eq!(days_from_date(year, month, day), days);
        }
    }

    #[test]
    fn test_add_gregorian_units() {
        let add = |at, years, months, days, seconds| {
            let units = CFGregorianUnits {
                years,
                months,
                days,
                hours: 0,
                minutes: 0,
                seconds,
            };
            add_gregorian_units(at, units)
        };
        assert_eq!(add(at(2026, 10, 13), 31, 0, 0, 0.0), at(2057, 10, 13));
        assert_eq!(add(at(2001, 1, 31), 0, 1, 0, 0.0), at(2001, 2, 28));
        assert_eq!(add(at(2004, 2, 29), 1, 0, 0, 0.0), at(2005, 2, 28));
        assert_eq!(add(at(2001, 3, 1), 0, -14, 0, 0.0), at(2000, 1, 1));
        assert_eq!(add(at(2001, 12, 31), 0, 0, 1, 0.0), at(2002, 1, 1));
        assert_eq!(
            add(at(2001, 1, 1) + 5.5, 0, 1, 0, 1.0),
            at(2001, 2, 1) + 6.5
        );
    }
}
