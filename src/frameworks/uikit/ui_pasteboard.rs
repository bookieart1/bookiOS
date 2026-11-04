/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIPasteboard`.

use crate::dyld::HostConstant;
use crate::frameworks::foundation::{ns_string, NSInteger, NSUInteger};
use crate::objc::{
    id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};
use std::collections::HashMap;

const UIPASTEBOARD_NAME_GENERAL: &str = "com.apple.UIKit.pboard.general";
const UIPASTEBOARD_NAME_FIND: &str = "com.apple.UIKit.pboard.find";

pub const CONSTANTS: &[(&str, HostConstant)] = &[
    (
        "_UIPasteboardNameGeneral",
        HostConstant::NSString(UIPASTEBOARD_NAME_GENERAL),
    ),
    (
        "_UIPasteboardNameFind",
        HostConstant::NSString(UIPASTEBOARD_NAME_FIND),
    ),
];

#[derive(Default)]
pub struct State {
    named: HashMap<String, id>,
    unique_counter: u64,
}

#[derive(Debug)]
struct PasteboardHostObject {
    name: id,
    string: id,
    /// Retained NSData values keyed by their pasteboard type string.
    data: HashMap<String, id>,
    persistent: bool,
    change_count: NSUInteger,
}
impl HostObject for PasteboardHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIPasteboard: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(PasteboardHostObject {
            name: nil,
            string: nil,
            data: HashMap::new(),
            persistent: false,
            change_count: 0,
        }),
        &mut env.mem,
    )
}

+ (id)pasteboardWithName:(id)name create:(bool)create {
    if name == nil {
        return nil;
    }
    let key = ns_string::to_rust_string(env, name).into_owned();
    if let Some(&pasteboard) = env.framework_state.uikit.ui_pasteboard.named.get(&key) {
        return pasteboard;
    }
    if !create {
        return nil;
    }

    let pasteboard: id = msg![env; this alloc];
    let pasteboard: id = msg![env; pasteboard initWithName:name];
    // The registry owns the allocation. Callers receive a non-owning,
    // singleton-style reference, as they do on iPhone OS.
    env.framework_state
        .uikit
        .ui_pasteboard
        .named
        .insert(key, pasteboard);
    pasteboard
}

+ (id)generalPasteboard {
    let name = ns_string::get_static_str(env, UIPASTEBOARD_NAME_GENERAL);
    let pasteboard: id = msg![env; this pasteboardWithName:name create:true];
    env.objc.borrow_mut::<PasteboardHostObject>(pasteboard).persistent = true;
    pasteboard
}

+ (id)pasteboardWithUniqueName {
    let counter = env.framework_state.uikit.ui_pasteboard.unique_counter;
    env.framework_state.uikit.ui_pasteboard.unique_counter = counter.wrapping_add(1);
    let name = ns_string::from_rust_string(env, format!("com.touchHLE.pboard.{counter}"));
    let pasteboard: id = msg![env; this pasteboardWithName:name create:true];
    release(env, name);
    pasteboard
}

+ (())removePasteboardWithName:(id)name {
    if name == nil {
        return;
    }
    let key = ns_string::to_rust_string(env, name).into_owned();
    if let Some(pasteboard) = env.framework_state.uikit.ui_pasteboard.named.remove(&key) {
        release(env, pasteboard);
    }
}

- (id)initWithName:(id)name {
    retain(env, name);
    env.objc.borrow_mut::<PasteboardHostObject>(this).name = name;
    this
}

- (id)name {
    env.objc.borrow::<PasteboardHostObject>(this).name
}

- (bool)isPersistent {
    env.objc.borrow::<PasteboardHostObject>(this).persistent
}

- (())setPersistent:(bool)persistent {
    env.objc.borrow_mut::<PasteboardHostObject>(this).persistent = persistent;
}

- (NSInteger)changeCount {
    env.objc.borrow::<PasteboardHostObject>(this).change_count as NSInteger
}

- (id)string {
    env.objc.borrow::<PasteboardHostObject>(this).string
}

- (())setString:(id)string {
    if string != nil {
        retain(env, string);
    }
    let old = {
        let host = env.objc.borrow_mut::<PasteboardHostObject>(this);
        let old = std::mem::replace(&mut host.string, string);
        host.change_count = host.change_count.wrapping_add(1);
        old
    };
    if old != nil {
        release(env, old);
    }
}

- (id)strings {
    let string = env.objc.borrow::<PasteboardHostObject>(this).string;
    if string == nil {
        nil
    } else {
        msg_class![env; NSArray arrayWithObject:string]
    }
}

- (())setStrings:(id)strings {
    let count: NSUInteger = if strings == nil { 0 } else { msg![env; strings count] };
    let first: id = if count == 0 { nil } else { msg![env; strings objectAtIndex:0u32] };
    () = msg![env; this setString:first];
}

- (id)dataForPasteboardType:(id)pasteboard_type {
    if pasteboard_type == nil {
        return nil;
    }
    let key = ns_string::to_rust_string(env, pasteboard_type).into_owned();
    env.objc
        .borrow::<PasteboardHostObject>(this)
        .data
        .get(&key)
        .copied()
        .unwrap_or(nil)
}

// Historical UIPasteboard also exposed the typed value directly.  The
// currently modelled pasteboard representation is NSData, so this is the same
// retained value returned non-owningly rather than an invented conversion.
- (id)valueForPasteboardType:(id)pasteboard_type {
    msg![env; this dataForPasteboardType:pasteboard_type]
}

- (())setData:(id)data forPasteboardType:(id)pasteboard_type {
    assert_ne!(pasteboard_type, nil, "UIPasteboard type must not be nil");
    assert_ne!(data, nil, "UIPasteboard data must not be nil");
    let key = ns_string::to_rust_string(env, pasteboard_type).into_owned();
    retain(env, data);
    let old = {
        let host = env.objc.borrow_mut::<PasteboardHostObject>(this);
        host.change_count = host.change_count.wrapping_add(1);
        host.data.insert(key, data)
    };
    if let Some(old) = old {
        release(env, old);
    }
}

- (bool)containsPasteboardTypes:(id)pasteboard_types {
    if pasteboard_types == nil {
        return false;
    }
    let count: NSUInteger = msg![env; pasteboard_types count];
    for index in 0..count {
        let pasteboard_type: id = msg![env; pasteboard_types objectAtIndex:index];
        if pasteboard_type == nil {
            continue;
        }
        let key = ns_string::to_rust_string(env, pasteboard_type).into_owned();
        let host = env.objc.borrow::<PasteboardHostObject>(this);
        if host.data.contains_key(&key) {
            return true;
        }
        if host.string != nil
            && matches!(
                key.as_str(),
                "public.text" | "public.plain-text" | "public.utf8-plain-text"
            )
        {
            return true;
        }
    }
    false
}

- (())dealloc {
    let (name, string, data) = {
        let host = env.objc.borrow_mut::<PasteboardHostObject>(this);
        (host.name, host.string, std::mem::take(&mut host.data))
    };
    if name != nil {
        release(env, name);
    }
    if string != nil {
        release(env, string);
    }
    for value in data.into_values() {
        release(env, value);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn named_pasteboards_obey_create_identity_and_string_state() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let name = ns_string::from_rust_string(env, "test.board".to_owned());
        let missing: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(missing, nil);
        let created: id = msg_class![env; UIPasteboard pasteboardWithName:name create:true];
        let found: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(created, found);

        let value = ns_string::from_rust_string(env, "hello".to_owned());
        () = msg![env; created setString:value];
        let actual: id = msg![env; found string];
        assert_eq!(ns_string::to_rust_string(env, actual), "hello");
        let change_count: NSInteger = msg![env; found changeCount];
        assert_eq!(change_count, 1);

        () = msg_class![env; UIPasteboard removePasteboardWithName:name];
        let missing_again: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(missing_again, nil);
        release(env, value);
        release(env, name);
    }

    #[test]
    fn typed_data_round_trips_and_increments_change_count() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let board: id = msg_class![env; UIPasteboard pasteboardWithUniqueName];
        let data: id = msg_class![env; NSData data];
        let pasteboard_type = ns_string::from_rust_string(env, "com.example.test".to_owned());
        () = msg![env; board setData:data forPasteboardType:pasteboard_type];
        let actual: id = msg![env; board dataForPasteboardType:pasteboard_type];
        assert_eq!(actual, data);
        let value: id = msg![env; board valueForPasteboardType:pasteboard_type];
        assert_eq!(value, data);
        let change_count: NSInteger = msg![env; board changeCount];
        assert_eq!(change_count, 1);
        release(env, pasteboard_type);
    }

    #[test]
    fn contains_pasteboard_types_matches_typed_and_string_values() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let board: id = msg_class![env; UIPasteboard pasteboardWithUniqueName];
        let value = ns_string::from_rust_string(env, "hello".to_owned());
        () = msg![env; board setString:value];
        let text_type = ns_string::from_rust_string(env, "public.utf8-plain-text".to_owned());
        let types: id = msg_class![env; NSArray arrayWithObject:text_type];
        assert!(msg![env; board containsPasteboardTypes:types]);
        release(env, text_type);
        release(env, value);
    }
}
