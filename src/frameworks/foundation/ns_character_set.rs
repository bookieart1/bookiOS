/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSCharacterSet` class cluster, including `NSMutableCharacterSet`.

use super::{ns_string, unichar, NSRange, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, retain, ClassExports, HostObject, NSZonePtr,
};
use std::collections::HashSet;

// Unicode General Category Zs and CHARACTER TABULATION (U+0009).
const WHITESPACE_CHARACTERS: [char; 18] = [
    '\u{0020}', '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
    '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{202F}', '\u{205F}',
    '\u{3000}', '\u{0009}',
];
// The newline characters (U+000A - U+000D, U+0085, U+2028, and U+2029).
const NEWLINE_CHARACTERS: [char; 7] = [
    '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0085}', '\u{2028}', '\u{2029}',
];

// Unicode General Category Nd ranges that fit in a UTF-16 code unit. These
// are the decimal digits represented by NSCharacterSet's historical
// `decimalDigitCharacterSet`, rather than every numeric character.
const DECIMAL_DIGIT_RANGES: &[(u16, u16)] = &[
    (0x0030, 0x0039),
    (0x0660, 0x0669),
    (0x06f0, 0x06f9),
    (0x07c0, 0x07c9),
    (0x0966, 0x096f),
    (0x09e6, 0x09ef),
    (0x0a66, 0x0a6f),
    (0x0ae6, 0x0aef),
    (0x0b66, 0x0b6f),
    (0x0be6, 0x0bef),
    (0x0c66, 0x0c6f),
    (0x0ce6, 0x0cef),
    (0x0d66, 0x0d6f),
    (0x0e50, 0x0e59),
    (0x0ed0, 0x0ed9),
    (0x0f20, 0x0f29),
    (0x1040, 0x1049),
    (0x1090, 0x1099),
    (0x17e0, 0x17e9),
    (0x1810, 0x1819),
    (0x1946, 0x194f),
    (0x19d0, 0x19d9),
    (0x1a80, 0x1a89),
    (0x1a90, 0x1a99),
    (0x1b50, 0x1b59),
    (0x1bb0, 0x1bb9),
    (0x1c40, 0x1c49),
    (0x1c50, 0x1c59),
    (0xa620, 0xa629),
    (0xa8d0, 0xa8d9),
    (0xa900, 0xa909),
    (0xa9d0, 0xa9d9),
    (0xa9f0, 0xa9f9),
    (0xaa50, 0xaa59),
    (0xabf0, 0xabf9),
    (0xff10, 0xff19),
];

// Unicode General Category Cc and Cf.
// TODO: This is not actually the full list of charadcters in Cc and Cf, but
// these are the ones reported from a simulator run with all `unichar`s. This
// excludes characters outside the BMP, whcih would only be needed when we
// support -longCharacterIsMember:.
const CONTROL_CHARACTERS: [char; 99] = [
    '\u{0000}', '\u{0001}', '\u{0002}', '\u{0003}', '\u{0004}', '\u{0005}', '\u{0006}', '\u{0007}',
    '\u{0008}', '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{000E}', '\u{000F}',
    '\u{0010}', '\u{0011}', '\u{0012}', '\u{0013}', '\u{0014}', '\u{0015}', '\u{0016}', '\u{0017}',
    '\u{0018}', '\u{0019}', '\u{001A}', '\u{001B}', '\u{001C}', '\u{001D}', '\u{001E}', '\u{001F}',
    '\u{007F}', '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}',
    '\u{0087}', '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}',
    '\u{008F}', '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}',
    '\u{0097}', '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}',
    '\u{009F}', '\u{00AD}', '\u{0600}', '\u{0601}', '\u{0602}', '\u{0603}', '\u{06DD}', '\u{070F}',
    '\u{17B4}', '\u{17B5}', '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{200F}', '\u{202A}',
    '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2060}', '\u{2061}', '\u{2062}', '\u{2063}',
    '\u{2064}', '\u{206A}', '\u{206B}', '\u{206C}', '\u{206D}', '\u{206E}', '\u{206F}', '\u{FEFF}',
    '\u{FFF9}', '\u{FFFA}', '\u{FFFB}',
];

/// Belongs to _touchHLE_NSCharacterSet
struct CharacterSetHostObject {
    set: HashSet<unichar>,
    inverted: bool,
}
impl HostObject for CharacterSetHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSCharacterSet is an abstract class. A subclass must provide:
// - (bool)characterIsMember:(unichar)character
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSCharacterSet.
@implementation NSCharacterSet: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSCharacterSet might be subclassed by something which needs
    // allocWithZone: to have the normal behaviour. Unimplemented: call
    // superclass alloc then.
    assert!(this == env.objc.get_known_class("NSCharacterSet", &mut env.mem));
    msg_class![env; _touchHLE_NSCharacterSet allocWithZone:zone]
}

// This doesn't have a corresponding init method for some reason.
+ (id)characterSetWithCharactersInString:(id)string { // NSString*
    let mut set = HashSet::new();
    ns_string::for_each_code_unit(env, string, |_idx, c| { set.insert(c); });

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)characterSetWithRange:(NSRange)range {
    let mut set = HashSet::new();

    for c in range.location..range.location.checked_add(range.length).unwrap() {
        set.insert(unichar::try_from(c).unwrap());
    }

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)alphanumericCharacterSet {
    // NSCharacterSet operates on UTF-16 code units. Build the set from every
    // valid Unicode scalar in the Basic Multilingual Plane; surrogate code
    // units are deliberately excluded because they are not characters on
    // their own.
    let set = (0..=u16::MAX)
        .filter(|&code_unit| {
            char::from_u32(code_unit.into()).is_some_and(char::is_alphanumeric)
        })
        .collect();

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)decimalDigitCharacterSet {
    let set = DECIMAL_DIGIT_RANGES
        .iter()
        .flat_map(|&(start, end)| start..=end)
        .collect();

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)newlineCharacterSet {
    let set = HashSet::from(NEWLINE_CHARACTERS.map(|c| unichar::try_from(c).unwrap()));

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)whitespaceCharacterSet {
    let set = HashSet::from(WHITESPACE_CHARACTERS.map(|c| unichar::try_from(c).unwrap()));

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)whitespaceAndNewlineCharacterSet {
    let set1 = HashSet::from(NEWLINE_CHARACTERS.map(|c| unichar::try_from(c).unwrap()));
    let set2 = HashSet::from(WHITESPACE_CHARACTERS.map(|c| unichar::try_from(c).unwrap()));
    let set = set1.union(&set2).copied().collect();

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

+ (id)controlCharacterSet {
    let set = HashSet::from(CONTROL_CHARACTERS.map(|c| unichar::try_from(c).unwrap()));

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new)
}

@end

// NSMutableCharacterSet defines no primitive methods. Subclasses must
// implement all methods declared by this class in addition to the
// primitives of NSCharacterSet. They must also implement mutableCopyWithZone:.
@implementation NSMutableCharacterSet: NSCharacterSet

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSMutableCharacterSet might be subclassed by something which needs
    // allocWithZone: to have the normal behaviour. Unimplemented: call
    // superclass alloc then.
    assert!(this == env.objc.get_known_class("NSMutableCharacterSet", &mut env.mem));
    msg_class![env; _touchHLE_NSMutableCharacterSet allocWithZone:zone]
}

@end

// Our private subclass that is the single implementation of NSCharacterSet for
// the time being.
@implementation _touchHLE_NSCharacterSet: NSCharacterSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(CharacterSetHostObject {
        set: HashSet::new(),
        inverted: false
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: initWithCoder:

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}
// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    todo!()
}

- (bool)characterIsMember:(unichar)code_unit {
    let host_object = env.objc.borrow::<CharacterSetHostObject>(this);
    host_object.set.contains(&code_unit) ^ host_object.inverted
}

- (id)invertedSet {
    let old_host_object = env.objc.borrow::<CharacterSetHostObject>(this);
    let host_object = Box::new(CharacterSetHostObject {
        set: old_host_object.set.clone(),
        inverted: !old_host_object.inverted
    });
    let class = env.objc.get_known_class("_touchHLE_NSCharacterSet", &mut env.mem);
    env.objc.alloc_object(class, host_object, &mut env.mem)
}

@end

// Our private subclass that is the single implementation of
// NSMutableCharacterSet for the time being.
@implementation _touchHLE_NSMutableCharacterSet: NSMutableCharacterSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(CharacterSetHostObject {
        set: HashSet::new(),
        inverted: false
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    todo!()
}
// NSMutableCopying implementation
- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    todo!()
}

- (bool)characterIsMember:(unichar)code_unit {
    let host_object = env.objc.borrow::<CharacterSetHostObject>(this);
    host_object.set.contains(&code_unit) ^ host_object.inverted
}

- (())addCharactersInString:(id)string { // NSString *
    assert!(!env.objc.borrow::<CharacterSetHostObject>(this).inverted); // TODO
    let length: NSUInteger = msg![env; string length];
    for i in 0..length {
        let c = msg![env; string characterAtIndex:i];
        env.objc.borrow_mut::<CharacterSetHostObject>(this).set.insert(c);
    }
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::msg_class;

    #[test]
    fn decimal_digit_set_contains_unicode_decimal_digits_only() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let set: id = msg_class![env; NSCharacterSet decimalDigitCharacterSet];

        let ascii: bool = msg![env; set characterIsMember:('5' as unichar)];
        let arabic_indic: bool = msg![env; set characterIsMember:(0x0665u16)];
        let roman_numeral: bool = msg![env; set characterIsMember:(0x2165u16)];
        assert!(ascii);
        assert!(arabic_indic);
        assert!(!roman_numeral);
    }
}
