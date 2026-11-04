/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSObject`, the root of most class hierarchies in Objective-C.
//!
//! Resources:
//! - Apple's [Advanced Memory Management Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html)
//!   explains how reference counting works. Note that we are interested in what
//!   it calls "manual retain-release", not ARC.
//! - Apple's [Key-Value Coding Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/SearchImplementation.html)
//!   explains the algorithm `setValue:forKey:` should follow.
//!
//! See also: [crate::objc], especially the `objects` module.

use super::ns_string::{from_rust_string, to_rust_string};
use super::{ns_key_value_observing, NSTimeInterval, NSUInteger};
use crate::abi::GuestFunction;
use crate::cpu::Cpu;
use crate::frameworks::foundation::ns_run_loop::{
    add_perform_request, cancel_perform_requests, run_loop_for_thread,
};
use crate::frameworks::foundation::ns_thread::{
    detach_new_thread_inner, thread_id_for_ns_thread,
};
use crate::libc::semaphore::{host_destroy_semaphore, sem_wait};
use crate::mem::MutVoidPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, msg_send_no_type_checking, nil, objc_classes,
    release, retain, Class, ClassExports, NSZonePtr, ObjC, TrivialHostObject, SEL, TYPE_BOOL,
    TYPE_CHAR,
    TYPE_CHAR_PTR, TYPE_CLASS, TYPE_DOUBLE, TYPE_FLOAT, TYPE_ID, TYPE_INT, TYPE_LONG,
    TYPE_LONGLONG, TYPE_SEL, TYPE_SHORT, TYPE_UCHAR, TYPE_UINT, TYPE_ULONG, TYPE_ULONGLONG,
    TYPE_UNDEF, TYPE_USHORT, TYPE_VOID,
};
use crate::Environment;

/// Decode the top-level Objective-C type tokens in a method encoding while
/// ignoring stack offsets and the contents of compound types.  Old games use
/// both compact encodings (v@:@) and offset-bearing encodings (v12@0:4i8).
fn method_type_tokens(types: &str) -> Vec<char> {
    fn skip_balanced(bytes: &[u8], index: &mut usize, open: u8, close: u8) {
        let mut depth = 1usize;
        while *index < bytes.len() && depth != 0 {
            match bytes[*index] {
                b'"' => {
                    *index += 1;
                    while *index < bytes.len() && bytes[*index] != b'"' {
                        *index += 1;
                    }
                }
                value if value == open => depth += 1,
                value if value == close => depth -= 1,
                _ => {}
            }
            *index += 1;
        }
    }

    fn consume_type(bytes: &[u8], index: &mut usize) -> Option<char> {
        while *index < bytes.len() && b"rnNoORV".contains(&bytes[*index]) {
            *index += 1;
        }
        let first = *bytes.get(*index)?;
        *index += 1;
        match first {
            b'@' => {
                if bytes.get(*index) == Some(&b'?') {
                    *index += 1;
                } else if bytes.get(*index) == Some(&b'"') {
                    *index += 1;
                    while *index < bytes.len() && bytes[*index] != b'"' {
                        *index += 1;
                    }
                    *index += usize::from(*index < bytes.len());
                }
            }
            b'^' => {
                let _ = consume_type(bytes, index);
            }
            b'[' => skip_balanced(bytes, index, b'[', b']'),
            b'{' => skip_balanced(bytes, index, b'{', b'}'),
            b'(' => skip_balanced(bytes, index, b'(', b')'),
            b'b' => {
                while *index < bytes.len() && bytes[*index].is_ascii_digit() {
                    *index += 1;
                }
            }
            _ => {}
        }
        Some(first as char)
    }

    let bytes = types.as_bytes();
    let mut index = 0usize;
    let mut tokens = Vec::new();
    while index < bytes.len() {
        while index < bytes.len()
            && (bytes[index].is_ascii_digit()
                || matches!(bytes[index], b'+' | b'-' | b' ' | b'\t'))
        {
            index += 1;
        }
        if let Some(token) = consume_type(bytes, &mut index) {
            tokens.push(token);
        }
    }
    tokens
}

/// The `NSMethodSignature*` of an instance method of `class`, or nil if
/// there's no such method.
fn method_signature(env: &mut Environment, class: Class, sel: SEL) -> id {
    if !env.objc.class_has_method(class, sel) {
        return nil;
    }
    let sig = if let Some(&sig) = env.objc.class_get_method_signature(class, sel) {
        sig
    } else {
        // Host methods carry a compact type encoding generated from their Rust
        // signature. NSMethodSignature copies the encoding, so this temporary
        // guest C string can be released immediately after construction.
        let Some(types) = env.objc.class_get_method_types(class, sel).map(str::to_owned) else {
            return nil;
        };
        let sig = env.mem.alloc_and_write_cstr(types.as_bytes()).cast_const();
        let result = msg_class![env; NSMethodSignature signatureWithObjCTypes:sig];
        env.mem.free(sig.cast_mut().cast());
        return result;
    };
    log_dbg!(
        "method signature: '{}' -> {:?}",
        sel.as_str(&env.mem),
        env.mem.cstr_at_utf8(sig)
    );
    msg_class![env; NSMethodSignature signatureWithObjCTypes:sig]
}

fn perform_selector_on_thread(
    env: &mut Environment,
    receiver: id,
    sel: SEL,
    thread: id,
    arg: id,
    wait: bool,
) {
    log_dbg!(
        "performSelector:{} onThread:{:?} withObject:{:?} waitUntilDone:{}",
        sel.as_str(&env.mem),
        thread,
        arg,
        wait
    );

    let thread_id = thread_id_for_ns_thread(env, thread)
        .expect("performSelector:onThread: target NSThread has not been started");
    if wait && thread_id == env.current_thread {
        if sel.as_str(&env.mem).ends_with(':') {
            () = msg_send_no_type_checking(env, (receiver, sel, arg));
        } else {
            () = msg_send(env, (receiver, sel));
        }
        return;
    }

    let run_loop_class = env.objc.get_known_class("NSRunLoop", &mut env.mem);
    let run_loop = run_loop_for_thread(env, run_loop_class, thread_id);
    let sem = add_perform_request(env, run_loop, receiver, sel, arg, None, wait);
    if wait {
        sem_wait(env, sem);
        host_destroy_semaphore(env, sem);
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSObject

+ (id)alloc {
    msg![env; this allocWithZone:(MutVoidPtr::null())]
}
+ (id)allocWithZone:(NSZonePtr)_zone { // struct _NSZone*
    log_dbg!("[{:?} allocWithZone:]", this);
    env.objc.alloc_object(this, Box::new(TrivialHostObject), &mut env.mem)
}

+ (id)new {
    let new_object: id = msg![env; this alloc];
    msg![env; new_object init]
}

+ (Class)class {
    this
}
+ (Class)superclass {
    env.objc.get_superclass(this)
}
+ (id)self {
    this
}
+ (bool)isSubclassOfClass:(Class)class {
    env.objc.class_is_subclass_of(this, class)
}
+ (bool)conformsToProtocol:(crate::mem::ConstVoidPtr)protocol {
    env.objc.class_conforms_to_protocol(this, protocol, &env.mem)
}

// See the instance method section for the normal versions of these.
+ (id)retain {
    this // classes are not refcounted
}
+ (())release {
    // classes are not refcounted
}
+ (())autorelease {
    // classes are not refcounted
}

// Class objects are immutable singleton objects. Foundation collections copy
// keys through NSCopying, so NSObject supplies this class-side implementation.
+ (id)copyWithZone:(NSZonePtr)_zone {
    this
}

+ (bool)instancesRespondToSelector:(SEL)selector {
    env.objc.class_has_method(this, selector)
}

+ (())cancelPreviousPerformRequestsWithTarget:(id)target selector:(SEL)selector object:(id)arg {
    let run_loop: id = msg_class![env; NSRunLoop currentRunLoop];
    cancel_perform_requests(env, run_loop, target, selector, arg);
}

+ (bool)accessInstanceVariablesDirectly {
    true
}

+ (id)description {
    let name = env.objc.get_class_name(this);
    let str = from_rust_string(env, name.to_string());
    autorelease(env, str)
}

+ (id)debugDescription {
    msg![env; this description]
}

+ (id)instanceMethodSignatureForSelector:(SEL)sel {
    method_signature(env, this, sel)
}

+ (GuestFunction)instanceMethodForSelector:(SEL)selector {
    ObjC::class_get_method_guest_function(env, this, selector)
}

+ (())initialize {
    // Do nothing
}

+ (())load {
    // NSObject's implementation is effectively a no-op. A few older
    // frameworks explicitly call [super load] from their own +load method.
}

- (id)init {
    this
}

- (NSUInteger)retainCount {
    if env.objc.has_static_lifetime(this) {
        NSUInteger::MAX
    } else {
        env.objc.get_refcount(this).into()
    }
}

- (id)retain {
    log_dbg!("[{:?} retain]", this);
    if !env.objc.has_static_lifetime(this) {
        let before = env.objc.refcount_for_diagnostics(this).unwrap_or(0);
        env.objc.increment_refcount(this, &env.mem, env.current_thread);
        let after = env.objc.refcount_for_diagnostics(this).unwrap_or(0);
        env.diagnostic_object_lifetime("retain", this, before, after);
    }
    this
}
- (())release {
    log_dbg!("[{:?} release]", this);
    let regs = env.cpu.regs();
    let (pc, lr) = (regs[Cpu::PC], regs[Cpu::LR]);
    if !env.objc.has_static_lifetime(this) {
        let before = env.objc.refcount_for_diagnostics(this).unwrap_or(0);
        let reached_zero = env
            .objc
            .decrement_refcount(this, env.current_thread, pc, lr);
        let after = env.objc.refcount_for_diagnostics(this).unwrap_or(0);
        env.diagnostic_object_lifetime(
            if reached_zero { "release-to-zero" } else { "release" },
            this,
            before,
            after,
        );
        if reached_zero {
            () = msg![env; this dealloc];
        }
    }
}
- (id)autorelease {
    if !env.objc.has_static_lifetime(this) {
        let refcount = env.objc.refcount_for_diagnostics(this).unwrap_or(0);
        env.diagnostic_object_lifetime("autorelease-scheduled", this, refcount, refcount);
        () = msg_class![env; NSAutoreleasePool addObject:this];
    }
    this
}

- (())dealloc {
    log_dbg!("[{:?} dealloc]", this);
    env.diagnostic_object_lifetime("dealloc", this, 0, 0);
    ns_key_value_observing::forget_object(env, this);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (Class)class {
    ObjC::read_isa(this, &env.mem)
}
- (Class)superclass {
    let class = ObjC::read_isa(this, &env.mem);
    env.objc.get_superclass(class)
}
- (bool)isMemberOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    class == this_class
}
- (bool)isKindOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    env.objc.class_is_subclass_of(this_class, class)
}
- (bool)conformsToProtocol:(crate::mem::ConstVoidPtr)protocol {
    let class = ObjC::read_isa(this, &env.mem);
    env.objc.class_conforms_to_protocol(class, protocol, &env.mem)
}

- (NSUInteger)hash {
    this.to_bits()
}

// To not confuse with isEqualTo:, which is
// a category of NSWhoseSpecifier!
// Reference https://nshipster.com/equality
- (bool)isEqual:(id)other {
    this == other
}

- (id)description {
    let class = ObjC::read_isa(this, &env.mem);
    let name = env.objc.get_class_name(class);
    let description = format!("<{name}: {this:?}>");
    let string = from_rust_string(env, description);
    autorelease(env, string)
}

- (id)debugDescription {
    msg![env; this description]
}

// TODO: localized description methods also? (not sure if NSObject has them)

// Helper for NSCopying
- (id)copy {
    msg![env; this copyWithZone:(MutVoidPtr::null())]
}

// Helper for NSMutableCopying
- (id)mutableCopy {
    msg![env; this mutableCopyWithZone:(MutVoidPtr::null())]
}

// NSKeyValueCoding
// https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/SearchImplementation.html
- (id)valueForKey:(id)key { // NSString*
    let key_string = to_rust_string(env, key); // TODO: avoid copy?
    assert!(key_string.is_ascii()); // TODO: do we have to handle non-ASCII keys?
    if key_string.is_empty() {
        let sel = env.objc.lookup_selector("valueForUndefinedKey:").unwrap();
        return msg_send(env, (this, sel, key));
    }
    let camel_case_key_string = format!("{}{}", key_string.as_bytes()[0].to_ascii_uppercase() as char, &key_string[1..]);
    let class: Class = msg![env; this class];

    // Search the standard KVC getter names in order.
    let getter = [
        format!("get{camel_case_key_string}"),
        key_string.to_string(),
        format!("is{camel_case_key_string}"),
        format!("_{key_string}"),
    ].into_iter().find_map(|name| {
        env.objc.lookup_selector(&name).filter(|&sel| env.objc.class_has_method(class, sel))
    });

    if let Some(sel) = getter {
        let return_type = env.objc.class_get_method_types(class, sel)
            .and_then(|types| types.chars().next())
            .unwrap_or(TYPE_UNDEF);
        return match return_type {
            TYPE_ID | TYPE_CLASS => msg_send(env, (this, sel)),
            TYPE_BOOL => {
                let value: bool = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithBool:value]
            },
            TYPE_CHAR => {
                let value: i8 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithChar:value]
            },
            TYPE_UCHAR => {
                let value: u8 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithUnsignedInt:(value as u32)]
            },
            TYPE_SHORT => {
                let value: i16 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithShort:value]
            },
            TYPE_USHORT => {
                let value: u16 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithUnsignedShort:value]
            },
            TYPE_INT | TYPE_LONG => {
                let value: i32 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithInt:value]
            },
            TYPE_UINT | TYPE_ULONG => {
                let value: u32 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithUnsignedInt:value]
            },
            TYPE_LONGLONG => {
                let value: i64 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithLongLong:value]
            },
            TYPE_ULONGLONG => {
                let value: u64 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithUnsignedLongLong:value]
            },
            TYPE_FLOAT => {
                let value: f32 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithFloat:value]
            },
            TYPE_DOUBLE => {
                let value: f64 = msg_send(env, (this, sel));
                msg_class![env; NSNumber numberWithDouble:value]
            },
            _ => unimplemented!("valueForKey: getter {} has unsupported return type {}", sel.as_str(&env.mem), return_type),
        };
    }

    // With direct ivar access enabled, use the standard KVC ivar search order.
    let access_sel = env.objc.lookup_selector("accessInstanceVariablesDirectly").unwrap();
    let access_ivars: bool = msg_send(env, (class, access_sel));
    if access_ivars {
        let ivar = env.objc.object_lookup_ivar(&env.mem, this, &format!("_{key_string}"))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &format!("_is{camel_case_key_string}")))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &key_string.to_string()))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &format!("is{camel_case_key_string}")))
            .map(|(ivar_ptr, ivar_type)| (ivar_ptr, ivar_type.to_owned()));
        if let Some((ivar_ptr, ivar_type)) = ivar {
            return match ivar_type.chars().next().unwrap_or(TYPE_UNDEF) {
                TYPE_ID | TYPE_CLASS => {
                    let value: id = env.mem.read(ivar_ptr.cast());
                    value
                },
                TYPE_BOOL => {
                    let value: bool = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithBool:value]
                },
                TYPE_CHAR => {
                    let value: i8 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithChar:value]
                },
                TYPE_UCHAR => {
                    let value: u8 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithUnsignedInt:(value as u32)]
                },
                TYPE_SHORT => {
                    let value: i16 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithShort:value]
                },
                TYPE_USHORT => {
                    let value: u16 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithUnsignedShort:value]
                },
                TYPE_INT | TYPE_LONG => {
                    let value: i32 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithInt:value]
                },
                TYPE_UINT | TYPE_ULONG => {
                    let value: u32 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithUnsignedInt:value]
                },
                TYPE_LONGLONG => {
                    let value: i64 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithLongLong:value]
                },
                TYPE_ULONGLONG => {
                    let value: u64 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithUnsignedLongLong:value]
                },
                TYPE_FLOAT => {
                    let value: f32 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithFloat:value]
                },
                TYPE_DOUBLE => {
                    let value: f64 = env.mem.read(ivar_ptr.cast());
                    msg_class![env; NSNumber numberWithDouble:value]
                },
                ivar_type => unimplemented!("valueForKey: ivar {} has unsupported type {}", key_string, ivar_type),
            };
        }
    }

    let sel = env.objc.lookup_selector("valueForUndefinedKey:").unwrap();
    msg_send(env, (this, sel, key))
}

- (id)valueForKeyPath:(id)key_path { // NSString*
    // A key path is evaluated one key at a time. This is deliberately built
    // on valueForKey: so dictionaries, accessor methods, scalar boxing and
    // direct-ivar lookup all keep their existing KVC semantics.
    let key_path = to_rust_string(env, key_path).into_owned();
    let mut value = this;
    for component in key_path.split('.') {
        if value == nil {
            break;
        }
        let key = from_rust_string(env, component.to_owned());
        value = msg![env; value valueForKey:key];
        release(env, key);
    }
    value
}

- (())setValue:(id)value
 forKeyPath:(id)key_path { // NSString*
    // Follow Foundation's default KVC contract: resolve every relationship
    // with valueForKey:, then apply the existing setValue:forKey: behavior to
    // the final object. This keeps accessor lookup, scalar unboxing, direct
    // ivar access, and undefined-key handling in one place.
    let key_path = to_rust_string(env, key_path).into_owned();
    let mut components = key_path.split('.').peekable();
    let mut destination = this;
    while let Some(component) = components.next() {
        let key = from_rust_string(env, component.to_owned());
        if components.peek().is_some() {
            destination = msg![env; destination valueForKey:key];
        } else {
            () = msg![env; destination setValue:value forKey:key];
        }
        release(env, key);
    }
}

- (id)dictionaryWithValuesForKeys:(id)keys { // NSArray*
    let count: NSUInteger = msg![env; keys count];
    let dictionary: id = msg_class![env; NSMutableDictionary dictionaryWithCapacity:count];
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        let mut value: id = msg![env; this valueForKey:key];
        if value == nil {
            value = msg_class![env; NSNull null];
        }
        () = msg![env; dictionary setObject:value forKey:key];
    }
    dictionary
}

- (())setValuesForKeysWithDictionary:(id)keyed_values { // NSDictionary*
    // Objective-C messages sent to nil return zero/nil, so the equivalent
    // allKeys/count sequence is an intentional no-op for a nil argument.
    if keyed_values == nil {
        return;
    }
    // allKeys is a snapshot, so this remains well-defined even if the source
    // and receiver are the same mutable dictionary.
    let keys: id = msg![env; keyed_values allKeys];
    let count: NSUInteger = msg![env; keys count];
    for index in 0..count {
        let key: id = msg![env; keys objectAtIndex:index];
        let value: id = msg![env; keyed_values objectForKey:key];
        () = msg![env; this setValue:value forKey:key];
    }
}

- (id)valueForUndefinedKey:(id)key { // NSString*
    // TODO: Raise NSUnknownKeyException.
    let class: Class = ObjC::read_isa(this, &env.mem);
    let class_name_string = env.objc.get_class_name(class).to_owned();
    let key_string = to_rust_string(env, key);
    panic!("Object {:?} of class {:?} ({:?}) does not have a value for {} ({:?})",
        this, class_name_string, class, key_string, key);
}

- (())setValue:(id)value
       forKey:(id)key { // NSString*
    let key_string = to_rust_string(env, key); // TODO: avoid copy?
    assert!(key_string.is_ascii()); // TODO: do we have to handle non-ASCII keys?
    let camel_case_key_string = format!("{}{}", key_string.as_bytes()[0].to_ascii_uppercase() as char, &key_string[1..]);

    let class = msg![env; this class];

    // TODO: If value is nil, the target ivar/method argument type must be
    // checked. If it's non-object type, invoke setNilValueForKey:
    // Object properties and setValue:forUndefinedKey: take nil as it is.
    let is_ns_value = value != nil && {
        let value_class = msg![env; value class];
        let ns_value_class = env.objc.get_known_class("NSValue", &mut env.mem);
        env.objc.class_is_subclass_of(value_class, ns_value_class)
    };

    // Look for the first accessor named set<Key>: or _set<Key>, in that order.
    // If found, invoke it with the input value (or unwrapped value, as needed)
    // and finish.
    if let Some(sel) = env.objc.lookup_selector(&format!("set{camel_case_key_string}:")).filter(|&sel| env.objc.class_has_method(class, sel))
        .or_else(|| env.objc.lookup_selector(&format!("_set{camel_case_key_string}:"))).filter(|&sel| env.objc.class_has_method(class, sel)
    ) {
        if is_ns_value {
            // If value is a NSValue, it must be unwrapped
            // Find the selector's first (and only) argument type:
            let types = env.objc.get_class_method(class, sel).types.clone();
            let tokens = method_type_tokens(&types);
            assert_eq!(tokens.first(), Some(&TYPE_VOID));
            assert!(matches!(tokens.get(1), Some(&TYPE_ID | &TYPE_CLASS)));
            assert_eq!(tokens.get(2), Some(&TYPE_SEL));
            let argument_type = tokens.get(3).copied();
            assert_eq!(tokens.len(), 4, "KVC setter must have exactly one explicit argument");

            match argument_type.unwrap() {
                TYPE_BOOL => {
                    let value: bool = msg![env; value boolValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_DOUBLE => {
                    let value: f64 = msg![env; value doubleValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_FLOAT => {
                    let value: f32 = msg![env; value floatValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_INT => {
                    let value: i32 = msg![env; value intValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_LONGLONG => {
                    let value: i64 = msg![env; value longLongValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_UINT => {
                    let value: u32 = msg![env; value unsignedIntValue];
                    () = msg_send(env, (this, sel, value));
                },
                TYPE_ULONGLONG => {
                    let value: u64 = msg![env; value unsignedLongLongValue];
                    () = msg_send(env, (this, sel, value));
                },
                argument_type => unimplemented!("TODO: [(NSObject*){:?} setValue:{:?} forKey:{:?}]: Received an NSValue which must be unwrapped to type {}", this, value, key, argument_type)
            };
        } else {
            () = msg_send(env, (this, sel, value));
        }
        return;
    }

    // If no simple accessor is found, and if the class method
    // accessInstanceVariablesDirectly returns YES, look for an instance
    // variable with a name like _<key>, _is<Key>, <key>, or is<Key>,
    // in that order.
    // If found, set the variable directly with the input value
    // (or unwrapped value) and finish.
    let sel = env.objc.lookup_selector("accessInstanceVariablesDirectly").unwrap();
    let accessInstanceVariablesDirectly = msg_send(env, (class, sel));
    if accessInstanceVariablesDirectly {
        if let Some((ivar_ptr, ivar_type)) = env.objc.object_lookup_ivar(&env.mem, this, &format!("_{key_string}"))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &format!("_is{camel_case_key_string}")))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &format!("{key_string}")))
            .or_else(|| env.objc.object_lookup_ivar(&env.mem, this, &format!("is{camel_case_key_string}"))
        ) {
            if is_ns_value {
                // If value is a NSValue, it must be unwrapped
                match ivar_type.chars().next().unwrap() {
                    TYPE_BOOL => {
                        let value: bool = msg![env; value boolValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_DOUBLE => {
                        let value: f64 = msg![env; value doubleValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_FLOAT => {
                        let value: f32 = msg![env; value floatValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_INT => {
                        let value: i32 = msg![env; value intValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_LONGLONG => {
                        let value: i64 = msg![env; value longLongValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_UINT => {
                        let value: u32 = msg![env; value unsignedIntValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    TYPE_ULONGLONG => {
                        let value: u64 = msg![env; value unsignedLongLongValue];
                        env.mem.write(ivar_ptr.cast(), value);
                    },
                    _ => unimplemented!("TODO: [(NSObject*){:?} setValue:{:?} forKey:{:?}]: Received an NSValue which must be unwrapped to type {}", this, value, key, ivar_type)
                };
            } else {
                retain(env, value);
                env.mem.write(ivar_ptr.cast(), value);
            };
            return;
        }
    }

    // Upon finding no accessor or instance variable,
    // invoke setValue:forUndefinedKey:.
    // This raises an exception by default, but a subclass of NSObject
    // may provide key-specific behavior.
    let sel = env.objc.lookup_selector("setValue:forUndefinedKey:").unwrap();
    () = msg_send(env, (this, sel, value, key));
}

- (())setValue:(id)_value
forUndefinedKey:(id)key { // NSString*
    // TODO: Raise NSUnknownKeyException
    let class: Class = ObjC::read_isa(this, &env.mem);
    let class_name_string = env.objc.get_class_name(class).to_owned(); // TODO: Avoid copying
    let key_string = to_rust_string(env, key);
    panic!("Object {:?} of class {:?} ({:?}) does not have a setter for {} ({:?})\
        \nAvailable selectors: {}\nAvailable ivars: {}",
        this, class_name_string, class, key_string, key,
        env.objc.debug_all_class_selectors_as_strings(&env.mem, class).join(", "),
        env.objc.debug_all_class_ivars_as_strings(class).join(", "));
}

// NSKeyValueObserving
- (())addObserver:(id)observer
       forKeyPath:(id)key_path // NSString *
          options:(NSUInteger)options
          context:(MutVoidPtr)context {
    ns_key_value_observing::add_observer(env, this, observer, key_path, options, context);
}
- (())removeObserver:(id)observer
          forKeyPath:(id)key_path { // NSString *
    ns_key_value_observing::remove_observer(env, this, observer, key_path, None);
}
- (())removeObserver:(id)observer
          forKeyPath:(id)key_path // NSString *
             context:(MutVoidPtr)context {
    ns_key_value_observing::remove_observer(env, this, observer, key_path, Some(context));
}
- (())willChangeValueForKey:(id)key { // NSString *
    ns_key_value_observing::will_change(env, this, key);
}
- (())didChangeValueForKey:(id)key { // NSString *
    ns_key_value_observing::did_change(env, this, key);
}

- (id)methodSignatureForSelector:(SEL)sel {
    let class = ObjC::read_isa(this, &env.mem);
    method_signature(env, class, sel)
}

- (bool)respondsToSelector:(SEL)selector {
    env.objc.object_has_method(&env.mem, this, selector)
}

- (id)performSelector:(SEL)sel {
    assert!(!sel.is_null());
    msg_send_no_type_checking(env, (this, sel))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1 {
    assert!(!sel.is_null());
    msg_send_no_type_checking(env, (this, sel, o1))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1
           withObject:(id)o2 {
    assert!(!sel.is_null());
    msg_send_no_type_checking(env, (this, sel, o1, o2))
}

- (())performSelectorInBackground:(SEL)sel
                       withObject:(id)arg {
    detach_new_thread_inner(env, sel, this, arg, /* tolerate_type_mismatch: */ true)
}

- (())performSelector:(SEL)sel withObject:(id)arg afterDelay:(NSTimeInterval)delay {
    let run_loop: id = msg_class![env; NSRunLoop currentRunLoop];
    add_perform_request(env, run_loop, this, sel, arg, Some(delay), false);
}

- (())performSelector:(SEL)sel
              onThread:(id)thread
            withObject:(id)arg
         waitUntilDone:(bool)wait {
    perform_selector_on_thread(env, this, sel, thread, arg, wait);
}

- (())performSelector:(SEL)sel
              onThread:(id)thread
            withObject:(id)arg
         waitUntilDone:(bool)wait
                 modes:(id)_modes {
    // The emulated run loop currently has a single effective mode, so all
    // accepted mode arrays map to that queue. Scheduling and wait semantics
    // are otherwise shared with the four-argument API.
    perform_selector_on_thread(env, this, sel, thread, arg, wait);
}

- (())performSelectorOnMainThread:(SEL)sel withObject:(id)arg waitUntilDone:(bool)wait {
    log_dbg!("performSelectorOnMainThread:{} withObject:{:?} waitUntilDone:{}", sel.as_str(&env.mem), arg, wait);
    if wait && env.current_thread == 0 {
        if sel.as_str(&env.mem).ends_with(':') {
            // Like performSelector:withObject:, the method may take a
            // non-object argument.
            () = msg_send_no_type_checking(env, (this, sel, arg));
        } else {
            // As on a device, a method without one ignores the argument.
            () = msg_send(env, (this, sel));
        }
        return;
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.POP") && (sel == env.objc.lookup_selector("startMovie:").unwrap() || sel == env.objc.lookup_selector("stopMovie").unwrap()) && wait {
        log!("Applying game-specific hack for PoP: WW: ignoring performSelectorOnMainThread:SEL({}) waitUntilDone:true", sel.as_str(&env.mem));
        return;
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.Asphalt5") && (sel == env.objc.lookup_selector("startMovie:").unwrap() || sel == env.objc.lookup_selector("stopMovie:").unwrap()) && wait {
        log!("Applying game-specific hack for Asphalt5: ignoring performSelectorOnMainThread:SEL({}) waitUntilDone:true", sel.as_str(&env.mem));
        return;
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.SplinterCell") && sel == env.objc.lookup_selector("startMovie:").unwrap() && wait {
        log!("Applying game-specific hack for SplinterCell: ignoring performSelectorOnMainThread:SEL({}) waitUntilDone:true", sel.as_str(&env.mem));
        return;
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.AssassinsCreed") && sel == env.objc.lookup_selector("moviePlayerInit:").unwrap() && wait {
        log!("Applying game-specific hack for AssassinsCreed: ignoring performSelectorOnMainThread:SEL(moviePlayerInit:) waitUntilDone:true");
        return;
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.Ferrari") && wait {
        if sel == env.objc.lookup_selector("startMovie:").unwrap() {
            log!("Applying game-specific hack for Ferrari GT: ignoring performSelectorOnMainThread:SEL({}) waitUntilDone:true", sel.as_str(&env.mem));
            return;
        }
        if sel == env.objc.lookup_selector("initTextInput:").unwrap() || sel == env.objc.lookup_selector("removeTextField:").unwrap() {
            log!("Applying game-specific hack for Ferrari GT: performing performSelectorOnMainThread:SEL({}) waitUntilDone:true on thread {}", sel.as_str(&env.mem), env.current_thread);
            () = msg_send(env, (this, sel, arg));
            return;
        }
    }
    if env.bundle.bundle_identifier().starts_with("com.gameloft.HOS2") && wait {
        if sel == env.objc.lookup_selector("loadMovie:").unwrap() || sel == env.objc.lookup_selector("sendGameInfo").unwrap() || sel == env.objc.lookup_selector("setStatusBar:").unwrap() {
            log!("Applying game-specific hack for HOS2: performing performSelectorOnMainThread:SEL({}) waitUntilDone:true on thread {}", sel.as_str(&env.mem), env.current_thread);
            if sel.as_str(&env.mem).ends_with(':') {
                () = msg_send(env, (this, sel, arg));
            } else {
                assert!(arg.is_null());
                () = msg_send(env, (this, sel));
            }
            return;
        }
        if sel == env.objc.lookup_selector("startMovie:").unwrap() || sel == env.objc.lookup_selector("stopMovie:").unwrap() {
            log!("Applying game-specific hack for HOS2: ignoring performSelectorOnMainThread:SEL({}) waitUntilDone:true", sel.as_str(&env.mem));
            return;
        }
    }

    let run_loop: id = msg_class![env; NSRunLoop mainRunLoop];
    let sem = add_perform_request(env, run_loop, this, sel, arg, None, wait);
    if wait {
        sem_wait(env, sem);
        host_destroy_semaphore(env, sem);
    }
}

- (GuestFunction)methodForSelector:(SEL)selector {
    let isa = ObjC::read_isa(this, &env.mem);
    ObjC::class_get_method_guest_function(env, isa, selector)
}

// UINibLoadingAdditions protocol
- (())awakeFromNib {
    // no-op
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::release;

    #[test]
    fn method_type_parser_handles_offsets_and_compound_arguments() {
        assert_eq!(method_type_tokens("v12@0:4i8"), vec!['v', '@', ':', 'i']);
        assert_eq!(method_type_tokens("v24@0:4{CGPoint=ff}8"), vec!['v', '@', ':', '{']);
        assert_eq!(method_type_tokens("v@:@\"NSString\""), vec!['v', '@', ':', '@']);
        assert_eq!(method_type_tokens("v16@0:4@?8"), vec!['v', '@', ':', '@']);
    }

    #[test]
    fn instance_description_is_inherited_without_replacing_overrides() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;

        let object: id = msg_class![env; NSObject new];
        let description: id = msg![env; object description];
        let description = to_rust_string(env, description);
        assert!(description.starts_with("<NSObject: 0x"));
        assert!(description.ends_with('>'));

        let string = from_rust_string(env, "override".to_string());
        let string_description: id = msg![env; string description];
        assert_eq!(string_description, string);

        release(env, object);
        release(env, string);
    }

    #[test]
    fn set_values_for_keys_with_dictionary_uses_normal_kvc_setters() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let source: id = msg_class![env; NSMutableDictionary new];
        let target: id = msg_class![env; NSMutableDictionary new];
        let key = from_rust_string(env, "score".to_owned());
        let value = from_rust_string(env, "100".to_owned());
        () = msg![env; source setObject:value forKey:key];

        () = msg![env; target setValuesForKeysWithDictionary:source];
        let copied: id = msg![env; target objectForKey:key];
        assert_eq!(copied, value);

        // A nil source has no keys and must leave the receiver unchanged.
        () = msg![env; target setValuesForKeysWithDictionary:nil];
        let still_copied: id = msg![env; target objectForKey:key];
        assert_eq!(still_copied, value);

        release(env, value);
        release(env, key);
        release(env, target);
        release(env, source);
    }

    #[test]
    fn class_objects_inherit_copy_with_zone_and_return_themselves() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let class = env
            .objc
            .get_known_class("NSMutableDictionary", &mut env.mem);

        let copied: id = msg![env; class copyWithZone:(MutVoidPtr::null())];
        assert_eq!(copied, class);
    }
}
