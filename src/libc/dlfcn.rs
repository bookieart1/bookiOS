/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `dlfcn.h` (`dlopen()` and friends)

use crate::abi::{CallFromHost, GuestFunction};
use crate::cpu::Cpu;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::Environment;
use std::collections::HashMap;

const RTLD_DEFAULT: MutVoidPtr = Ptr::from_bits(-2 as _);

#[derive(Default)]
pub struct State {
    /// C strings returned by `dladdr()`, which the caller doesn't free, so
    /// that each one is only allocated once.
    dladdr_strings: HashMap<String, ConstPtr<u8>>,
    /// Callbacks registered through `__dyld_register_func_for_add_image`.
    /// Dynamic guest loading is not implemented yet, but retaining these
    /// registrations preserves the contract for that future path.
    add_image_callbacks: Vec<GuestFunction>,
}

#[allow(non_camel_case_types)]
#[repr(C, packed)]
struct Dl_info {
    dli_fname: ConstPtr<u8>,
    dli_fbase: ConstVoidPtr,
    dli_sname: ConstPtr<u8>,
    dli_saddr: ConstVoidPtr,
}
unsafe impl SafeRead for Dl_info {}

fn is_known_library(path: &str) -> bool {
    crate::dyld::DYLIB_LIST
        .iter()
        .any(|dylib| dylib.path == path || dylib.aliases.contains(&path))
}

fn dlopen(env: &mut Environment, path: ConstPtr<u8>, _mode: i32) -> MutVoidPtr {
    if path.is_null() {
        return RTLD_DEFAULT;
    }
    // TODO: dlopen() support for real dynamic libraries.
    assert!(is_known_library(env.mem.cstr_at_utf8(path).unwrap()));
    // For convenience, use the path as the handle.
    // TODO: Find out whether the handle is truly opaque on iPhone OS, and if
    // not, where it points.
    path.cast_mut().cast()
}

fn dlsym(env: &mut Environment, handle: MutVoidPtr, symbol: ConstPtr<u8>) -> MutVoidPtr {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    // For some reason, the symbols passed to dlsym() don't have the leading _.
    let symbol = format!("_{}", env.mem.cstr_at_utf8(symbol).unwrap());

    // RTLD_DEFAULT searches the process-wide symbol scope. That includes the
    // main executable and its loaded guest dylibs, not only the iPhone OS
    // functions implemented by touchHLE. Older Unity players use this path for
    // native functions exported by their own executable (via __Internal
    // P/Invoke).
    // The Mach-O loader has already applied the image slide and preserved the
    // Thumb bit in exported_symbols, so the address can be returned directly.
    if handle == RTLD_DEFAULT {
        if let Some((binary, &addr)) = env
            .bins
            .iter()
            .find_map(|binary| binary.exported_symbols.get(&symbol).map(|addr| (binary, addr)))
        {
            log_dbg!(
                "dlsym(RTLD_DEFAULT, {symbol}) resolved to guest symbol at {addr:#x} in {}",
                binary.name,
            );
            return Ptr::from_bits(addr);
        }
    }

    // TODO: Symbol lookup should be scoped to the specific library requested,
    // where appropriate!
    let addr = match env
        .dyld
        .create_proc_address(&mut env.mem, &mut env.cpu, &symbol)
    {
        Ok(addr) => addr,
        Err(()) => {
            // dlsym is commonly used to probe for optional functionality. Its
            // documented failure result is NULL, allowing the caller to fall
            // back or disable that feature rather than crashing the process.
            let caller = env.cpu.regs()[Cpu::LR];
            let context = env.bins.iter().find_map(|binary| {
                let base = binary.text_segment_base?;
                if !(base <= caller && caller < binary.last_segment_end) {
                    return None;
                }
                let nearest = binary
                    .exported_symbols
                    .iter()
                    .filter(|(_, &address)| (address & !1) <= caller)
                    .max_by_key(|(_, &address)| address & !1)
                    .map(|(name, &address)| {
                        format!(
                            "{}!{}+{:#x}",
                            binary.name,
                            name,
                            caller.saturating_sub(address & !1)
                        )
                    })
                    .unwrap_or_else(|| {
                        format!("{}+{:#x}", binary.name, caller.saturating_sub(base))
                    });
                Some(nearest)
            });
            if let Some(context) = context {
                log!(
                    "Warning: dlsym() could not find {symbol}, returning NULL (guest caller {caller:#010x}, {context})"
                );
            } else {
                log!(
                    "Warning: dlsym() could not find {symbol}, returning NULL (guest caller {caller:#010x})"
                );
            }
            return Ptr::null();
        }
    };
    Ptr::from_bits(addr.addr_with_thumb_bit())
}

/// Get a C string for `dladdr()` to return, allocating it the first time.
fn dladdr_string(env: &mut Environment, string: String) -> ConstPtr<u8> {
    if let Some(&ptr) = env.libc_state.dlfcn.dladdr_strings.get(&string) {
        return ptr;
    }
    let ptr = env.mem.alloc_and_write_cstr(string.as_bytes()).cast_const();
    env.libc_state.dlfcn.dladdr_strings.insert(string, ptr);
    ptr
}

/// Find which loaded binary contains `addr`, and the nearest symbol at or
/// before it. Returns 0 if no binary contains it.
fn dladdr(env: &mut Environment, addr: ConstVoidPtr, info: MutPtr<Dl_info>) -> i32 {
    let addr = addr.to_bits();
    let Some((bin_idx, base)) = env.bins.iter().enumerate().find_map(|(i, bin)| {
        let base = bin.text_segment_base?;
        (base <= addr && addr < bin.last_segment_end).then_some((i, base))
    }) else {
        log!("dladdr({:#x}): not in any loaded binary", addr);
        return 0;
    };

    let bin = &env.bins[bin_idx];
    let file_name = if bin_idx == 0 {
        // The app's own executable.
        env.bundle.executable_path().as_str().to_string()
    } else {
        bin.name.clone()
    };
    // Thumb function symbols have the Thumb bit set, so ignore it when
    // comparing, but return the address with it, as it's how the function is
    // called.
    let symbol = bin
        .exported_symbols
        .iter()
        .filter(|(_, &sym_addr)| (sym_addr & !1) <= addr)
        .max_by_key(|(_, &sym_addr)| sym_addr & !1)
        .map(|(name, &sym_addr)| {
            // Symbol names are reported without the leading underscore, as C
            // code would refer to them.
            (name.strip_prefix('_').unwrap_or(name).to_string(), sym_addr)
        });

    let dli_fname = dladdr_string(env, file_name);
    let (dli_sname, dli_saddr) = match symbol {
        Some((name, sym_addr)) => (dladdr_string(env, name), Ptr::from_bits(sym_addr)),
        None => (Ptr::null(), Ptr::null()),
    };
    env.mem.write(
        info,
        Dl_info {
            dli_fname,
            dli_fbase: Ptr::from_bits(base),
            dli_sname,
            dli_saddr,
        },
    );
    1
}

fn dlclose(env: &mut Environment, handle: MutVoidPtr) -> i32 {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    0 // success
}

fn _dyld_get_image_header(env: &mut Environment, image_index: u32) -> ConstVoidPtr {
    env.bins
        .get(image_index as usize)
        .and_then(|bin| bin.text_segment_base)
        .map_or(Ptr::null(), Ptr::from_bits)
}

fn _dyld_image_count(env: &mut Environment) -> u32 {
    env.bins.len().try_into().unwrap_or(u32::MAX)
}

fn _dyld_get_image_name(env: &mut Environment, image_index: u32) -> ConstPtr<u8> {
    let Some(bin) = env.bins.get(image_index as usize) else {
        return Ptr::null();
    };
    let name = if image_index == 0 {
        env.bundle.executable_path().as_str().to_string()
    } else {
        bin.name.clone()
    };
    dladdr_string(env, name)
}

fn _dyld_get_image_vmaddr_slide(_env: &mut Environment, _image_index: u32) -> i32 {
    // touchHLE loads each image at the virtual addresses recorded in its
    // Mach-O image, so there is no additional guest-visible ASLR slide.
    0
}

fn register_add_image_callback(
    env: &mut Environment,
    callback: GuestFunction,
) -> Vec<(ConstVoidPtr, i32)> {
    assert!(!callback.to_ptr().is_null());

    let existing_images = env
        .bins
        .iter()
        .filter_map(|bin| bin.text_segment_base)
        .map(|header| (Ptr::from_bits(header), 0i32))
        .collect();
    env.libc_state.dlfcn.add_image_callbacks.push(callback);
    existing_images
}

fn __dyld_register_func_for_add_image(env: &mut Environment, callback: GuestFunction) {
    // Registration is synchronous: dyld first reports every image already in
    // the process, in load order. Collect the headers before entering guest
    // code so the callback can safely inspect or patch the images reentrantly.
    for (header, slide) in register_add_image_callback(env, callback) {
        () = callback.call_from_host(env, (header, slide));
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dlopen(_, _)),
    export_c_func!(dlsym(_, _)),
    export_c_func!(dlclose(_)),
    export_c_func!(dladdr(_, _)),
    export_c_func!(_dyld_get_image_header(_)),
    export_c_func!(_dyld_image_count()),
    export_c_func!(_dyld_get_image_name(_)),
    export_c_func!(_dyld_get_image_vmaddr_slide(_)),
    export_c_func!(__dyld_register_func_for_add_image(_)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_255_dyld_add_image_callback_is_exported() {
        assert!(
            FUNCTIONS
                .iter()
                .any(|(name, _)| *name == "___dyld_register_func_for_add_image")
        );
    }

    #[test]
    fn development_255_dyld_add_image_registration_reports_existing_images() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.bins.push(crate::mach_o::MachO {
            name: "fixture".to_owned(),
            dynamic_libraries: Vec::new(),
            sections: Vec::new(),
            exported_symbols: HashMap::new(),
            external_relocations: Vec::new(),
            entry_point_pc: None,
            text_segment_base: Some(0x1234_5000),
            last_segment_end: 0x1234_6000,
        });

        let callback = GuestFunction::from_addr_with_thumb_bit(0x2000);
        let notifications = register_add_image_callback(&mut env, callback);

        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].0.to_bits(), 0x1234_5000);
        assert_eq!(notifications[0].1, 0);
        assert_eq!(env.libc_state.dlfcn.add_image_callbacks.len(), 1);
    }
}
