/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Handling of Objective-C properties.
//!
//! Note that these are not the same as instance variables (ivars), though
//! they're closely related, so maybe this file will end up being used for those
//! too.
//!
//! Resources:
//! - `objc_setProperty` and friends are not documented, so [reading the source code](https://opensource.apple.com/source/objc4/objc4-551.1/runtime/Accessors.subproj/objc-accessors.mm.auto.html) is useful.
//!
//! See also: [crate::frameworks::foundation::ns_object].

use super::{id, msg, nil, release, retain, Class, ClassHostObject, ObjC, SEL};
use crate::mem::{
    guest_size_of, ConstPtr, ConstVoidPtr, GuestISize, GuestUSize, Mem, MutPtr, MutVoidPtr, Ptr,
    SafeRead,
};
use crate::Environment;

pub struct IVar {
    pub type_: String,
    pub offset: ConstPtr<GuestUSize>,
    pub alignment: u32,
    /// Stable guest pointer to the binary's `ivar_t` metadata.
    pub(super) handle: ConstPtr<ivar_t>,
}

/// The layout of a property list in an app binary.
///
/// The name, field names and field layout are based on what Ghidra outputs.
#[repr(C, packed)]
pub(super) struct ivar_list_t {
    entsize: GuestUSize,
    count: GuestUSize,
    // entries follow the struct
}
unsafe impl SafeRead for ivar_list_t {}

/// The layout of a property in an app binary.
///
/// The name, field names and field layout are based on what Ghidra outputs.
#[repr(C, packed)]
pub(super) struct ivar_t {
    offset: ConstPtr<GuestUSize>,
    name: ConstPtr<u8>,
    type_: ConstPtr<u8>,
    alignment: u32,
    size: u32,
}
unsafe impl SafeRead for ivar_t {}

impl ClassHostObject {
    pub(super) fn add_ivars_from_bin(&mut self, ivar_list_ptr: ConstPtr<ivar_list_t>, mem: &Mem) {
        let ivar_list_t { entsize, count } = mem.read(ivar_list_ptr);
        assert!(entsize >= guest_size_of::<ivar_t>());

        let ivars_base_ptr: ConstPtr<ivar_t> = (ivar_list_ptr + 1).cast();

        for i in 0..count {
            let ivar_ptr: ConstPtr<ivar_t> = Ptr::from_bits(ivars_base_ptr.to_bits() + i * entsize);

            // TODO: support type strings
            let ivar_t {
                offset,
                name,
                alignment,
                type_,
                ..
            } = mem.read(ivar_ptr);

            let name_string = mem.cstr_at_utf8(name).unwrap().into();
            let type_ = mem.cstr_at_utf8(type_).unwrap().into();
            self.ivars.insert(
                name_string,
                IVar {
                    type_,
                    offset,
                    alignment,
                    handle: ivar_ptr,
                },
            );
        }
    }
}

impl ObjC {
    /// Looks up an ivar by name and returns its metadata together with the
    /// address of that ivar in `obj`.
    pub(super) fn object_lookup_ivar_metadata(
        &self,
        mem: &Mem,
        obj: id,
        name: &str,
    ) -> Option<(MutPtr<GuestUSize>, ConstPtr<ivar_t>)> {
        if obj == nil {
            return None;
        }
        let mut class = ObjC::read_isa(obj, mem);
        loop {
            let &ClassHostObject {
                superclass,
                ref ivars,
                ..
            } = self.borrow(class);
            if let Some(ivar) = ivars.get(name) {
                let ivar_offset = mem.read(ivar.offset);
                let ivar_ptr = MutVoidPtr::from_bits(obj.to_bits() + ivar_offset);
                return Some((ivar_ptr.cast(), ivar.handle));
            } else if superclass == nil {
                return None;
            } else {
                class = superclass;
            }
        }
    }

    /// Checks if the object's class has an ivar in its class chain with the
    /// provided name and returns the pointer to the object's ivar and its type
    /// if any or None if the object's class has no ivar with that name.
    pub fn object_lookup_ivar(
        &self,
        mem: &Mem,
        obj: id,
        name: &String,
    ) -> Option<(MutPtr<GuestUSize>, &str)> {
        let mut class = ObjC::read_isa(obj, mem);
        loop {
            let &ClassHostObject {
                superclass,
                ref ivars,
                ..
            } = self.borrow(class);
            if let Some(ivar) = ivars.get(name) {
                let ivar_offset = mem.read(ivar.offset);
                let ivar_ptr = MutVoidPtr::from_bits(obj.to_bits() + ivar_offset);
                return Some((ivar_ptr.cast(), &ivar.type_));
            } else if superclass == nil {
                return None;
            } else {
                class = superclass;
            }
        }
    }

    pub fn debug_all_class_ivars_as_strings(&self, class: Class) -> Vec<String> {
        let mut class = class;
        let mut ivars_strings = Vec::new();
        loop {
            let &ClassHostObject {
                superclass,
                ref ivars,
                ..
            } = self.borrow(class);
            let mut class_ivars_strings = ivars.keys().cloned().collect();
            ivars_strings.append(&mut class_ivars_strings);
            if superclass == nil {
                break;
            } else {
                class = superclass;
            }
        }
        ivars_strings
    }
}

/// Obtains an object-valued instance variable by name and returns the runtime
/// metadata describing it. This API predates `object_getIvar`; its output is a
/// pointer-sized value and is not retained.
pub(super) fn object_getInstanceVariable(
    env: &mut Environment,
    obj: id,
    name: ConstPtr<u8>,
    out_value: MutPtr<MutVoidPtr>,
) -> ConstPtr<ivar_t> {
    if obj == nil || name.is_null() {
        if !out_value.is_null() {
            env.mem.write(out_value, Ptr::null());
        }
        return Ptr::null();
    }

    let name = env.mem.cstr_at_utf8(name).unwrap().to_owned();
    let Some((ivar_ptr, ivar_handle)) =
        env.objc.object_lookup_ivar_metadata(&env.mem, obj, &name)
    else {
        if !out_value.is_null() {
            env.mem.write(out_value, Ptr::null());
        }
        return Ptr::null();
    };

    if !out_value.is_null() {
        let value: GuestUSize = env.mem.read(ivar_ptr);
        env.mem.write(out_value, MutVoidPtr::from_bits(value));
    }
    ivar_handle
}

/// Sets a pointer-sized instance variable by name and returns the runtime
/// metadata describing it. Like Apple's Objective-C runtime function, this is
/// a raw ivar store: it does not retain, release, copy, or otherwise interpret
/// the value.
pub(super) fn object_setInstanceVariable(
    env: &mut Environment,
    obj: id,
    name: ConstPtr<u8>,
    value: MutVoidPtr,
) -> ConstPtr<ivar_t> {
    if obj == nil || name.is_null() {
        return Ptr::null();
    }

    let name = env.mem.cstr_at_utf8(name).unwrap().to_owned();
    let Some((ivar_ptr, ivar_handle)) =
        env.objc.object_lookup_ivar_metadata(&env.mem, obj, &name)
    else {
        return Ptr::null();
    };

    env.mem.write(ivar_ptr, value.to_bits());
    ivar_handle
}

/// Sets an object-valued instance variable using the stable runtime `Ivar`
/// handle returned by `class_getInstanceVariable`. The historical runtime API
/// is a direct pointer-sized store; ownership policy remains the responsibility
/// of the declaring class just as it is for `object_setInstanceVariable`.
pub(super) fn object_setIvar(
    env: &mut Environment,
    obj: id,
    ivar: ConstPtr<ivar_t>,
    value: id,
) {
    if obj == nil || ivar.is_null() {
        return;
    }

    let offset_ptr = env.mem.read(ivar).offset;
    if offset_ptr.is_null() {
        return;
    }
    let offset = env.mem.read(offset_ptr);
    let destination = MutPtr::<GuestUSize>::from_bits(obj.to_bits() + offset);
    env.mem.write(destination, value.to_bits());
}

/// Returns the compiler-provided Objective-C type encoding for an ivar runtime
/// handle. Ivar handles exposed by this runtime are stable pointers to the
/// guest binary's `ivar_t` metadata, so the returned string has the same
/// lifetime as that image.
pub(super) fn ivar_getTypeEncoding(
    env: &mut Environment,
    ivar: ConstPtr<ivar_t>,
) -> ConstPtr<u8> {
    if ivar.is_null() {
        return Ptr::null();
    }
    env.mem.read(ivar).type_
}

/// Returns the byte offset stored in a compiler-provided ivar runtime handle.
/// Apple's Objective-C runtime also returns zero for a null handle.
pub(super) fn ivar_getOffset(env: &mut Environment, ivar: ConstPtr<ivar_t>) -> GuestISize {
    if ivar.is_null() {
        return 0;
    }
    let offset_ptr = env.mem.read(ivar).offset;
    let offset = env.mem.read(offset_ptr);
    GuestISize::try_from(offset).expect("ivar offset does not fit in guest ptrdiff_t")
}

#[cfg(test)]
mod dev_246_tests {
    use super::*;
    use crate::objc::{msg_class, release, TrivialHostObject};

    #[test]
    fn object_set_ivar_performs_a_nonowning_pointer_sized_store() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let class = env.objc.get_known_class("NSObject", &mut env.mem);
        let offset = env.mem.alloc_and_write(4_u32);
        let name = env.mem.alloc_and_write_cstr(b"_dev246Value");
        let type_encoding = env.mem.alloc_and_write_cstr(b"@");
        let handle = env.mem.alloc_and_write(ivar_t {
            offset: offset.cast_const(),
            name: name.cast_const(),
            type_: type_encoding.cast_const(),
            alignment: 2,
            size: 4,
        });
        {
            let class = env.objc.borrow_mut::<ClassHostObject>(class);
            class.instance_size = class.instance_size.max(8);
            class.ivars.insert(
                "_dev246Value".to_owned(),
                IVar {
                    type_: "@".to_owned(),
                    offset: offset.cast_const(),
                    alignment: 2,
                    handle: handle.cast_const(),
                },
            );
        }
        let object = env
            .objc
            .alloc_object(class, Box::new(TrivialHostObject), &mut env.mem);
        let value: id = msg_class![env; NSObject new];
        let initial_refcount = env.objc.get_refcount(value);

        object_setIvar(env, object, handle.cast_const(), value);

        assert_eq!(env.mem.read(MutPtr::<id>::from_bits(object.to_bits() + 4)), value);
        assert_eq!(env.objc.get_refcount(value), initial_refcount);
        object_setIvar(env, nil, handle.cast_const(), value);
        object_setIvar(env, object, Ptr::null(), value);

        env.objc
            .borrow_mut::<ClassHostObject>(class)
            .ivars
            .remove("_dev246Value");
        release(env, object);
        release(env, value);
        env.mem.free(handle.cast());
        env.mem.free(type_encoding.cast());
        env.mem.free(name.cast());
        env.mem.free(offset.cast());
    }
}

/// Undocumented function (see link above) apparently used by auto-generated
/// methods for properties to get an ivar.
pub(super) fn objc_getProperty(
    env: &mut Environment,
    this: id,
    _cmd: SEL,
    offset: GuestISize,
    atomic: bool,
) -> id {
    // We currently aren't touching the ivar layouts contained in the binary, so
    // we are assuming they are already correctly set by the compiler. Since we
    // aren't using ivars at all in our host classes, we shouldn't have any
    // issues with host classes' ivars clobbering guest classes' ivars, but
    // what if the compiler doesn't set the ivar layout at all? This is a simple
    // safeguard: any real ivar offset will be after the isa pointer.
    assert!(offset >= 4);

    if atomic {
        log_once!("TODO: Lock when atomic is set to true in objc_getProperty");
    }

    let ivar: MutPtr<id> = Ptr::from_bits(this.to_bits().checked_add_signed(offset).unwrap());
    env.mem.read(ivar)
}

/// Undocumented function (see link above) apparently used by auto-generated
/// methods for properties to set an ivar and handle reference counting, copying
/// and locking.
pub(super) fn objc_setProperty(
    env: &mut Environment,
    this: id,
    _cmd: SEL,
    offset: GuestISize,
    value: id,
    atomic: bool,
    should_copy: i8,
) {
    // We currently aren't touching the ivar layouts contained in the binary, so
    // we are assuming they are already correctly set by the compiler. Since we
    // aren't using ivars at all in our host classes, we shouldn't have any
    // issues with host classes' ivars clobbering guest classes' ivars, but
    // what if the compiler doesn't set the ivar layout at all? This is a simple
    // safeguard: any real ivar offset will be after the isa pointer.
    assert!(offset >= 4);

    if atomic {
        log_once!("TODO: Lock when atomic is set to true in objc_setProperty");
    }

    let ivar: MutPtr<id> = Ptr::from_bits(this.to_bits().checked_add_signed(offset).unwrap());
    let old = env.mem.read(ivar);

    let void_null: MutVoidPtr = Ptr::null();
    let value: id = if value != nil {
        match should_copy {
            0 => retain(env, value),
            1 => msg![env; value copyWithZone:void_null],
            2 => msg![env; value mutableCopyWithZone:void_null],
            // Apple's source code implies that any non-zero value that isn't 2
            // should mean "copy", but that seems weird, let's be conservative.
            _ => panic!("Unknown \"should copy\" value: {should_copy}"),
        }
    } else {
        nil
    };
    env.mem.write(ivar, value);

    if old != nil {
        release(env, old);
    }
}

// note: https://opensource.apple.com/source/objc4/objc4-723/runtime/objc-accessors.mm.auto.html
//       says that hasStrong is unused.
pub(super) fn objc_copyStruct(
    env: &mut Environment,
    dest: MutVoidPtr,
    src: ConstVoidPtr,
    size: GuestUSize,
    _atomic: bool,
    _hasStrong: bool,
) {
    // It's safe to ignore atomic as we never switch thread unless we call back
    // into guest code and we're not doing that here, just calling memmove.
    // TODO: implement atomic support
    env.mem.memmove(dest, src, size);
}

/// Logs a placeholder message for an unimplemented ObjC setter
///
/// This macro must be used inside [crate::_objc_method],
/// as it relies on constants for the current class and selector
/// set by it and [crate::objc::objc_classes]
#[macro_export]
macro_rules! todo_objc_setter {
    ($this:ident, $($arg:tt)+) => {
        const _: () = {
            let bytes = _OBJC_CURRENT_SELECTOR.as_bytes();
            let starts_with_set =
                bytes.len() > 3 && bytes[0] == b's' && bytes[1] == b'e' && bytes[2] == b't';
            assert!(starts_with_set, "Selector does not start with set.");
        };
        log!(
            "TODO: [({}*) {:?} {}:{:?}]",
            _OBJC_CURRENT_CLASS,
            $this,
            _OBJC_CURRENT_SELECTOR,
            $($arg)+
        );
    };
}
pub use crate::todo_objc_setter;
