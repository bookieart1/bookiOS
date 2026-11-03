/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::dyld::{ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::foundation::ns_string;
use crate::mem::MutVoidPtr;
use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};
use crate::{export_c_func, Environment};

#[derive(Default)]
pub struct State {
    uncaught_exception_handler: MutVoidPtr,
}

struct NSExceptionHostObject {
    name: id,
    reason: id,
    user_info: id,
}
impl HostObject for NSExceptionHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSException: NSObject

+ (())raise:(id)name format:(id)format, ...args {
    let _ = this;
    let reason_text = ns_string::with_format(env, format, args.start());
    let name_text = if name == nil {
        "(unnamed)".to_owned()
    } else {
        ns_string::to_rust_string(env, name).into_owned()
    };
    // Objective-C exception unwinding is not implemented. Preserve strict
    // failure here while exposing the actual formatted guest exception rather
    // than failing earlier on a missing selector or silently continuing.
    panic!("Guest raised NSException {name_text:?}: {reason_text:?}");
}

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSExceptionHostObject {
        name: nil,
        reason: nil,
        user_info: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)exceptionWithName:(id)name reason:(id)reason userInfo:(id)user_info {
    let exception: id = msg![env; this alloc];
    let exception: id = msg![env; exception initWithName:name reason:reason userInfo:user_info];
    autorelease(env, exception)
}

- (id)initWithName:(id)name reason:(id)reason userInfo:(id)user_info {
    retain(env, name);
    retain(env, reason);
    retain(env, user_info);
    let host_object = env.objc.borrow_mut::<NSExceptionHostObject>(this);
    host_object.name = name;
    host_object.reason = reason;
    host_object.user_info = user_info;
    this
}

- (id)name {
    env.objc.borrow::<NSExceptionHostObject>(this).name
}

- (id)reason {
    env.objc.borrow::<NSExceptionHostObject>(this).reason
}

- (id)userInfo {
    env.objc.borrow::<NSExceptionHostObject>(this).user_info
}

- (())raise {
    // Full Objective-C stack unwinding is not available yet. A number of old
    // third-party SDKs construct and raise an exception for recoverable setup
    // failures, then rely on their surrounding Objective-C handler. Let the
    // guest continue at the call site rather than turning that SDK failure into
    // an emulator crash.
    let (name, reason) = {
        let host_object = env.objc.borrow::<NSExceptionHostObject>(this);
        (host_object.name, host_object.reason)
    };
    let name = if name == nil {
        "(unnamed)".to_owned()
    } else {
        ns_string::to_rust_string(env, name).into_owned()
    };
    let reason = if reason == nil {
        "(no reason)".to_owned()
    } else {
        ns_string::to_rust_string(env, reason).into_owned()
    };
    log!("Warning: ignoring raised NSException {:?}: {:?}", name, reason);
}

- (())dealloc {
    let &NSExceptionHostObject { name, reason, user_info } = env.objc.borrow(this);
    release(env, name);
    release(env, reason);
    release(env, user_info);
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

};

// All constants are NSExceptionName
pub const CONSTANTS: ConstantExports = &[
    (
        "_NSCharacterConversionException",
        HostConstant::NSString("NSCharacterConversionException"),
    ),
    (
        "_NSDecimalNumberDivideByZeroException",
        HostConstant::NSString("NSDecimalNumberDivideByZeroException"),
    ),
    (
        "_NSDecimalNumberExactnessException",
        HostConstant::NSString("NSDecimalNumberExactnessException"),
    ),
    (
        "_NSDecimalNumberOverflowException",
        HostConstant::NSString("NSDecimalNumberOverflowException"),
    ),
    (
        "_NSDecimalNumberUnderflowException",
        HostConstant::NSString("NSDecimalNumberUnderflowException"),
    ),
    (
        "_NSDestinationInvalidException",
        HostConstant::NSString("NSDestinationInvalidException"),
    ),
    (
        "_NSFileHandleOperationException",
        HostConstant::NSString("NSFileHandleOperationException"),
    ),
    (
        "_NSGenericException",
        HostConstant::NSString("NSGenericException"),
    ),
    (
        "_NSInternalInconsistencyException",
        HostConstant::NSString("NSInternalInconsistencyException"),
    ),
    (
        "_NSInvalidArchiveOperationException",
        HostConstant::NSString("NSInvalidArchiveOperationException"),
    ),
    (
        "_NSInvalidArgumentException",
        HostConstant::NSString("NSInvalidArgumentException"),
    ),
    (
        "_NSInvalidReceivePortException",
        HostConstant::NSString("NSInvalidReceivePortException"),
    ),
    (
        "_NSInvalidSendPortException",
        HostConstant::NSString("NSInvalidSendPortException"),
    ),
    (
        "_NSInvalidUnarchiveOperationException",
        HostConstant::NSString("NSInvalidUnarchiveOperationException"),
    ),
    (
        "_NSInvocationOperationCancelledException",
        HostConstant::NSString("NSInvocationOperationCancelledException"),
    ),
    (
        "_NSInvocationOperationVoidResultException",
        HostConstant::NSString("NSInvocationOperationVoidResultException"),
    ),
    (
        "_NSMallocException",
        HostConstant::NSString("NSMallocException"),
    ),
    (
        "_NSObjectInaccessibleException",
        HostConstant::NSString("NSObjectInaccessibleException"),
    ),
    (
        "_NSObjectNotAvailableException",
        HostConstant::NSString("NSObjectNotAvailableException"),
    ),
    (
        "_NSOldStyleException",
        HostConstant::NSString("NSOldStyleException"),
    ),
    (
        "_NSParseErrorException",
        HostConstant::NSString("NSParseErrorException"),
    ),
    (
        "_NSPortReceiveException",
        HostConstant::NSString("NSPortReceiveException"),
    ),
    (
        "_NSPortSendException",
        HostConstant::NSString("NSPortSendException"),
    ),
    (
        "_NSPortTimeoutException",
        HostConstant::NSString("NSPortTimeoutException"),
    ),
    (
        "_NSRangeException",
        HostConstant::NSString("NSRangeException"),
    ),
    (
        "_NSUndefinedKeyException",
        HostConstant::NSString("NSUndefinedKeyException"),
    ),
    (
        "_NSInconsistentArchiveException",
        HostConstant::NSString("NSInconsistentArchiveException"),
    ),
    (
        "_NSPPDIncludeNotFoundException",
        HostConstant::NSString("NSPPDIncludeNotFoundException"),
    ),
    (
        "_NSPPDIncludeStackOverflowException",
        HostConstant::NSString("NSPPDIncludeStackOverflowException"),
    ),
    (
        "_NSPPDIncludeStackUnderflowException",
        HostConstant::NSString("NSPPDIncludeStackUnderflowException"),
    ),
    (
        "_NSPPDParseException",
        HostConstant::NSString("NSPPDParseException"),
    ),
    (
        "_NSRTFPropertyStackOverflowException",
        HostConstant::NSString("NSRTFPropertyStackOverflowException"),
    ),
    (
        "_NSTIFFException",
        HostConstant::NSString("NSTIFFException"),
    ),
    (
        "_NSAbortModalException",
        HostConstant::NSString("NSAbortModalException"),
    ),
    (
        "_NSAbortPrintingException",
        HostConstant::NSString("NSAbortPrintingException"),
    ),
    (
        "_NSAccessibilityException",
        HostConstant::NSString("NSAccessibilityException"),
    ),
    (
        "_NSAppKitIgnoredException",
        HostConstant::NSString("NSAppKitIgnoredException"),
    ),
    (
        "_NSAppKitVirtualMemoryException",
        HostConstant::NSString("NSAppKitVirtualMemoryException"),
    ),
    (
        "_NSBadBitmapParametersException",
        HostConstant::NSString("NSBadBitmapParametersException"),
    ),
    (
        "_NSBadComparisonException",
        HostConstant::NSString("NSBadComparisonException"),
    ),
    (
        "_NSBadRTFColorTableException",
        HostConstant::NSString("NSBadRTFColorTableException"),
    ),
    (
        "_NSBadRTFDirectiveException",
        HostConstant::NSString("NSBadRTFDirectiveException"),
    ),
    (
        "_NSBadRTFFontTableException",
        HostConstant::NSString("NSBadRTFFontTableException"),
    ),
    (
        "_NSBadRTFStyleSheetException",
        HostConstant::NSString("NSBadRTFStyleSheetException"),
    ),
    (
        "_NSBrowserIllegalDelegateException",
        HostConstant::NSString("NSBrowserIllegalDelegateException"),
    ),
    (
        "_NSColorListIOException",
        HostConstant::NSString("NSColorListIOException"),
    ),
    (
        "_NSColorListNotEditableException",
        HostConstant::NSString("NSColorListNotEditableException"),
    ),
    (
        "_NSDraggingException",
        HostConstant::NSString("NSDraggingException"),
    ),
    (
        "_NSFontUnavailableException",
        HostConstant::NSString("NSFontUnavailableException"),
    ),
    (
        "_NSIllegalSelectorException",
        HostConstant::NSString("NSIllegalSelectorException"),
    ),
    (
        "_NSImageCacheException",
        HostConstant::NSString("NSImageCacheException"),
    ),
    (
        "_NSNibLoadingException",
        HostConstant::NSString("NSNibLoadingException"),
    ),
    (
        "_NSPasteboardCommunicationException",
        HostConstant::NSString("NSPasteboardCommunicationException"),
    ),
    (
        "_NSPrintOperationExistsException",
        HostConstant::NSString("NSPrintOperationExistsException"),
    ),
    (
        "_NSPrintPackageException",
        HostConstant::NSString("NSPrintPackageException"),
    ),
    (
        "_NSPrintingCommunicationException",
        HostConstant::NSString("NSPrintingCommunicationException"),
    ),
    (
        "_NSTextLineTooLongException",
        HostConstant::NSString("NSTextLineTooLongException"),
    ),
    (
        "_NSTextNoSelectionException",
        HostConstant::NSString("NSTextNoSelectionException"),
    ),
    (
        "_NSTextReadException",
        HostConstant::NSString("NSTextReadException"),
    ),
    (
        "_NSTextWriteException",
        HostConstant::NSString("NSTextWriteException"),
    ),
    (
        "_NSTypedStreamVersionException",
        HostConstant::NSString("NSTypedStreamVersionException"),
    ),
    (
        "_NSWindowServerCommunicationException",
        HostConstant::NSString("NSWindowServerCommunicationException"),
    ),
    (
        "_NSWordTablesReadException",
        HostConstant::NSString("NSWordTablesReadException"),
    ),
    (
        "_NSWordTablesWriteException",
        HostConstant::NSString("NSWordTablesWriteException"),
    ),
    (
        "_UIViewControllerHierarchyInconsistencyException",
        HostConstant::NSString("UIViewControllerHierarchyInconsistencyException"),
    ),
    (
        "_UIApplicationInvalidInterfaceOrientationException",
        HostConstant::NSString("UIApplicationInvalidInterfaceOrientationException"),
    ),
];

/// Store the process-wide handler exactly as Foundation's setter/getter pair
/// does. Objective-C exception unwinding is still outside the emulator's
/// current boundary, but SDKs legitimately use this pair to chain handlers.
fn NSSetUncaughtExceptionHandler(env: &mut Environment, handler: MutVoidPtr) {
    env.framework_state
        .foundation
        .ns_exception
        .uncaught_exception_handler = handler;
}

fn NSGetUncaughtExceptionHandler(env: &mut Environment) -> MutVoidPtr {
    env.framework_state
        .foundation
        .ns_exception
        .uncaught_exception_handler
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSSetUncaughtExceptionHandler(_)),
    export_c_func!(NSGetUncaughtExceptionHandler()),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncaught_exception_handler_round_trips() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let handler = MutVoidPtr::from_bits(0x1234_5679);
        NSSetUncaughtExceptionHandler(&mut env, handler);
        assert_eq!(NSGetUncaughtExceptionHandler(&mut env), handler);
        NSSetUncaughtExceptionHandler(&mut env, MutVoidPtr::null());
        assert!(NSGetUncaughtExceptionHandler(&mut env).is_null());
    }

    #[test]
    fn class_raise_format_selector_is_exported() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let class = env.objc.get_known_class("NSException", &mut env.mem);
        let selector = env
            .objc
            .register_host_selector("raise:format:".to_owned(), &mut env.mem);
        let responds: bool = msg![env; class respondsToSelector:selector];
        assert!(responds);
    }
}
