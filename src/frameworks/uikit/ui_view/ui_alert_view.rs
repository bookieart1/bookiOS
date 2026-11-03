/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIAlertView`.
//!
//! The alert is shown with the host's own alert (an SDL messagebox) rather
//! than drawn as a view.

use crate::frameworks::foundation::{ns_string, NSInteger};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, Class, ClassExports, NSZonePtr, SEL,
};
use std::ffi::{c_char, CStr, CString};
use std::sync::OnceLock;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn historical_initializer_preserves_button_roles_and_order() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // UIView initialization passes CGRect values through the guest ABI.
        // The lightweight fixture has no stack unless the test supplies one.
        let stack_size = 0x1000;
        let stack = env.mem.alloc(stack_size);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + stack_size;
        let title = ns_string::get_static_str(env, "Title");
        let message = ns_string::get_static_str(env, "Message");
        let default_button = ns_string::get_static_str(env, "Play");
        let cancel_button = ns_string::get_static_str(env, "Cancel");
        let extra = ns_string::get_static_str(env, "Help");
        let extras: id = msg_class![env; NSMutableArray new];
        () = msg![env; extras addObject:extra];
        let alert: id = msg_class![env; UIAlertView alloc];
        let alert: id = msg![env; alert initWithTitle:title
                                             message:message
                                            delegate:nil
                                       defaultButton:default_button
                                        cancelButton:cancel_button
                                        otherButtons:extras];
        let count: NSInteger = msg![env; alert numberOfButtons];
        let default_index: NSInteger = msg![env; alert defaultButtonIndex];
        let cancel_index: NSInteger = msg![env; alert cancelButtonIndex];
        assert_eq!(count, 3);
        assert_eq!(default_index, 0);
        assert_eq!(cancel_index, 1);
        let returned_extra: id = msg![env; alert buttonTitleAtIndex:2i32];
        let equal: bool = msg![env; returned_extra isEqual:extra];
        assert!(equal);
        release(env, alert);
        release(env, extras);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}

type NativeTextAlertFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
static NATIVE_TEXT_ALERT: OnceLock<NativeTextAlertFn> = OnceLock::new();

/// Registered by the iOS executable. Keeping the callback optional lets the
/// Rust library continue to build and run on desktop platforms.
#[no_mangle]
pub extern "C" fn idared_set_native_text_alert(prompt: NativeTextAlertFn) {
    let _ = NATIVE_TEXT_ALERT.set(prompt);
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ch if (ch as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('\"');
    out
}

fn collect_text_fields(env: &mut crate::Environment, view: id, result: &mut Vec<id>) {
    let text_field_class = env.objc.get_known_class("UITextField", &mut env.mem);
    let subviews = env.objc.borrow::<super::UIViewHostObject>(view).subviews.clone();
    for subview in subviews {
        let class: Class = msg![env; subview class];
        if env.objc.class_is_subclass_of(class, text_field_class) {
            result.push(subview);
        } else {
            collect_text_fields(env, subview, result);
        }
    }
}

fn decode_hex_text(value: &str) -> Option<String> {
    if value.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<Vec<u8>> = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect();
    String::from_utf8(bytes?).ok()
}

fn show_native_text_alert(
    env: &mut crate::Environment,
    title: &str,
    message: &str,
    buttons: &[String],
    cancel_button: Option<usize>,
    text_fields: &[id],
) -> Option<(usize, Vec<String>)> {
    let prompt = *NATIVE_TEXT_ALERT.get()?;
    let field_json: Vec<String> = text_fields
        .iter()
        .map(|&field| {
            let text: id = msg![env; field text];
            let placeholder: id = msg![env; field placeholder];
            let secure: bool = msg![env; field isSecureTextEntry];
            let text = if text == nil {
                String::new()
            } else {
                ns_string::to_rust_string(env, text).into_owned()
            };
            let placeholder = if placeholder == nil {
                "null".to_owned()
            } else {
                json_string(&ns_string::to_rust_string(env, placeholder))
            };
            format!(
                "{{\"text\":{},\"placeholder\":{},\"secure\":{}}}",
                json_string(&text),
                placeholder,
                secure
            )
        })
        .collect();
    let button_json = buttons
        .iter()
        .map(|button| json_string(button))
        .collect::<Vec<_>>()
        .join(",");
    let cancel_json = cancel_button.map_or("null".to_owned(), |index| index.to_string());
    let input = format!(
        "{{\"title\":{},\"message\":{},\"buttons\":[{}],\"cancelIndex\":{},\"textFields\":[{}]}}",
        json_string(title),
        json_string(message),
        button_json,
        cancel_json,
        field_json.join(",")
    );
    let input = CString::new(input).unwrap();
    let result = env.on_parent_stack_in_coroutine(|window, _| {
        // A guest field may already have started SDL text input. The native
        // alert owns input until it returns, so dismiss the SDL keyboard first.
        window.stop_text_input();
        unsafe { prompt(input.as_ptr()) }
    });
    if result.is_null() {
        return None;
    }
    let result_string = unsafe { CStr::from_ptr(result) }
        .to_string_lossy()
        .into_owned();
    unsafe { libc::free(result.cast()) };

    let mut parts = result_string.split('\n');
    let index = parts.next()?.parse().ok()?;
    let texts = parts.map(decode_hex_text).collect::<Option<Vec<_>>>()?;
    if index >= buttons.len() || texts.len() != text_fields.len() {
        return None;
    }
    Some((index, texts))
}

#[derive(Default)]
pub struct UIAlertViewHostObject {
    superclass: super::UIViewHostObject,
    /// `NSString*`
    title: id,
    /// `NSString*`
    message: id,
    /// Weak reference, like in UIKit.
    delegate: id,
    /// `NSString*`s, in index order.
    button_titles: Vec<id>,
    default_button_index: NSInteger,
    cancel_button_index: NSInteger,
}
impl_HostObject_with_superclass!(UIAlertViewHostObject);

fn add_button(env: &mut crate::Environment, this: id, title: id) -> NSInteger {
    let title: id = msg![env; title copy];
    let host = env.objc.borrow_mut::<UIAlertViewHostObject>(this);
    host.button_titles.push(title);
    (host.button_titles.len() - 1) as NSInteger
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIAlertView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIAlertViewHostObject {
        default_button_index: -1,
        cancel_button_index: -1,
        ..Default::default()
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithTitle:(id)title
                      message:(id)message
                     delegate:(id)delegate
            cancelButtonTitle:(id)cancelButtonTitle
            otherButtonTitles:(id)otherButtonTitles, ...args {
    let this: id = msg_super![env; this init];
    () = msg![env; this setTitle:title];
    () = msg![env; this setMessage:message];
    () = msg![env; this setDelegate:delegate];
    if cancelButtonTitle != nil {
        let index = add_button(env, this, cancelButtonTitle);
        env.objc.borrow_mut::<UIAlertViewHostObject>(this).cancel_button_index = index;
    }
    if otherButtonTitles != nil {
        add_button(env, this, otherButtonTitles);
        let mut varargs = args.start();
        loop {
            let title: id = varargs.next(env);
            if title == nil {
                break;
            }
            add_button(env, this, title);
        }
    }
    this
}

// Historical private UIKit initializer used by early Apple/ngmoco code. The
// first two button arguments are title strings and otherButtons is an NSArray
// of title strings. Keep the public alert model and ownership rules rather
// than inventing a second presentation path.
- (id)initWithTitle:(id)title
                      message:(id)message
                     delegate:(id)delegate
                defaultButton:(id)default_button
                 cancelButton:(id)cancel_button
                 otherButtons:(id)other_buttons {
    let this: id = msg_super![env; this init];
    () = msg![env; this setTitle:title];
    () = msg![env; this setMessage:message];
    () = msg![env; this setDelegate:delegate];
    if default_button != nil {
        let index = add_button(env, this, default_button);
        env.objc.borrow_mut::<UIAlertViewHostObject>(this).default_button_index = index;
    }
    if cancel_button != nil {
        let index = add_button(env, this, cancel_button);
        env.objc.borrow_mut::<UIAlertViewHostObject>(this).cancel_button_index = index;
    }
    if other_buttons != nil {
        let count: NSInteger = msg![env; other_buttons count];
        assert!(count >= 0, "UIAlertView otherButtons returned a negative count");
        for index in 0..count {
            let title: id = msg![env; other_buttons objectAtIndex:index];
            assert_ne!(title, nil, "UIAlertView otherButtons contains nil");
            add_button(env, this, title);
        }
    }
    this
}

- (())dealloc {
    let UIAlertViewHostObject {
        title,
        message,
        button_titles,
        ..
    } = std::mem::take(env.objc.borrow_mut(this));
    release(env, title);
    release(env, message);
    for title in button_titles {
        release(env, title);
    }
    msg_super![env; this dealloc]
}

- (id)title {
    env.objc.borrow::<UIAlertViewHostObject>(this).title
}
- (())setTitle:(id)title { // NSString*
    let title: id = msg![env; title copy];
    let host = env.objc.borrow_mut::<UIAlertViewHostObject>(this);
    let old = std::mem::replace(&mut host.title, title);
    release(env, old);
}

- (id)message {
    env.objc.borrow::<UIAlertViewHostObject>(this).message
}
- (())setMessage:(id)message { // NSString*
    let message: id = msg![env; message copy];
    let host = env.objc.borrow_mut::<UIAlertViewHostObject>(this);
    let old = std::mem::replace(&mut host.message, message);
    release(env, old);
}

- (id)delegate {
    env.objc.borrow::<UIAlertViewHostObject>(this).delegate
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).delegate = delegate;
}

- (NSInteger)addButtonWithTitle:(id)title { // NSString*
    add_button(env, this, title)
}

- (NSInteger)numberOfButtons {
    env.objc.borrow::<UIAlertViewHostObject>(this).button_titles.len() as NSInteger
}

- (id)buttonTitleAtIndex:(NSInteger)index {
    let titles = &env.objc.borrow::<UIAlertViewHostObject>(this).button_titles;
    usize::try_from(index).ok().and_then(|i| titles.get(i).copied()).unwrap_or(nil)
}

- (NSInteger)cancelButtonIndex {
    env.objc.borrow::<UIAlertViewHostObject>(this).cancel_button_index
}

- (NSInteger)defaultButtonIndex {
    env.objc.borrow::<UIAlertViewHostObject>(this).default_button_index
}
- (())setDefaultButtonIndex:(NSInteger)index {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).default_button_index = index;
}
- (())setCancelButtonIndex:(NSInteger)index {
    env.objc.borrow_mut::<UIAlertViewHostObject>(this).cancel_button_index = index;
}

- (())show {
    let host = env.objc.borrow::<UIAlertViewHostObject>(this);
    let (title, message) = (host.title, host.message);
    let (button_titles, cancel_index) = (host.button_titles.clone(), host.cancel_button_index);
    let to_string = |env: &mut crate::Environment, s: id| {
        if s == nil {
            String::new()
        } else {
            ns_string::to_rust_string(env, s).into_owned()
        }
    };
    let title = to_string(env, title);
    let message = to_string(env, message);
    let buttons: Vec<String> = button_titles.iter().map(|&b| to_string(env, b)).collect();
    log!("UIAlertView: title: {:?}, message: {:?}, buttons: {:?}", title, message, buttons);

    // UIKit owns an alert for the whole time that it is presented. The host
    // presentation below is synchronous and may yield to another guest run
    // loop/autorelease pool, so take that ownership before crossing into host
    // UI. The delayed delegate callback releases it after dismissal.
    retain(env, this);

    let cancel_button = usize::try_from(cancel_index)
        .ok()
        .filter(|&i| i < buttons.len());
    let mut text_fields = Vec::new();
    collect_text_fields(env, this, &mut text_fields);
    let index = if text_fields.is_empty() {
        env.on_parent_stack_in_coroutine(|window, _| {
            window.show_alert(&title, &message, &buttons, cancel_button)
        })
    } else if let Some((index, texts)) = show_native_text_alert(
        env,
        &title,
        &message,
        &buttons,
        cancel_button,
        &text_fields,
    ) {
        for (&field, text) in text_fields.iter().zip(texts) {
            let text = ns_string::from_rust_string(env, text);
            () = msg![env; field setText:text];
            let _: bool = msg![env; field resignFirstResponder];
        }
        Some(index)
    } else {
        log!("Warning: native text-entry alert was unavailable; using a message box without guest text fields.");
        env.on_parent_stack_in_coroutine(|window, _| {
            window.show_alert(&title, &message, &buttons, cancel_button)
        })
    };
    // Closing the alert without a button counts as the cancel button, like
    // UIKit does when the app is sent to the background.
    let Some(index) = index.or(cancel_button) else {
        release(env, this);
        return;
    };
    log!("UIAlertView: button {} ({:?}) tapped", index, buttons[index]);

    // On a device, show returns at once and the delegate hears about the tap
    // later, from the run loop.
    let number: id = msg_class![env; NSNumber numberWithInteger:(index as NSInteger)];
    let sel = env
        .objc
        .register_host_selector("_touchHLE_clickedButtonAtIndex:".to_string(), &mut env.mem);
    () = msg![env; this performSelector:sel withObject:number afterDelay:0.0];
}

- (())_touchHLE_clickedButtonAtIndex:(id)number { // NSNumber*
    let index: NSInteger = msg![env; number integerValue];
    let delegate = env.objc.borrow::<UIAlertViewHostObject>(this).delegate;
    if delegate != nil {
        for name in [
            "alertView:clickedButtonAtIndex:",
            "alertView:willDismissWithButtonIndex:",
            "alertView:didDismissWithButtonIndex:",
        ] {
            let sel: SEL = env.objc.register_host_selector(name.to_string(), &mut env.mem);
            let responds: bool = msg![env; delegate respondsToSelector:sel];
            if responds {
                () = crate::objc::msg_send(env, (delegate, sel, this, index));
            }
        }
    }
    release(env, this);
}

@end

};
