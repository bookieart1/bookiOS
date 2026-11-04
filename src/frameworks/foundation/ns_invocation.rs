/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSInvocation`.

use crate::abi::{extend_stack_for_args, write_next_arg, GuestArg};
use crate::cpu::Cpu;
use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::{NSInteger, NSUInteger};
use crate::libc::string::strdup;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr};
use crate::msg;
use crate::objc::{
    autorelease, id, nil, objc_classes, objc_msgSend, release, retain, ClassExports, HostObject,
    SEL,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::msg_class;
    use crate::Environment;

    #[test]
    fn selector_typed_argument_round_trips_through_invocation() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let target: id = msg_class![env; NSObject new];
        let perform = env
            .objc
            .register_host_selector("performSelector:".to_owned(), &mut env.mem);
        let description = env
            .objc
            .register_host_selector("description".to_owned(), &mut env.mem);
        let signature: id = msg![env; target methodSignatureForSelector:perform];
        assert_ne!(signature, nil);
        let invocation: id = msg_class![env; NSInvocation invocationWithMethodSignature:signature];
        () = msg![env; invocation setTarget:target];
        () = msg![env; invocation setSelector:perform];
        let recorded_selector: SEL = msg![env; invocation selector];
        assert_eq!(recorded_selector, perform);
        let selector_argument = env.mem.alloc_and_write(description);
        let selector_argument_raw: MutVoidPtr = selector_argument.cast();
        () = msg![env; invocation setArgument:selector_argument_raw atIndex:2i32];
        () = msg![env; invocation invoke];
        let return_value = env.mem.alloc_and_write(nil);
        let return_value_raw: MutVoidPtr = return_value.cast();
        () = msg![env; invocation getReturnValue:return_value_raw];
        assert_ne!(env.mem.read(return_value), nil);
        env.mem.free(return_value.cast());
        env.mem.free(selector_argument.cast());
        release(env, target);
        release(env, pool);
        env.cpu.regs_mut()[Cpu::SP] = 0;
        env.mem.free(stack);
    }
}

struct NSInvocationHostObject {
    /// `NSMethodSignature *`
    sig: id,
    /// Argument type strings resolved from `sig` at creation time
    argument_types: Vec<String>,
    /// Objective-C return type resolved from `sig` at creation time.
    return_type: String,
    /// Most recent return value, in the representation exposed by
    /// `getReturnValue:`.
    return_value: Vec<u8>,
    target: id,
    selector: Option<SEL>,
    /// Per-slot owned buffer for each argument.
    /// Option denotes if argument was set with `setArgument:atIndex:`
    arguments: Vec<Option<MutVoidPtr>>,
    arguments_retained: bool,
    /// Objects retained by `retainArguments`
    retained_objects: Vec<id>,
    /// C string copies made by `retainArguments`
    copied_strings: Vec<MutPtr<u8>>,
}
impl HostObject for NSInvocationHostObject {}

fn record_invocation_state(
    env: &mut crate::Environment,
    action: &'static str,
    invocation: id,
    argument_index: i32,
) {
    let (target, selector, argument_count, set_mask) = {
        let host = env.objc.borrow::<NSInvocationHostObject>(invocation);
        let set_mask =
            host.arguments
                .iter()
                .take(32)
                .enumerate()
                .fold(0u32, |mask, (index, argument)| {
                    if argument.is_some() {
                        mask | (1u32 << index)
                    } else {
                        mask
                    }
                });
        (
            host.target.to_bits(),
            host.selector.map_or(0, |selector| {
                let mut regs = [0];
                selector.to_regs(&mut regs);
                regs[0]
            }),
            host.arguments.len().try_into().unwrap_or(u32::MAX),
            set_mask,
        )
    };
    env.diagnostic_invocation_event(
        action,
        invocation.to_bits(),
        target,
        selector,
        argument_index,
        argument_count,
        set_mask,
    );
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSInvocation: NSObject

+ (id)invocationWithMethodSignature:(id)sig { // NSMethodSignature *
    retain(env, sig);
    let num_of_args: NSUInteger = msg![env; sig numberOfArguments];
    let mut argument_types: Vec<String> = Vec::with_capacity(num_of_args as usize);
    for i in 0..num_of_args {
        let type_ptr: ConstPtr<u8> = msg![env; sig getArgumentTypeAtIndex:i];
        argument_types.push(env.mem.cstr_at_utf8(type_ptr).unwrap().to_string());
    }
    let return_type_ptr: ConstPtr<u8> = msg![env; sig methodReturnType];
    let return_type = env.mem.cstr_at_utf8(return_type_ptr).unwrap().to_string();
    let return_length: NSUInteger = msg![env; sig methodReturnLength];
    let host_object = Box::new(NSInvocationHostObject {
        sig,
        argument_types,
        return_type,
        return_value: vec![0; return_length as usize],
        target: nil,
        selector: None,
        arguments: vec![None; num_of_args as usize],
        arguments_retained: false,
        retained_objects: Vec::new(),
        copied_strings: Vec::new(),
    });
    let res = env.objc.alloc_object(this, host_object, &mut env.mem);
    record_invocation_state(env, "create", res, -1);
    autorelease(env, res)
}

- (id)target {
    env.objc.borrow::<NSInvocationHostObject>(this).target
}
- (())setTarget:(id)target {
    let old_target = env.objc.borrow::<NSInvocationHostObject>(this).target;
    let arguments_retained = env.objc.borrow::<NSInvocationHostObject>(this).arguments_retained;
    env.objc.borrow_mut::<NSInvocationHostObject>(this).target = target;
    if arguments_retained {
        retain(env, target);
        release(env, old_target);
    }
    record_invocation_state(env, "set-target", this, -1);
}

- (())setSelector:(SEL)selector {
    assert!(env.objc.borrow_mut::<NSInvocationHostObject>(this).selector.is_none()); // TODO
    env.objc.borrow_mut::<NSInvocationHostObject>(this).selector = Some(selector);
    record_invocation_state(env, "set-selector", this, -1);
}

- (SEL)selector {
    env.objc
        .borrow::<NSInvocationHostObject>(this)
        .selector
        .expect("NSInvocation selector requested before it was set")
}

- (())retainArguments {
    // TODO: handle return val
    // TODO: copy blocks
    assert!(!env.objc.borrow::<NSInvocationHostObject>(this).arguments_retained); // TODO

    let target = env.objc.borrow::<NSInvocationHostObject>(this).target;
    retain(env, target);

    let mut retained_objects: Vec<id> = Vec::new();
    let mut copied_strings: Vec<MutPtr<u8>> = Vec::new();

    // Skip index 0 (self) and 1 (SEL): handled via target/selector fields.
    let num_of_args = env.objc.borrow::<NSInvocationHostObject>(this).argument_types.len();
    for i in 2..num_of_args {
        let host = env.objc.borrow::<NSInvocationHostObject>(this);
        let Some(arg_loc) = host.arguments[i] else { continue };
        match host.argument_types[i].as_str() {
            "@" => {
                let obj: id = env.mem.read(arg_loc.cast().cast_const());
                retain(env, obj);
                retained_objects.push(obj);
            }
            "*" => {
                let str: MutPtr<u8> = env.mem.read(arg_loc.cast().cast_const());
                let str_copy = strdup(env, str.cast_const());
                env.mem.write(arg_loc.cast(), str_copy);
                copied_strings.push(str_copy);
            }
            _ => {}
        }
    }

    let host = env.objc.borrow_mut::<NSInvocationHostObject>(this);
    host.retained_objects = retained_objects;
    host.copied_strings = copied_strings;
    host.arguments_retained = true;
    record_invocation_state(env, "retain-arguments", this, -1);
}

- (())setArgument:(MutVoidPtr)arg_loc
          atIndex:(NSInteger)idx {
    let &NSInvocationHostObject {
        ref arguments,
        arguments_retained,
        ..
    } = env.objc.borrow::<NSInvocationHostObject>(this);

    // 0 and 1 are reserved for `self` and `_cmd`
    // TODO: can they be set too?
    assert!(1 < idx && idx < arguments.len() as NSInteger);

    if let Some(prev_arg) = arguments[idx as usize] {
        env.mem.free(prev_arg.cast());
    }

    let argument_types: &Vec<String> = env
        .objc
        .borrow::<NSInvocationHostObject>(this)
        .argument_types
        .as_ref();
    let arg_type = argument_types.get(idx as usize).unwrap();
    let selector = env
        .objc
        .borrow::<NSInvocationHostObject>(this)
        .selector;
    let is_init_with_frame_rect = arg_type == "?"
        && idx == 2
        && selector
            .map(|selector| selector.as_str(&env.mem) == "initWithFrame:")
            .unwrap_or(false);
    let new: MutVoidPtr = match arg_type.as_str() {
        "f" => {
            let arg_loc: MutPtr<f32> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        "@" => {
            assert!(!arguments_retained); // TODO
            let arg_loc: MutPtr<id> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        ":" => {
            let arg_loc: MutPtr<SEL> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        "*" => {
            assert!(!arguments_retained); // TODO
            let arg_loc: MutPtr<MutPtr<u8>> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        // pointer cases
        _ if arg_type.starts_with('^') => {
            let arg_loc: MutPtr<MutVoidPtr> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        "?" if is_init_with_frame_rect => {
            let arg_loc: MutPtr<CGRect> = arg_loc.cast();
            let arg = env.mem.read(arg_loc);
            env.mem.alloc_and_write(arg).cast()
        }
        _ => {
            let selector = env
                .objc
                .borrow::<NSInvocationHostObject>(this)
                .selector
                .map(|selector| selector.as_str(&env.mem).to_owned())
                .unwrap_or_else(|| "<not set>".to_owned());
            unimplemented!(
                "unhandled argument type {arg_type} for selector {selector}; signature arguments: {:?}",
                argument_types
            )
        }
    };

    env.objc.borrow_mut::<NSInvocationHostObject>(this).arguments[idx as usize] = Some(new);
    record_invocation_state(env, "set-argument", this, idx);
}

- (())invokeWithTarget:(id)target {
    () = msg![env; this setTarget:target];
    () = msg![env; this invoke];
}

- (())getReturnValue:(MutVoidPtr)buffer {
    let return_value = &env.objc.borrow::<NSInvocationHostObject>(this).return_value;
    if !return_value.is_empty() {
        env.mem
            .bytes_at_mut(buffer.cast(), return_value.len().try_into().unwrap())
            .copy_from_slice(return_value);
    }
}

- (())setReturnValue:(MutVoidPtr)buffer {
    let return_length = env
        .objc
        .borrow::<NSInvocationHostObject>(this)
        .return_value
        .len();
    if return_length != 0 {
        let new_value = env
            .mem
            .bytes_at(buffer.cast(), return_length.try_into().unwrap())
            .to_vec();
        env.objc.borrow_mut::<NSInvocationHostObject>(this).return_value = new_value;
    }
}

- (())invoke {
    record_invocation_state(env, "invoke", this, -1);
    // Native NSInvocation storage is zero-filled. Preserve strict handling for
    // unsupported scalar layouts, but materialize omitted object arguments as
    // nil instead of treating the untouched zero slot as an emulator fault.
    let missing_object_arguments: Vec<usize> = {
        let host = env.objc.borrow::<NSInvocationHostObject>(this);
        host.arguments
            .iter()
            .enumerate()
            .skip(2)
            .filter_map(|(index, argument)| {
                argument.is_none().then(|| {
                    assert_eq!(host.argument_types[index], "@");
                    index
                })
            })
            .collect()
    };
    for index in missing_object_arguments {
        let zero = env.mem.alloc_and_write(nil).cast();
        env.objc.borrow_mut::<NSInvocationHostObject>(this).arguments[index] = Some(zero);
        record_invocation_state(env, "default-nil-argument", this, index as i32);
    }

    // `call_from_host` re-use
    // TODO: retval_ptr
    // TODO: cross check against frame length from NSMethodSignature
    let mut reg_count = 0;
    let argument_types: &Vec<String> = env.objc.borrow::<NSInvocationHostObject>(this).argument_types.as_ref();
    let selector = env.objc.borrow::<NSInvocationHostObject>(this).selector.unwrap();
    for (index, arg_type) in argument_types.iter().enumerate() {
        // TODO: refactor and simplify
        reg_count += match arg_type.as_str() {
            "@" => <id as GuestArg>::REG_COUNT,
            ":" => <SEL as GuestArg>::REG_COUNT,
            "f" => <f32 as GuestArg>::REG_COUNT,
            "c" => <u8 as GuestArg>::REG_COUNT,
            "*" => <MutPtr<u8> as GuestArg>::REG_COUNT,
            // pointer cases
            _ if arg_type.starts_with('^') => <MutVoidPtr as GuestArg>::REG_COUNT,
            "?" if index == 2 && selector.as_str(&env.mem) == "initWithFrame:" => {
                <CGRect as GuestArg>::REG_COUNT
            }
            _ => unimplemented!("reg_count for {arg_type}")
        }
    }
    let regs = env.cpu.regs_mut();
    let old_sp = extend_stack_for_args(
        reg_count,
        regs,
    );

    let arguments: &Vec<Option<MutVoidPtr>> = env.objc.borrow::<NSInvocationHostObject>(this).arguments.as_ref();
    let mut reg_offset = 0;
    for i in 0..arguments.len() {
        // TODO: do not handle target and sel as special cases
        if i == 0 {
            assert!(argument_types[i] == "@");
            // target
            let target = env.objc.borrow::<NSInvocationHostObject>(this).target;
            let regs = env.cpu.regs_mut();
            write_next_arg::<id>(&mut reg_offset, regs, &mut env.mem, target);
            continue;
        }
        if i == 1 {
            assert!(argument_types[i] == ":");
            // selector
            let selector = env.objc.borrow::<NSInvocationHostObject>(this).selector.unwrap();
            let regs = env.cpu.regs_mut();
            write_next_arg::<SEL>(&mut reg_offset, regs, &mut env.mem, selector);
            continue;
        }
        let arg_slot = arguments[i].unwrap();
        let arg_type = argument_types[i].as_str();
        // TODO: refactor and simplify
        match arg_type {
            "@" => {
                let arg: ConstPtr<id> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<id>(&mut reg_offset, regs, &mut env.mem, arg_val);
            },
            ":" => {
                let arg: ConstPtr<SEL> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<SEL>(&mut reg_offset, regs, &mut env.mem, arg_val);
            },
            "f" => {
                let arg: ConstPtr<f32> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<f32>(&mut reg_offset, regs, &mut env.mem, arg_val);
            },
            "c" => {
                let arg: ConstPtr<u8> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<u8>(&mut reg_offset, regs, &mut env.mem, arg_val);
            }
            "*" => {
                let arg: ConstPtr<MutPtr<u8>> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<MutPtr<u8>>(&mut reg_offset, regs, &mut env.mem, arg_val);
            }
            // pointer cases
            _ if arg_type.starts_with('^') => {
                let arg: ConstPtr<MutVoidPtr> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<MutVoidPtr>(&mut reg_offset, regs, &mut env.mem, arg_val);
            }
            "?" if i == 2 && selector.as_str(&env.mem) == "initWithFrame:" => {
                let arg: ConstPtr<CGRect> = arg_slot.cast().cast_const();
                let arg_val = env.mem.read(arg);
                let regs = env.cpu.regs_mut();
                write_next_arg::<CGRect>(&mut reg_offset, regs, &mut env.mem, arg_val);
            }
            _ => unimplemented!("write_next_arg for {arg_type}")
        }
    }

    // actual invocation
    let &NSInvocationHostObject { target, selector, .. } = env.objc.borrow::<NSInvocationHostObject>(this);
    objc_msgSend(env, target, selector.unwrap());

    // Scalar and object return values on 32-bit ARM use r0, or r0-r1 for
    // eight-byte values. NSMethodSignature currently accepts these same
    // encodings, so preserve the raw ABI representation for getReturnValue:.
    // Larger aggregate returns need an implicit result pointer and remain a
    // separate implementation task.
    let return_length = env
        .objc
        .borrow::<NSInvocationHostObject>(this)
        .return_value
        .len();
    if return_length != 0 {
        let return_type = env
            .objc
            .borrow::<NSInvocationHostObject>(this)
            .return_type
            .clone();
        assert!(
            return_length <= 8,
            "NSInvocation aggregate return type {return_type:?} ({return_length} bytes) is not supported"
        );
        let regs = env.cpu.regs();
        let mut raw_return = [0u8; 8];
        raw_return[..4].copy_from_slice(&regs[0].to_le_bytes());
        if return_length > 4 {
            raw_return[4..].copy_from_slice(&regs[1].to_le_bytes());
        }
        env.objc
            .borrow_mut::<NSInvocationHostObject>(this)
            .return_value
            .copy_from_slice(&raw_return[..return_length]);
    }

    let regs = env.cpu.regs_mut(); // re-borrow
    regs[Cpu::SP] = old_sp;
}

- (())dealloc {
    let &NSInvocationHostObject { sig, target, arguments_retained, .. } = env.objc.borrow::<NSInvocationHostObject>(this);
    release(env, sig);
    if arguments_retained {
        release(env, target);
        let retained_objects = std::mem::take(
            &mut env.objc.borrow_mut::<NSInvocationHostObject>(this).retained_objects
        );
        for obj in retained_objects {
            release(env, obj);
        }
        let copied_strings = std::mem::take(
            &mut env.objc.borrow_mut::<NSInvocationHostObject>(this).copied_strings
        );
        for s in copied_strings {
            env.mem.free(s.cast());
        }
    } else {
        assert!(env.objc.borrow::<NSInvocationHostObject>(this).retained_objects.is_empty());
        assert!(env.objc.borrow::<NSInvocationHostObject>(this).copied_strings.is_empty());
    }
    for ptr in env.objc.borrow::<NSInvocationHostObject>(this).arguments.iter().flatten() {
        env.mem.free(ptr.cast());
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
