/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `Mach-O` related functions.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, GuestUSize, MutPtr, Ptr, SafeRead};
use crate::Environment;
use std::collections::HashMap;

const CPU_TYPE_ARM: i32 = 12;
const CPU_SUBTYPE_MULTIPLE: i32 = -1;
const CPU_SUBTYPE_MASK: u32 = 0xff00_0000;
const NX_LITTLE_ENDIAN: i32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct NXArchInfo {
    name: ConstPtr<u8>,
    cputype: i32,
    cpusubtype: i32,
    byteorder: i32,
    description: ConstPtr<u8>,
}
unsafe impl SafeRead for NXArchInfo {}

#[derive(Default)]
pub struct State {
    arch_infos: HashMap<(i32, i32), ConstPtr<NXArchInfo>>,
}

struct ArchDescriptor {
    name: &'static [u8],
    cputype: i32,
    cpusubtype: i32,
    description: &'static [u8],
}

fn arch_descriptor(cputype: i32, cpusubtype: i32) -> Option<ArchDescriptor> {
    if cputype != CPU_TYPE_ARM {
        return None;
    }
    let cpusubtype = if cpusubtype == CPU_SUBTYPE_MULTIPLE {
        0
    } else {
        (cpusubtype as u32 & !CPU_SUBTYPE_MASK) as i32
    };
    let (name, description): (&[u8], &[u8]) = match cpusubtype {
        0 => (b"arm", b"ARM"),
        5 => (b"armv4t", b"arm v4t"),
        6 => (b"armv6", b"arm v6"),
        7 => (b"armv5", b"arm v5"),
        8 => (b"xscale", b"arm xscale"),
        9 => (b"armv7", b"arm v7"),
        10 => (b"armv7f", b"arm v7f"),
        11 => (b"armv7s", b"arm v7s"),
        12 => (b"armv7k", b"arm v7k"),
        _ => return None,
    };
    Some(ArchDescriptor {
        name,
        cputype,
        cpusubtype,
        description,
    })
}

fn NXGetArchInfoFromCpuType(
    env: &mut Environment,
    cputype: i32,
    cpusubtype: i32,
) -> ConstPtr<NXArchInfo> {
    let Some(descriptor) = arch_descriptor(cputype, cpusubtype) else {
        return Ptr::null();
    };
    let key = (descriptor.cputype, descriptor.cpusubtype);
    if let Some(&arch_info) = env.libc_state.mach_o.arch_infos.get(&key) {
        return arch_info;
    }

    let name = env.mem.alloc_and_write_cstr(descriptor.name).cast_const();
    let description = env
        .mem
        .alloc_and_write_cstr(descriptor.description)
        .cast_const();
    let arch_info = env
        .mem
        .alloc_and_write(NXArchInfo {
            name,
            cputype: descriptor.cputype,
            cpusubtype: descriptor.cpusubtype,
            byteorder: NX_LITTLE_ENDIAN,
            description,
        })
        .cast_const();
    env.libc_state.mach_o.arch_infos.insert(key, arch_info);
    arch_info
}

fn _NSGetExecutablePath(env: &mut Environment, buf: MutPtr<u8>, buf_size: MutPtr<u32>) -> i32 {
    let binding = env.bundle.executable_path();
    let bin_path = binding.as_str();

    let size = env.mem.read(buf_size);
    let len: GuestUSize = bin_path.len().try_into().unwrap();
    let required_size = len.checked_add(1).unwrap();
    if size < required_size {
        env.mem.write(buf_size, required_size);
        return -1;
    }

    env.mem
        .bytes_at_mut(buf, len)
        .copy_from_slice(bin_path.as_bytes());
    env.mem.write(buf + len, b'\0');
    0
}

fn get_end(env: &mut Environment) -> u32 {
    // Assume app binary is the first.
    // From https://www.manpagez.com/man/3/get_end/
    // `In a Mach-O file <...> get_end returns the first address after
    // the last segment in the executable`
    // It was confirmed on a real device with the TestApp binary.
    env.bins[0].last_segment_end
}

fn get_etext(env: &mut Environment) -> u32 {
    // Assume app binary is the first.
    let app_sections = &env.bins[0].sections;
    assert_eq!(
        app_sections
            .iter()
            .filter(|s| s.name.to_uppercase() == "__TEXT")
            .count(),
        1
    );
    let text_section = app_sections
        .iter()
        .find(|s| s.name.to_uppercase() == "__TEXT")
        .unwrap();
    text_section.next_section_addr()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NXGetArchInfoFromCpuType(_, _)),
    export_c_func!(_NSGetExecutablePath(_, _)),
    export_c_func!(get_end()),
    export_c_func!(get_etext()),
];

#[cfg(test)]
mod tests {
    use super::{arch_descriptor, CPU_TYPE_ARM};

    #[test]
    fn recognizes_legacy_arm_architectures() {
        let armv7 = arch_descriptor(CPU_TYPE_ARM, 9).unwrap();
        assert_eq!(armv7.name, b"armv7");
        assert_eq!(armv7.description, b"arm v7");

        let family = arch_descriptor(CPU_TYPE_ARM, -1).unwrap();
        assert_eq!(family.name, b"arm");
        assert_eq!(family.cpusubtype, 0);

        let armv7s = arch_descriptor(CPU_TYPE_ARM, 11).unwrap();
        assert_eq!(armv7s.name, b"armv7s");
    }

    #[test]
    fn rejects_unknown_architectures() {
        assert!(arch_descriptor(7, 3).is_none());
        assert!(arch_descriptor(CPU_TYPE_ARM, 123).is_none());
    }
}
