/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded historical `NSNumberFormatter` support.

use super::{ns_string, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, ClassExports, HostObject,
    NSZonePtr,
};
use crate::Environment;

const NSNumberFormatterNoStyle: NSUInteger = 0;
const NSNumberFormatterDecimalStyle: NSUInteger = 1;
const NSNumberFormatterCurrencyStyle: NSUInteger = 2;
const NSNumberFormatterPercentStyle: NSUInteger = 3;
const NSNumberFormatterScientificStyle: NSUInteger = 4;

struct NSNumberFormatterHostObject {
    formatter_behavior: NSUInteger,
    style: NSUInteger,
    minimum_fraction_digits: NSUInteger,
    maximum_fraction_digits: NSUInteger,
    minimum_integer_digits: NSUInteger,
    uses_grouping_separator: bool,
    grouping_size: NSUInteger,
    padding_position: NSUInteger,
    lenient: bool,
    positive_format: id,
    negative_format: id,
    grouping_separator: id,
    currency_grouping_separator: id,
    currency_code: id,
    currency_symbol: id,
    positive_prefix: id,
    positive_suffix: id,
    negative_prefix: id,
    negative_suffix: id,
    multiplier: id,
    locale: id,
}
impl HostObject for NSNumberFormatterHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSNumberFormatter: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberFormatterHostObject {
        style: NSNumberFormatterNoStyle,
        formatter_behavior: 0,
        minimum_fraction_digits: 0,
        maximum_fraction_digits: 3,
        minimum_integer_digits: 1,
        uses_grouping_separator: false,
        grouping_size: 3,
        padding_position: 0,
        lenient: false,
        positive_format: nil,
        negative_format: nil,
        grouping_separator: nil,
        currency_grouping_separator: nil,
        currency_code: nil,
        currency_symbol: nil,
        positive_prefix: nil,
        positive_suffix: nil,
        negative_prefix: nil,
        negative_suffix: nil,
        multiplier: nil,
        locale: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())dealloc {
    let (positive_format, negative_format, grouping_separator,
         currency_grouping_separator, currency_code, currency_symbol,
         positive_prefix, positive_suffix, negative_prefix, negative_suffix,
         multiplier, locale) = {
        let state = env.objc.borrow_mut::<NSNumberFormatterHostObject>(this);
        (
            std::mem::replace(&mut state.positive_format, nil),
            std::mem::replace(&mut state.negative_format, nil),
            std::mem::replace(&mut state.grouping_separator, nil),
            std::mem::replace(&mut state.currency_grouping_separator, nil),
            std::mem::replace(&mut state.currency_code, nil),
            std::mem::replace(&mut state.currency_symbol, nil),
            std::mem::replace(&mut state.positive_prefix, nil),
            std::mem::replace(&mut state.positive_suffix, nil),
            std::mem::replace(&mut state.negative_prefix, nil),
            std::mem::replace(&mut state.negative_suffix, nil),
            std::mem::replace(&mut state.multiplier, nil),
            std::mem::replace(&mut state.locale, nil),
        )
    };
    release(env, positive_format);
    release(env, negative_format);
    release(env, grouping_separator);
    release(env, currency_grouping_separator);
    release(env, currency_code);
    release(env, currency_symbol);
    release(env, positive_prefix);
    release(env, positive_suffix);
    release(env, negative_prefix);
    release(env, negative_suffix);
    release(env, multiplier);
    release(env, locale);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (NSUInteger)numberStyle {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).style
}
- (())setNumberStyle:(NSUInteger)style {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).style = style;
}

- (NSUInteger)formatterBehavior {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).formatter_behavior
}
- (())setFormatterBehavior:(NSUInteger)behavior {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).formatter_behavior = behavior;
}

- (NSUInteger)minimumFractionDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).minimum_fraction_digits
}
- (())setMinimumFractionDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).minimum_fraction_digits = digits;
}
- (NSUInteger)maximumFractionDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).maximum_fraction_digits
}
- (())setMaximumFractionDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).maximum_fraction_digits = digits;
}
- (NSUInteger)minimumIntegerDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).minimum_integer_digits
}
- (())setMinimumIntegerDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).minimum_integer_digits = digits;
}
- (bool)usesGroupingSeparator {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).uses_grouping_separator
}
- (())setUsesGroupingSeparator:(bool)uses {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).uses_grouping_separator = uses;
}
- (NSUInteger)groupingSize {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).grouping_size
}
- (())setGroupingSize:(NSUInteger)size {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).grouping_size = size;
}
- (NSUInteger)paddingPosition {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).padding_position
}
- (())setPaddingPosition:(NSUInteger)position {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).padding_position = position;
}
- (id)groupingSeparator {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).grouping_separator
}
- (())setGroupingSeparator:(id)separator {
    set_owned_string(env, this, separator, false);
}
- (id)currencyGroupingSeparator {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).currency_grouping_separator
}
- (())setCurrencyGroupingSeparator:(id)separator {
    set_owned_string(env, this, separator, true);
}
- (id)currencyCode {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).currency_code
}
- (())setCurrencyCode:(id)code {
    let copied: id = if code == nil { nil } else { msg![env; code copy] };
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).currency_code,
        copied,
    );
    release(env, old);
}
- (id)currencySymbol {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).currency_symbol
}
- (())setCurrencySymbol:(id)symbol {
    let copied: id = if symbol == nil { nil } else { msg![env; symbol copy] };
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).currency_symbol,
        copied,
    );
    release(env, old);
}
- (id)positivePrefix {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).positive_prefix
}
- (())setPositivePrefix:(id)prefix {
    set_owned_affix(env, this, prefix, 0);
}
- (id)positiveSuffix {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).positive_suffix
}
- (())setPositiveSuffix:(id)suffix {
    set_owned_affix(env, this, suffix, 1);
}
- (id)negativePrefix {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).negative_prefix
}
- (())setNegativePrefix:(id)prefix {
    set_owned_affix(env, this, prefix, 2);
}
- (id)negativeSuffix {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).negative_suffix
}
- (())setNegativeSuffix:(id)suffix {
    set_owned_affix(env, this, suffix, 3);
}
- (id)multiplier {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).multiplier
}
- (())setMultiplier:(id)multiplier {
    let copied: id = if multiplier == nil { nil } else { msg![env; multiplier copy] };
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).multiplier,
        copied,
    );
    release(env, old);
}
- (bool)isLenient {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).lenient
}
- (())setLenient:(bool)lenient {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).lenient = lenient;
}

- (id)positiveFormat {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).positive_format
}
- (())setPositiveFormat:(id)format {
    set_format(env, this, format, true);
}
- (id)negativeFormat {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).negative_format
}
- (())setNegativeFormat:(id)format {
    set_format(env, this, format, false);
}

- (id)locale {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).locale
}
- (())setLocale:(id)locale {
    let copied: id = if locale == nil { nil } else { msg![env; locale copy] };
    let old = std::mem::replace(
        &mut env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).locale,
        copied,
    );
    release(env, old);
}

- (id)stringFromNumber:(id)number {
    if number == nil {
        return nil;
    }
    let (style, min_fraction, max_fraction, min_integer, grouping_size, grouping,
         positive_format, negative_format, grouping_separator,
         currency_grouping_separator, currency_symbol, positive_prefix,
         positive_suffix, negative_prefix, negative_suffix, multiplier) = {
        let state = env.objc.borrow::<NSNumberFormatterHostObject>(this);
        (
            state.style,
            state.minimum_fraction_digits as usize,
            state.maximum_fraction_digits.max(state.minimum_fraction_digits) as usize,
            state.minimum_integer_digits as usize,
            state.grouping_size as usize,
            state.uses_grouping_separator,
            state.positive_format,
            state.negative_format,
            state.grouping_separator,
            state.currency_grouping_separator,
            state.currency_symbol,
            state.positive_prefix,
            state.positive_suffix,
            state.negative_prefix,
            state.negative_suffix,
            state.multiplier,
        )
    };

    if positive_format != nil {
        let positive = ns_string::to_rust_string(env, positive_format);
        let negative = (negative_format != nil)
            .then(|| ns_string::to_rust_string(env, negative_format));
        let value: f64 = msg![env; number doubleValue];
        if let Some(output) = format_with_pattern(value, &positive, negative.as_deref()) {
            let string = ns_string::from_rust_string(env, output);
            return autorelease(env, string);
        }
    }

    if style == NSNumberFormatterNoStyle {
        return msg![env; number stringValue];
    }

    let mut value: f64 = msg![env; number doubleValue];
    if multiplier != nil {
        let scale: f64 = msg![env; multiplier doubleValue];
        value *= scale;
    }
    let negative = value.is_sign_negative();
    value = value.abs();
    if style == NSNumberFormatterPercentStyle {
        value *= 100.0;
    }
    let mut output = if style == NSNumberFormatterScientificStyle {
        format!("{value:.max_fraction$E}")
    } else {
        let mut fixed = format!("{value:.max_fraction$}");
        trim_fraction(&mut fixed, min_fraction);
        pad_integer(&mut fixed, min_integer);
        if grouping {
            let separator = if style == NSNumberFormatterCurrencyStyle
                && currency_grouping_separator != nil
            {
                ns_string::to_rust_string(env, currency_grouping_separator)
            } else if grouping_separator != nil {
                ns_string::to_rust_string(env, grouping_separator)
            } else {
                ",".into()
            };
            fixed = add_grouping(&fixed, grouping_size.max(1), &separator);
        }
        fixed
    };
    let currency_symbol = if currency_symbol == nil {
            "$".into()
        } else {
            ns_string::to_rust_string(env, currency_symbol).into_owned()
        };
    let default_prefix = if style == NSNumberFormatterCurrencyStyle {
        format!("{}{}", if negative { "-" } else { "" }, currency_symbol)
    } else if negative {
        "-".to_owned()
    } else {
        String::new()
    };
    let default_suffix = if style == NSNumberFormatterPercentStyle { "%" } else { "" };
    let prefix = if negative { negative_prefix } else { positive_prefix };
    let suffix = if negative { negative_suffix } else { positive_suffix };
    let prefix = if prefix == nil {
        default_prefix
    } else {
        ns_string::to_rust_string(env, prefix).into_owned()
    };
    let suffix = if suffix == nil {
        default_suffix.to_owned()
    } else {
        ns_string::to_rust_string(env, suffix).into_owned()
    };
    output.insert_str(0, &prefix);
    output.push_str(&suffix);
    let string = ns_string::from_rust_string(env, output);
    autorelease(env, string)
}

- (id)stringForObjectValue:(id)object {
    msg![env; this stringFromNumber:object]
}

- (id)numberFromString:(id)string {
    if string == nil {
        return nil;
    }
    let (style, lenient, grouping_separator, currency_grouping_separator, multiplier) = {
        let state = env.objc.borrow::<NSNumberFormatterHostObject>(this);
        (state.style, state.lenient, state.grouping_separator,
         state.currency_grouping_separator, state.multiplier)
    };
    let mut value = ns_string::to_rust_string(env, string).trim().to_owned();
    let separator = if style == NSNumberFormatterCurrencyStyle
        && currency_grouping_separator != nil
    {
        ns_string::to_rust_string(env, currency_grouping_separator).into_owned()
    } else if grouping_separator != nil {
        ns_string::to_rust_string(env, grouping_separator).into_owned()
    } else {
        ",".to_owned()
    };
    if !separator.is_empty() {
        value = value.replace(&separator, "");
    }
    value.retain(|character| character != '$' && character != '%');
    let parsed = if lenient {
        value.split_whitespace().next().unwrap_or("").parse::<f64>()
    } else {
        value.parse::<f64>()
    };
    let Ok(mut parsed) = parsed else {
        return nil;
    };
    if style == NSNumberFormatterPercentStyle {
        parsed /= 100.0;
    }
    if multiplier != nil {
        let scale: f64 = msg![env; multiplier doubleValue];
        if scale == 0.0 {
            return nil;
        }
        parsed /= scale;
    }
    msg_class![env; NSNumber numberWithDouble:parsed]
}

@end

};

fn set_format(env: &mut Environment, formatter: id, format: id, positive: bool) {
    let copied: id = if format == nil {
        nil
    } else {
        msg![env; format copy]
    };
    let old = {
        let state = env
            .objc
            .borrow_mut::<NSNumberFormatterHostObject>(formatter);
        if positive {
            std::mem::replace(&mut state.positive_format, copied)
        } else {
            std::mem::replace(&mut state.negative_format, copied)
        }
    };
    release(env, old);
}

fn set_owned_string(env: &mut Environment, formatter: id, value: id, currency: bool) {
    let copied: id = if value == nil {
        nil
    } else {
        msg![env; value copy]
    };
    let old = {
        let state = env
            .objc
            .borrow_mut::<NSNumberFormatterHostObject>(formatter);
        if currency {
            std::mem::replace(&mut state.currency_grouping_separator, copied)
        } else {
            std::mem::replace(&mut state.grouping_separator, copied)
        }
    };
    release(env, old);
}

fn set_owned_affix(env: &mut Environment, formatter: id, value: id, slot: u8) {
    let copied: id = if value == nil {
        nil
    } else {
        msg![env; value copy]
    };
    let old = {
        let state = env
            .objc
            .borrow_mut::<NSNumberFormatterHostObject>(formatter);
        match slot {
            0 => std::mem::replace(&mut state.positive_prefix, copied),
            1 => std::mem::replace(&mut state.positive_suffix, copied),
            2 => std::mem::replace(&mut state.negative_prefix, copied),
            3 => std::mem::replace(&mut state.negative_suffix, copied),
            _ => unreachable!(),
        }
    };
    release(env, old);
}

/// A bounded subset of the historical NSNumberFormatter/ICU decimal pattern
/// language. It preserves literal prefix/suffix text and implements the `0`,
/// `#`, decimal, grouping and percent behavior used by older game UIs.
fn format_with_pattern(value: f64, positive: &str, negative: Option<&str>) -> Option<String> {
    let (pattern, magnitude, add_minus) = if value.is_sign_negative() {
        if let Some(negative) = negative {
            (negative, value.abs(), false)
        } else {
            (positive, value.abs(), true)
        }
    } else {
        (positive, value, false)
    };
    let pattern = pattern.split(';').next().unwrap_or(pattern);
    let first = pattern.find(|character| character == '0' || character == '#')?;
    let last = pattern.rfind(|character| character == '0' || character == '#')?;
    let prefix = &pattern[..first];
    let suffix = &pattern[last + 1..];
    let number_pattern = &pattern[first..=last];
    let (integer_pattern, fraction_pattern) = number_pattern
        .split_once('.')
        .map_or((number_pattern, ""), |parts| parts);
    let minimum_integer = integer_pattern.chars().filter(|&c| c == '0').count();
    let minimum_fraction = fraction_pattern.chars().filter(|&c| c == '0').count();
    let maximum_fraction = fraction_pattern
        .chars()
        .filter(|&c| c == '0' || c == '#')
        .count();
    let grouping_size = integer_pattern
        .rfind(',')
        .map(|comma| {
            integer_pattern[comma + 1..]
                .chars()
                .filter(|&c| c == '0' || c == '#')
                .count()
        })
        .filter(|&size| size != 0);
    let percent = prefix.contains('%') || suffix.contains('%');
    let magnitude = if percent {
        magnitude * 100.0
    } else {
        magnitude
    };

    let mut number = format!("{magnitude:.maximum_fraction$}");
    trim_fraction(&mut number, minimum_fraction);
    pad_integer(&mut number, minimum_integer);
    if let Some(grouping_size) = grouping_size {
        number = add_grouping(&number, grouping_size, ",");
    }

    let mut output = String::new();
    if add_minus {
        output.push('-');
    }
    output.push_str(&prefix.replace('\'', ""));
    output.push_str(&number);
    output.push_str(&suffix.replace('\'', ""));
    Some(output)
}

fn trim_fraction(value: &mut String, minimum_digits: usize) {
    let Some(decimal) = value.find('.') else {
        return;
    };
    while value.ends_with('0') && value.len() - decimal - 1 > minimum_digits {
        value.pop();
    }
    if value.ends_with('.') && minimum_digits == 0 {
        value.pop();
    }
}

fn pad_integer(value: &mut String, minimum_digits: usize) {
    let negative = value.starts_with('-');
    let start = usize::from(negative);
    let integer_end = value.find('.').unwrap_or(value.len());
    let integer_digits = integer_end.saturating_sub(start);
    if integer_digits < minimum_digits {
        value.insert_str(start, &"0".repeat(minimum_digits - integer_digits));
    }
}

fn add_grouping(value: &str, grouping_size: usize, separator: &str) -> String {
    let negative = value.starts_with('-');
    let start = usize::from(negative);
    let integer_end = value.find('.').unwrap_or(value.len());
    let integer = &value[start..integer_end];
    let mut grouped = String::new();
    if negative {
        grouped.push('-');
    }
    for (idx, character) in integer.chars().enumerate() {
        if idx != 0 && (integer.len() - idx) % grouping_size == 0 {
            grouped.push_str(separator);
        }
        grouped.push(character);
    }
    grouped.push_str(&value[integer_end..]);
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::release;

    #[test]
    fn development_255_parapanic_currency_code_is_copied_and_round_trips() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        let initial: id = msg![env; formatter currencyCode];
        assert_eq!(initial, nil);
        let code = ns_string::from_rust_string(env, "CRD".to_owned());
        () = msg![env; formatter setCurrencyCode:code];
        release(env, code);
        let stored: id = msg![env; formatter currencyCode];
        assert_eq!(ns_string::to_rust_string(env, stored), "CRD");
        release(env, formatter);
    }

    #[test]
    fn development_255_parapanic_currency_symbol_round_trips_and_formats() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterCurrencyStyle];
        let symbol = ns_string::from_rust_string(env, "CR".to_owned());
        () = msg![env; formatter setCurrencySymbol:symbol];
        release(env, symbol);
        let stored: id = msg![env; formatter currencySymbol];
        assert_eq!(ns_string::to_rust_string(env, stored), "CR");
        let number: id = msg_class![env; NSNumber numberWithDouble:12.0f64];
        let output: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, output), "CR12");
        release(env, formatter);
    }

    #[test]
    fn development_255_parapanic_number_prefixes_and_suffixes_round_trip_and_format() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterCurrencyStyle];
        () = msg![env; formatter setMaximumFractionDigits:0u32];

        let positive_prefix = ns_string::from_rust_string(env, "+".to_owned());
        let positive_suffix = ns_string::from_rust_string(env, "!".to_owned());
        let negative_prefix = ns_string::from_rust_string(env, "(".to_owned());
        let negative_suffix = ns_string::from_rust_string(env, ")".to_owned());
        () = msg![env; formatter setPositivePrefix:positive_prefix];
        () = msg![env; formatter setPositiveSuffix:positive_suffix];
        () = msg![env; formatter setNegativePrefix:negative_prefix];
        () = msg![env; formatter setNegativeSuffix:negative_suffix];
        release(env, positive_prefix);
        release(env, positive_suffix);
        release(env, negative_prefix);
        release(env, negative_suffix);

        let positive_prefix: id = msg![env; formatter positivePrefix];
        let negative_suffix: id = msg![env; formatter negativeSuffix];
        assert_eq!(ns_string::to_rust_string(env, positive_prefix), "+");
        assert_eq!(ns_string::to_rust_string(env, negative_suffix), ")");

        let positive: id = msg_class![env; NSNumber numberWithDouble:12.0f64];
        let negative: id = msg_class![env; NSNumber numberWithDouble:(-12.0f64)];
        let positive_output: id = msg![env; formatter stringFromNumber:positive];
        let negative_output: id = msg![env; formatter stringFromNumber:negative];
        assert_eq!(ns_string::to_rust_string(env, positive_output), "+12!");
        assert_eq!(ns_string::to_rust_string(env, negative_output), "(12)");
        release(env, formatter);
    }

    #[test]
    fn development_255_parapanic_multiplier_round_trips_and_scales_both_directions() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterDecimalStyle];
        () = msg![env; formatter setMaximumFractionDigits:2u32];
        let multiplier: id = msg_class![env; NSNumber numberWithDouble:100.0f64];
        () = msg![env; formatter setMultiplier:multiplier];
        let stored: id = msg![env; formatter multiplier];
        let stored_value: f64 = msg![env; stored doubleValue];
        assert_eq!(stored_value, 100.0);

        let number: id = msg_class![env; NSNumber numberWithDouble:1.25f64];
        let output: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, output), "125");
        let parsed: id = msg![env; formatter numberFromString:output];
        let parsed_value: f64 = msg![env; parsed doubleValue];
        assert_eq!(parsed_value, 1.25);
        release(env, formatter);
    }

    #[test]
    fn development_255_parapanic_padding_position_round_trips_without_format_width() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        for position in 0u32..=3 {
            let _: () = msg![env; formatter setPaddingPosition:position];
            let stored: NSUInteger = msg![env; formatter paddingPosition];
            assert_eq!(stored, position);
        }
        let _: () = msg![env; formatter setNumberStyle:NSNumberFormatterDecimalStyle];
        let number: id = msg_class![env; NSNumber numberWithDouble:12.0f64];
        let output: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, output), "12");
        release(env, formatter);
    }

    #[test]
    fn development_255_parapanic_decimal_generation_and_minus_sign_contract() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        let minus = ns_string::from_rust_string(env, "~".to_owned());
        let _: () = msg![env; formatter setMinusSign:minus];
        release(env, minus);
        let _: () = msg![env; formatter setGeneratesDecimalNumbers:true];
        let generates: bool = msg![env; formatter generatesDecimalNumbers];
        let stored_minus: id = msg![env; formatter minusSign];
        assert!(generates);
        assert_eq!(ns_string::to_rust_string(env, stored_minus), "~");

        let negative: id = msg_class![env; NSNumber numberWithDouble:(-12.5f64)];
        let output: id = msg![env; formatter stringFromNumber:negative];
        assert_eq!(ns_string::to_rust_string(env, output), "~12.5");
        let parsed: id = msg![env; formatter numberFromString:output];
        let decimal_class: crate::objc::Class = msg_class![env; NSDecimalNumber class];
        let is_decimal: bool = msg![env; parsed isKindOfClass:decimal_class];
        assert!(is_decimal);
        let parsed_value: f64 = msg![env; parsed doubleValue];
        assert_eq!(parsed_value, -12.5);
        release(env, formatter);
    }

    #[test]
    fn decimal_formatter_round_trips_bounded_number_state() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterDecimalStyle];
        () = msg![env; formatter setMaximumFractionDigits:2u32];
        () = msg![env; formatter setUsesGroupingSeparator:true];
        let number: id = msg_class![env; NSNumber numberWithDouble:1234.5f64];
        let string: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, string), "1,234.5");
        let parsed: id = msg![env; formatter numberFromString:string];
        let value: f64 = msg![env; parsed doubleValue];
        assert_eq!(value, 1234.5);
        release(env, formatter);
    }

    #[test]
    fn positive_format_is_copied_and_applied_with_negative_fallback() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        let pattern = ns_string::from_rust_string(env, "#,##0.00".to_owned());
        () = msg![env; formatter setPositiveFormat:pattern];
        release(env, pattern);

        let number: id = msg_class![env; NSNumber numberWithDouble:(-1234.5f64)];
        let string: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, string), "-1,234.50");
        release(env, formatter);
    }

    #[test]
    fn formatter_behavior_round_trips() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setFormatterBehavior:1040u32];
        let behavior: NSUInteger = msg![env; formatter formatterBehavior];
        assert_eq!(behavior, 1040);
        release(env, formatter);
    }

    #[test]
    fn custom_grouping_separators_are_copied_and_applied() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterCurrencyStyle];
        () = msg![env; formatter setUsesGroupingSeparator:true];
        let separator = ns_string::from_rust_string(env, "_".to_owned());
        () = msg![env; formatter setCurrencyGroupingSeparator:separator];
        release(env, separator);
        let number: id = msg_class![env; NSNumber numberWithDouble:1234.0f64];
        let string: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, string), "$1_234");
        let parsed: id = msg![env; formatter numberFromString:string];
        let value: f64 = msg![env; parsed doubleValue];
        assert_eq!(value, 1234.0);
        release(env, formatter);
    }

    #[test]
    fn locale_is_copied_and_round_trips() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        let locale = ns_string::from_rust_string(env, "en_US".to_owned());
        () = msg![env; formatter setLocale:locale];
        release(env, locale);
        let stored: id = msg![env; formatter locale];
        assert_eq!(ns_string::to_rust_string(env, stored), "en_US");
        release(env, formatter);
    }
}
