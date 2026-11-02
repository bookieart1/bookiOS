/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Objective-C runtime.
//!
//! Apple's [Programming with Objective-C](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/Introduction/Introduction.html)
//! is a useful introduction to the language from a user's perspective.
//! There are further resources in the child modules of this module, but they
//! are more implementation-specific.
//!
//! The strategy for this emulator will be to provide our own implementations of
//! an Objective-C runtime and libraries for it (Foundation etc). These
//! implementations will be "host code": Rust code forming part of the emulator,
//! not emulated code. The runtime will need to be able to handle classes that
//! originate from the guest app, classes defined by the host, and sometimes
//! classes that are both (considering Objective-C's support for inheritance,
//! categories and dynamic class editing).

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant, HostDylib};
use crate::objc::messages::ThreadInitializer;
use crate::MutexId;
use std::collections::HashMap;

mod classes;
mod messages;
mod methods;
mod objects;
mod properties;
mod selectors;
mod synchronization;

pub use classes::{objc_classes, Class, ClassExports, ClassTemplate};
pub use messages::{
    autorelease, msg, msg_class, msg_send, msg_send_no_initialize, msg_send_no_type_checking,
    msg_send_super2, msg_super, objc_super, release, retain,
};
pub use methods::{HostIMP, IMP};
pub use objects::{
    id, impl_HostObject_with_superclass, nil, AnyHostObject, HostObject, TrivialHostObject,
};
pub use properties::todo_objc_setter;
pub use selectors::{selector, SEL};

use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, Ptr};
use crate::Environment;
use classes::{
    class_addMethod, class_copyPropertyList, class_getClassMethod, class_getInstanceMethod,
    class_getInstanceSize, class_getMethodImplementation, class_getProperty, class_getSuperclass,
    class_replaceMethod, method_getImplementation, method_getTypeEncoding,
    method_setImplementation, objc_getClass, ClassHostObject, FakeClass, UnimplementedClass,
};
pub(crate) use messages::objc_msgSend;
use messages::{
    objc_msgSendSuper2, objc_msgSendSuper2_stret, objc_msgSend_stret, MsgSendSignature,
    MsgSendSuperSignature,
};
use methods::{method_list_t, method_t};
use objects::{objc_object, object_getClass, HostObjectEntry};
use properties::{
    ivar_getOffset, ivar_getTypeEncoding, ivar_list_t, objc_copyStruct, objc_getProperty,
    objc_setProperty, object_getInstanceVariable, object_setInstanceVariable,
};
use selectors::{sel_getUid, sel_registerName};
use synchronization::{objc_sync_enter, objc_sync_exit};

/// Typedef for `NSZone *`. This is a [fossil type] found in the signature of
/// `allocWithZone:` and similar methods. Its value is always ignored.
///
/// [fossil type]: https://en.wiktionary.org/wiki/fossil_word
pub type NSZonePtr = crate::mem::MutVoidPtr;

pub const TYPE_ID: char = '@';
pub const TYPE_CLASS: char = '#';
pub const TYPE_SEL: char = ':';
pub const TYPE_CHAR: char = 'c';
pub const TYPE_UCHAR: char = 'C';
pub const TYPE_SHORT: char = 's';
pub const TYPE_USHORT: char = 'S';
pub const TYPE_INT: char = 'i';
pub const TYPE_UINT: char = 'I';
pub const TYPE_LONG: char = 'l';
pub const TYPE_ULONG: char = 'L';
pub const TYPE_LONGLONG: char = 'q';
pub const TYPE_ULONGLONG: char = 'Q';
pub const TYPE_FLOAT: char = 'f';
pub const TYPE_DOUBLE: char = 'd';
#[allow(unused)]
pub const TYPE_BITFIELD: char = 'b';
pub const TYPE_BOOL: char = 'B';
pub const TYPE_VOID: char = 'v';
pub const TYPE_UNDEF: char = '?';
#[allow(unused)]
pub const TYPE_PTR: char = '^';
#[allow(unused)]
pub const TYPE_CHAR_PTR: char = '*';
#[allow(unused)]
pub const TYPE_ARRAY_OPEN: char = '[';
#[allow(unused)]
pub const TYPE_ARRAY_CLOSE: char = ']';
#[allow(unused)]
pub const TYPE_UNION_OPEN: char = '(';
#[allow(unused)]
pub const TYPE_UNION_CLOSE: char = ')';
#[allow(unused)]
pub const TYPE_STRUCT_OPEN: char = '{';
#[allow(unused)]
pub const TYPE_STRUCT_CLOSE: char = '}';

#[allow(unused)]
pub const METHOD_ENCODING_CONST: char = 'r';
#[allow(unused)]
pub const METHOD_ENCODING_IN: char = 'n';
#[allow(unused)]
pub const METHOD_ENCODING_INOUT: char = 'N';
#[allow(unused)]
pub const METHOD_ENCODING_OUT: char = 'o';
#[allow(unused)]
pub const METHOD_ENCODING_BYCOPY: char = 'O';
#[allow(unused)]
pub const METHOD_ENCODING_BYREF: char = 'R';
#[allow(unused)]
pub const METHOD_ENCODING_ONEWAY: char = 'V';

/// Main type holding Objective-C runtime state.
pub struct ObjC {
    /// Known selectors (interned method name strings).
    selectors: HashMap<String, SEL>,

    /// Mapping of known (guest) object pointers to their host objects.
    ///
    /// If an object isn't in this map, we will consider it not to exist.
    objects: HashMap<id, HostObjectEntry>,

    /// Known classes.
    ///
    /// Look at the `isa` to get the metaclass for a class.
    classes: HashMap<String, Class>,

    /// Mutexes used in @synchronized blocks (objc_sync_enter/exit).
    sync_mutexes: HashMap<id, MutexId>,

    /// Mutexes for running the +initialize function.
    initializer_threads: HashMap<id, ThreadInitializer>,

    /// What `class_getInstanceMethod` has returned, so a method always gets the
    /// same `Method`.
    method_handles: HashMap<(Class, SEL), MutPtr<method_t>>,

    /// Stable C strings used by the `types` field of host method handles.
    method_type_cstrings: HashMap<(Class, SEL), ConstPtr<u8>>,

    /// Cache mapping metaclasses to classes.
    metaclass_classes: HashMap<Class, Class>,

    /// GuestFunctions for class+selector combos
    cached_guest_functions: HashMap<(Class, SEL), GuestFunction>,

    /// Stable C strings returned by object_getClassName().
    class_name_cstrings: HashMap<Class, ConstPtr<u8>>,

    /// Temporary storage for optional type information when sending a message.
    /// Type information isn't part of the `objc_msgSend` ABI, so an alternative
    /// channel is needed.
    message_type_info: Option<(std::any::TypeId, &'static str)>,
}

impl ObjC {
    pub fn new() -> ObjC {
        ObjC {
            selectors: HashMap::new(),
            objects: HashMap::new(),
            classes: HashMap::new(),
            sync_mutexes: HashMap::new(),
            initializer_threads: HashMap::new(),
            method_handles: HashMap::new(),
            method_type_cstrings: HashMap::new(),
            metaclass_classes: HashMap::new(),
            cached_guest_functions: HashMap::new(),
            class_name_cstrings: HashMap::new(),
            message_type_info: None,
        }
    }
}

pub const DYLIB: HostDylib = HostDylib {
    path: "/usr/lib/libobjc.A.dylib",
    aliases: &["/usr/lib/libobjc.dylib"],
    class_exports: &[],
    constant_exports: &[CONSTANTS],
    function_exports: &[FUNCTIONS],
};

const CONSTANTS: ConstantExports = &[
    // We don't use these in our Objective-C runtime, but exporting useless
    // symbols for these silences the warning about the unhandled relocation,
    // and avoids a linker error for the integration tests.
    ("__objc_empty_vtable", HostConstant::NullPtr),
    ("__objc_empty_cache", HostConstant::NullPtr),
    // Blocks use these addresses as isa markers. The runtime implementation
    // does not need Objective-C class metadata for them, but they must be
    // stable, distinct, non-null addresses so legacy Blocks ABI checks work.
    (
        "__NSConcreteStackBlock",
        HostConstant::Custom(|env| env.mem.alloc_and_write(0u8).cast().cast_const()),
    ),
    (
        "__NSConcreteGlobalBlock",
        HostConstant::Custom(|env| env.mem.alloc_and_write(0u8).cast().cast_const()),
    ),
    (
        "__NSConcreteMallocBlock",
        HostConstant::Custom(|env| env.mem.alloc_and_write(0u8).cast().cast_const()),
    ),
];

/// Block support is iOS 4+, but it seems like Block Runtime Helpers
/// could still be called on even if minimal iOS version is set to 3.x?
///
/// ref. <https://clang.llvm.org/docs/Block-ABI-Apple.html#runtime-helper-functions>
fn _Block_object_assign(
    env: &mut Environment,
    dest: MutPtr<ConstVoidPtr>,
    object: ConstVoidPtr,
    flags: i32,
) {
    // The low byte describes the captured field. Full block/byref copying is
    // still TODO, but storing the captured pointer is enough for stack blocks
    // that do not escape. Objective-C objects need the normal retain balance.
    const BLOCK_FIELD_IS_OBJECT: i32 = 3;
    let kind = flags & 0xff;
    if kind == BLOCK_FIELD_IS_OBJECT && !object.is_null() {
        retain(env, object.cast_mut().cast());
    }
    env.mem.write(dest, object);
    log_dbg!(
        "_Block_object_assign({:?}, {:?}, {:#x}) using minimal Blocks ABI support",
        dest,
        object,
        flags
    );
}

fn _Block_object_dispose(env: &mut Environment, object: ConstVoidPtr, flags: i32) {
    const BLOCK_FIELD_IS_OBJECT: i32 = 3;
    let kind = flags & 0xff;
    if kind == BLOCK_FIELD_IS_OBJECT && !object.is_null() {
        release(env, object.cast_mut().cast());
    }
    log_dbg!(
        "_Block_object_dispose({:?}, {:#x}) using minimal Blocks ABI support",
        object,
        flags
    );
}

fn class_getName(env: &mut Environment, class: Class) -> ConstPtr<u8> {
    if let Some(&name) = env.objc.class_name_cstrings.get(&class) {
        return name;
    }
    // Apple's runtime returns an empty string for Nil, rather than NULL.
    let class_name = if class == nil {
        String::new()
    } else {
        env.objc.get_class_name(class).to_owned()
    };
    let name = env
        .mem
        .alloc_and_write_cstr(class_name.as_bytes())
        .cast_const();
    env.objc.class_name_cstrings.insert(class, name);
    name
}

fn object_getClassName(env: &mut Environment, object: id) -> ConstPtr<u8> {
    let class = object_getClass(env, object);
    if class == nil {
        return Ptr::null();
    }
    class_getName(env, class)
}

const FUNCTIONS: FunctionExports = &[
    export_c_func!(class_copyPropertyList(_, _)),
    export_c_func!(class_getInstanceSize(_)),
    export_c_func!(class_getSuperclass(_)),
    export_c_func!(class_getProperty(_, _)),
    export_c_func!(class_getInstanceMethod(_, _)),
    export_c_func!(class_getClassMethod(_, _)),
    export_c_func!(class_getName(_)),
    export_c_func!(class_addMethod(_, _, _, _)),
    export_c_func!(method_getImplementation(_)),
    export_c_func!(method_setImplementation(_, _)),
    export_c_func!(method_getTypeEncoding(_)),
    export_c_func!(class_getMethodImplementation(_, _)),
    export_c_func!(class_replaceMethod(_, _, _, _)),
    export_c_func!(objc_msgSend(_, _)),
    export_c_func!(objc_msgSend_stret(_, _, _)),
    export_c_func!(objc_msgSendSuper2(_, _)),
    export_c_func!(objc_msgSendSuper2_stret(_, _, _)),
    export_c_func!(objc_getClass(_)),
    export_c_func!(objc_getProperty(_, _, _, _)),
    export_c_func!(objc_setProperty(_, _, _, _, _, _)),
    export_c_func!(objc_copyStruct(_, _, _, _, _)),
    export_c_func!(objc_sync_enter(_)),
    export_c_func!(objc_sync_exit(_)),
    export_c_func!(object_getClass(_)),
    export_c_func!(object_getInstanceVariable(_, _, _)),
    export_c_func!(object_setInstanceVariable(_, _, _)),
    export_c_func!(ivar_getTypeEncoding(_)),
    export_c_func!(ivar_getOffset(_)),
    export_c_func!(object_getClassName(_)),
    export_c_func!(sel_getUid(_)),
    export_c_func!(sel_registerName(_)),
    export_c_func!(_Block_object_assign(_, _, _)),
    export_c_func!(_Block_object_dispose(_, _)),
];
