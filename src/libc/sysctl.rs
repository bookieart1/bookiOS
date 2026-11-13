/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `sys/sysctl.h`

use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::errno::{set_errno, EINVAL, ENOENT, ENOMEM};
use crate::mem::{
    guest_size_of, ConstPtr, GuestUSize, MutPtr, MutVoidPtr, SafeRead, PAGE_SIZE,
};
use crate::window::DeviceFamily;
use crate::Environment;

// Top level constants
const CTL_KERN: i32 = 1;
const CTL_NET: i32 = 4;
const CTL_HW: i32 = 6;

// CTL_NET/PF_ROUTE. These values and the 32-bit layouts below come from the
// Darwin 10.4 XNU headers used by iPhone OS 4.x (xnu-1504.15.3).
const PF_ROUTE: i32 = 17;
const AF_LINK: i32 = 18;
const NET_RT_IFLIST: i32 = 3;
const EMULATED_EN0_INDEX: i32 = 2;
const RTM_VERSION: u8 = 5;
const RTM_IFINFO: u8 = 0x0e;
const RTA_IFP: i32 = 0x10;
const IFF_UP: i32 = 0x01;
const IFF_RUNNING: i32 = 0x40;
const IFT_ETHER: u8 = 0x06;
const USER32_IF_MSGHDR_SIZE: usize = 112;
const SOCKADDR_DL_SIZE: usize = 20;
const ROUTE_IFLIST_RECORD_SIZE: GuestUSize =
    (USER32_IF_MSGHDR_SIZE + SOCKADDR_DL_SIZE) as GuestUSize;
const VIRTUAL_EN0_MAC: [u8; 6] = [0x02, 0x74, 0x48, 0x4c, 0x45, 0x01];

// CTL_KERN
const KERN_OSTYPE: i32 = 1;
const KERN_OSRELEASE: i32 = 2;
const KERN_OSREV: i32 = 3;
const KERN_VERSION: i32 = 4;
const KERN_HOSTNAME: i32 = 10;
const KERN_PROC: i32 = 14;
const KERN_BOOTTIME: i32 = 21;
const KERN_OSVERSION: i32 = 65;

// KERN_PROC
const KERN_PROC_ALL: i32 = 0;
const KERN_PROC_PID: i32 = 1;

// 32-bit Darwin's `struct kinfo_proc` is 492 bytes. These offsets are the
// stable beginning of `struct user32_extern_proc`, which is the first member:
// `p_flag` at 16, `p_stat` at 20 and `p_pid` at 24.
const USER32_KINFO_PROC_SIZE: GuestUSize = 492;
const USER32_KINFO_PROC_FLAG_OFFSET: usize = 16;
const USER32_KINFO_PROC_STAT_OFFSET: usize = 20;
const USER32_KINFO_PROC_PID_OFFSET: usize = 24;
const SRUN: u8 = 2;
const EMULATED_PID: i32 = 1;

// CTL_HW
const HW_MACHINE: i32 = 1;
const HW_MODEL: i32 = 2;
const HW_NCPU: i32 = 3;
const HW_PHYSMEM: i32 = 5;
const HW_USERMEM: i32 = 6;
const HW_PAGESIZE: i32 = 7;
const HW_BUS_FREQ: i32 = 14;
const HW_CPU_FREQ: i32 = 15;
const HW_MEMSIZE: i32 = 24;

/// There is probably more idiomatic way to express "a variable sized array of
/// integers, up to 12 max", but this enum would do for now.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum SysCtlNamePath {
    Undefined,
    Length2(i32, i32),
    Length4(i32, i32, i32, i32),
}

// Clippy complains about the type.
// Below values corresponds to the original iPhone.
// Reference https://www.mail-archive.com/misc@openbsd.org/msg80988.html
static SYSCTL_VALUES: [(SysCtlNamePath, &str, SysInfoType); 24] = [
    // Generic CPU, I/O
    (SysCtlNamePath::Length2(CTL_HW, HW_MACHINE), "hw.machine", SysInfoType::String(b"iPhone1,1")),
    (SysCtlNamePath::Length2(CTL_HW, HW_MODEL), "hw.model", SysInfoType::String(b"M68AP")),
    (SysCtlNamePath::Length2(CTL_HW, HW_NCPU), "hw.ncpu", SysInfoType::Int32(1)),
    // Darwin exposes the number of CPUs currently available for scheduling as
    // an int. The emulated single-core iPhone therefore reports the same value
    // as hw.ncpu.
    (SysCtlNamePath::Undefined, "hw.activecpu", SysInfoType::Int32(1)),
    // Darwin reports this maximum as a 32-bit integer. The emulated
    // single-core device has one physical CPU available.
    (SysCtlNamePath::Undefined, "hw.physicalcpu_max", SysInfoType::Int32(1)),
    // XNU exposes logicalcpu_max as a 32-bit integer. The virtual iPhone1,1
    // has one logical processor, matching the rest of the emulated topology.
    (SysCtlNamePath::Undefined, "hw.logicalcpu_max", SysInfoType::Int32(1)),
    // ARM1176JZF-S, used by the original iPhone model reported above, has a
    // 32-byte L1 cache line. Darwin exposes this value as an int.
    (SysCtlNamePath::Undefined, "hw.cachelinesize", SysInfoType::Int32(32)),
    // The original iPhone's S5L8900/ARM1176 has a 16 KiB L1 data cache.
    // Darwin exposes cache byte counts as 64-bit values.
    (SysCtlNamePath::Undefined, "hw.l1dcachesize", SysInfoType::Int64(16 * 1024)),
    (SysCtlNamePath::Undefined, "hw.cputype", SysInfoType::Int32(12)),
    (SysCtlNamePath::Undefined, "hw.cpusubtype", SysInfoType::Int32(6)),
    (SysCtlNamePath::Length2(CTL_HW, HW_CPU_FREQ), "hw.cpufrequency", SysInfoType::Int64(412000000)),
    (SysCtlNamePath::Length2(CTL_HW, HW_BUS_FREQ), "hw.busfrequency", SysInfoType::Int64(103000000)),
    (SysCtlNamePath::Length2(CTL_HW, HW_PHYSMEM), "hw.physmem", SysInfoType::Int32(121634816)),
    (SysCtlNamePath::Length2(CTL_HW, HW_USERMEM), "hw.usermem", SysInfoType::Int32(93564928)),
    (SysCtlNamePath::Length2(CTL_HW, HW_MEMSIZE), "hw.memsize", SysInfoType::Int32(121634816)),
    // `int` on the 32-bit iPhone OS ABI. Returning an i64 here makes callers
    // with the normal four-byte buffer fail with ENOMEM.
    (SysCtlNamePath::Length2(CTL_HW, HW_PAGESIZE), "hw.pagesize", SysInfoType::Int32(PAGE_SIZE as i32)),
    // High kernel limits
    (SysCtlNamePath::Length2(CTL_KERN, KERN_OSTYPE), "kern.ostype", SysInfoType::String(b"Darwin")),
    (SysCtlNamePath::Length2(CTL_KERN, KERN_OSRELEASE), "kern.osrelease", SysInfoType::String(b"10.0.0d3")),
    (SysCtlNamePath::Length2(CTL_KERN, KERN_OSREV), "kern.osrevision", SysInfoType::String(b"199506")),
    (SysCtlNamePath::Length2(CTL_KERN, KERN_HOSTNAME), "kern.hostname", SysInfoType::String(b"touchHLE")), // this is arbitrary
    (SysCtlNamePath::Length2(CTL_KERN, KERN_BOOTTIME), "kern.boottime", SysInfoType::BootTime),
    (SysCtlNamePath::Length2(CTL_KERN, KERN_VERSION), "kern.version", SysInfoType::String(b"Darwin Kernel Version 10.0.0d3: Wed May 13 22:11:58 PDT 2009; root:xnu-1357.2.89~4/RELEASE_ARM_S5L8900X")),
    (SysCtlNamePath::Length2(CTL_KERN, KERN_OSVERSION), "kern.osversion", SysInfoType::String(b"7A341")),
    // Last 0 here is an unnamed placeholder
    (SysCtlNamePath::Length4(CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0), "kern.proc.all", SysInfoType::Struct),
];

static STRING_MAP: LazyLock<HashMap<&str, SysInfoType>> = LazyLock::new(|| {
    // Can't use from_iter because the closure erases the lifetime
    let mut hashmap = HashMap::new();
    for (_, str, value) in SYSCTL_VALUES.iter() {
        hashmap.insert(*str, value.clone());
    }
    hashmap
});

#[allow(clippy::type_complexity)]
static INT_MAP: LazyLock<HashMap<SysCtlNamePath, (&str, SysInfoType)>> = LazyLock::new(|| {
    // Can't use from_iter because the closure erases the lifetime
    let mut hashmap = HashMap::new();
    for (ints, str, value) in SYSCTL_VALUES.iter() {
        if *ints == SysCtlNamePath::Undefined {
            // skip entries which do not have integer name paths
            continue;
        }
        hashmap.insert(*ints, (*str, value.clone()));
    }
    hashmap
});

/// `hw.machine`: the original iPhone's, or the original iPad's for iPad apps.
pub fn hw_machine(env: &Environment) -> &'static [u8] {
    if env.options.device_family == Some(DeviceFamily::iPad) {
        b"iPad1,1"
    } else {
        b"iPhone1,1"
    }
}

/// Values that differ for iPad apps, which run as if on the original iPad.
fn for_device_family(env: &Environment, name: &str, val: SysInfoType) -> SysInfoType {
    match name {
        "hw.machine" => SysInfoType::String(hw_machine(env)),
        "hw.model" if env.options.device_family == Some(DeviceFamily::iPad) => {
            SysInfoType::String(b"K48AP")
        }
        _ => val,
    }
}

#[derive(Clone)]
enum SysInfoType {
    String(&'static [u8]),
    Int32(i32),
    Int64(i64),
    BootTime,
    Struct,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct GuestTimeval {
    tv_sec: i32,
    tv_usec: i32,
}
unsafe impl SafeRead for GuestTimeval {}

fn current_process_kinfo() -> [u8; USER32_KINFO_PROC_SIZE as usize] {
    // XNU zero-initializes the whole record before filling known fields. In
    // particular, p_flag remains clear because the emulated process is not
    // being traced. Keep kernel-only pointers and unsupported accounting data
    // zero rather than fabricating host process state.
    let mut record = [0; USER32_KINFO_PROC_SIZE as usize];
    record[USER32_KINFO_PROC_FLAG_OFFSET..USER32_KINFO_PROC_FLAG_OFFSET + 4]
        .copy_from_slice(&0i32.to_le_bytes());
    record[USER32_KINFO_PROC_STAT_OFFSET] = SRUN;
    record[USER32_KINFO_PROC_PID_OFFSET..USER32_KINFO_PROC_PID_OFFSET + 4]
        .copy_from_slice(&EMULATED_PID.to_le_bytes());
    record
}

fn virtual_en0_iflist_record() -> [u8; ROUTE_IFLIST_RECORD_SIZE as usize] {
    let mut record = [0; ROUTE_IFLIST_RECORD_SIZE as usize];

    // struct if_msghdr. 32-bit Darwin aligns struct if_data to four bytes,
    // so it begins at byte 16 and the whole header is 112 bytes.
    record[0..2].copy_from_slice(&(ROUTE_IFLIST_RECORD_SIZE as u16).to_le_bytes());
    record[2] = RTM_VERSION;
    record[3] = RTM_IFINFO;
    record[4..8].copy_from_slice(&RTA_IFP.to_le_bytes());
    record[8..12].copy_from_slice(&(IFF_UP | IFF_RUNNING).to_le_bytes());
    record[12..14].copy_from_slice(&(EMULATED_EN0_INDEX as u16).to_le_bytes());

    // The beginning of struct if_data contains the link type, address length,
    // Ethernet header length, MTU and line rate. Unsupported statistics stay
    // zero, as they describe the virtual link rather than the host network.
    let if_data = 16;
    record[if_data] = IFT_ETHER;
    record[if_data + 3] = VIRTUAL_EN0_MAC.len() as u8;
    record[if_data + 4] = 14;
    record[if_data + 8..if_data + 12].copy_from_slice(&1500u32.to_le_bytes());
    record[if_data + 16..if_data + 20].copy_from_slice(&54_000_000u32.to_le_bytes());

    // struct sockaddr_dl, immediately after struct if_msghdr. LLADDR() is
    // sdl_data + sdl_nlen, so callers see the six stable virtual MAC bytes.
    let sdl = USER32_IF_MSGHDR_SIZE;
    record[sdl] = SOCKADDR_DL_SIZE as u8;
    record[sdl + 1] = AF_LINK as u8;
    record[sdl + 2..sdl + 4].copy_from_slice(&(EMULATED_EN0_INDEX as u16).to_le_bytes());
    record[sdl + 4] = IFT_ETHER;
    record[sdl + 5] = 3;
    record[sdl + 6] = VIRTUAL_EN0_MAC.len() as u8;
    record[sdl + 8..sdl + 11].copy_from_slice(b"en0");
    record[sdl + 11..sdl + 17].copy_from_slice(&VIRTUAL_EN0_MAC);

    record
}

fn sysctl_route_iflist_en0(
    env: &mut Environment,
    oldp: MutVoidPtr,
    oldlenp: MutPtr<GuestUSize>,
    newp: MutVoidPtr,
    newlen: GuestUSize,
) -> i32 {
    if !newp.is_null() || newlen != 0 || oldlenp.is_null() {
        set_errno(env, EINVAL);
        return -1;
    }

    if oldp.is_null() {
        env.mem.write(oldlenp, ROUTE_IFLIST_RECORD_SIZE);
        return 0;
    }

    let oldlen = env.mem.read(oldlenp);
    if oldlen < ROUTE_IFLIST_RECORD_SIZE {
        // A routing message is indivisible. No complete record was copied.
        env.mem.write(oldlenp, 0);
        set_errno(env, ENOMEM);
        return -1;
    }

    let record = virtual_en0_iflist_record();
    env.mem
        .bytes_at_mut(oldp.cast(), ROUTE_IFLIST_RECORD_SIZE)
        .copy_from_slice(&record);
    env.mem.write(oldlenp, ROUTE_IFLIST_RECORD_SIZE);
    log_dbg!(
        "sysctl() returned 32-bit NET_RT_IFLIST record for virtual en0 index {}",
        EMULATED_EN0_INDEX
    );
    0
}

fn sysctl_kern_proc_pid(
    env: &mut Environment,
    pid: i32,
    oldp: MutVoidPtr,
    oldlenp: MutPtr<GuestUSize>,
    newp: MutVoidPtr,
    newlen: GuestUSize,
) -> i32 {
    if !newp.is_null() || newlen != 0 || oldlenp.is_null() {
        set_errno(env, EINVAL);
        return -1;
    }

    if pid != EMULATED_PID {
        // A valid KERN_PROC_PID lookup for an absent process succeeds with no
        // records, matching XNU's process iteration behavior.
        env.mem.write(oldlenp, 0);
        return 0;
    }

    if oldp.is_null() {
        // XNU adds five records of slop when sizing kern.proc output, even for
        // a PID-filtered query. The subsequent populated call returns one.
        env.mem
            .write(oldlenp, USER32_KINFO_PROC_SIZE.saturating_mul(6));
        return 0;
    }

    let oldlen = env.mem.read(oldlenp);
    if oldlen < USER32_KINFO_PROC_SIZE {
        // With no complete record copied, XNU reports a consumed length of
        // zero and ENOMEM when the required output exceeds that length.
        env.mem.write(oldlenp, 0);
        set_errno(env, ENOMEM);
        return -1;
    }

    let record = current_process_kinfo();
    env.mem
        .bytes_at_mut(oldp.cast(), USER32_KINFO_PROC_SIZE)
        .copy_from_slice(&record);
    env.mem.write(oldlenp, USER32_KINFO_PROC_SIZE);
    log_dbg!(
        "sysctl() returned 32-bit kern.proc.pid record for emulated PID {pid}"
    );
    0
}

fn sysctl(
    env: &mut Environment,
    name: MutPtr<i32>,
    name_len: u32,
    oldp: MutVoidPtr,
    oldlenp: MutPtr<GuestUSize>,
    newp: MutVoidPtr,
    newlen: GuestUSize,
) -> i32 {
    set_errno(env, 0);

    log_dbg!(
        "sysctl({:?}, {:#x}, {:?}, {:?}, {:?}, {:x})",
        name,
        name_len,
        oldp,
        oldlenp,
        newp,
        newlen
    );
    match name_len {
        2 => {
            let (name0, name1) = (env.mem.read(name), env.mem.read(name + 1));
            sysctl_generic(
                env,
                |_| {
                    let Some(val) = INT_MAP.get(&SysCtlNamePath::Length2(name0, name1)).cloned()
                    else {
                        unimplemented!("Unknown sysctl parameter ({name0}, {name1})!")
                    };
                    val
                },
                oldp,
                oldlenp,
                newp,
                newlen,
            )
        }
        4 => {
            let (name0, name1, name2, name3) = (
                env.mem.read(name),
                env.mem.read(name + 1),
                env.mem.read(name + 2),
                env.mem.read(name + 3),
            );
            if name0 == CTL_KERN && name1 == KERN_PROC {
                if name2 == KERN_PROC_PID {
                    return sysctl_kern_proc_pid(
                        env, name3, oldp, oldlenp, newp, newlen,
                    );
                }
                // In some Unity games, mono initialization set-ups perf
                // counters, which set-ups a shared memory area, which gets
                // a list of all processes via sysctl() if shm_open() fails.
                // (See mono's [mono-perfcounters.c](https://github.com/mono/mono/blob/62121afbb28f0b62f100ec9a942d10c5e0f4814f/mono/metadata/mono-perfcounters.c#L392)
                // and [mono-mmap.c](https://github.com/mono/mono/blob/0f53e9e151d92944cacab3e24ac359410c606df6/mono/utils/mono-mmap.c#L555))
                // Stubbing that sysctl() call doesn't seem to have any impact
                // on the games' functionality.
                //
                // ...but wait, why would perf counters need a shared memory
                // area in the first place? This is left as an exercise for
                // the reader.
                set_errno(env, ENOENT);
                log!(
                    "TODO: sysctl() for kern.proc query ({name2}, {name3}), returning -1"
                );
                return -1;
            }
            sysctl_generic(
                env,
                |_| {
                    let Some(val) = INT_MAP
                        .get(&SysCtlNamePath::Length4(name0, name1, name2, name3))
                        .cloned()
                    else {
                        unimplemented!(
                            "Unknown sysctl parameter ({name0}, {name1}, {name2}, {name3})!"
                        )
                    };
                    val
                },
                oldp,
                oldlenp,
                newp,
                newlen,
            )
        }
        6 => {
            let path = std::array::from_fn(|index| env.mem.read(name + index as u32));
            if path
                == [
                    CTL_NET,
                    PF_ROUTE,
                    0,
                    AF_LINK,
                    NET_RT_IFLIST,
                    EMULATED_EN0_INDEX,
                ]
            {
                return sysctl_route_iflist_en0(env, oldp, oldlenp, newp, newlen);
            }
            unimplemented!("Unknown six-component sysctl parameter {path:?}!")
        }
        _ => unimplemented!("sysctl() for name length {name_len} is unimplemented!"),
    }
}

fn sysctlbyname(
    env: &mut Environment,
    name: ConstPtr<u8>,
    oldp: MutVoidPtr,
    oldlenp: MutPtr<GuestUSize>,
    newp: MutVoidPtr,
    newlen: GuestUSize,
) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    let name_str = env.mem.cstr_at_utf8(name).unwrap();
    log_dbg!(
        "sysctlbyname({:?}, {:?}, {:?}, {:?}, {:x})",
        name_str,
        oldp,
        oldlenp,
        newp,
        newlen
    );
    // The S5L8900 in the original iPhone model reported by this emulator has
    // no L2 cache. Historical Darwin therefore has no value for this optional
    // name: callers must receive the normal "name not found" result rather
    // than an invented cache size or an emulator panic.
    if name_str == "hw.l2cachesize" || name_str == "hw.l3cachesize" {
        log_dbg!("sysctlbyname({name_str:?}) is unavailable on iPhone1,1");
        set_errno(env, ENOENT);
        return -1;
    }
    sysctl_generic(
        env,
        |env| {
            let name_str = env.mem.cstr_at_utf8(name).unwrap();
            let Some((name_str, val)) = STRING_MAP.get_key_value(name_str) else {
                unimplemented!("Unknown sysctlbyname parameter {name_str}!")
            };
            (name_str, val.clone())
        },
        oldp,
        oldlenp,
        newp,
        newlen,
    )
}

fn sysctl_generic<F>(
    env: &mut Environment,
    // Returns the name and value of the property (or exits)
    name_lookup: F,
    oldp: MutVoidPtr,
    oldlenp: MutPtr<GuestUSize>,
    newp: MutVoidPtr,
    newlen: GuestUSize,
) -> i32
where
    F: FnOnce(&mut Environment) -> (&'static str, SysInfoType),
{
    assert!(newp.is_null());
    assert_eq!(newlen, 0);

    let (name_str, val) = name_lookup(env);
    let val = for_device_family(env, name_str, val);
    let len: GuestUSize = match val {
        SysInfoType::String(str) => str.len() as GuestUSize + 1,
        SysInfoType::Int32(_) => guest_size_of::<i32>(),
        SysInfoType::Int64(_) => guest_size_of::<i64>(),
        SysInfoType::BootTime => guest_size_of::<GuestTimeval>(),
        _ => unimplemented!(),
    };
    if oldp.is_null() {
        env.mem.write(oldlenp, len);
        return 0;
    }
    assert!(!oldp.is_null() && !oldlenp.is_null());
    let oldlen = env.mem.read(oldlenp);
    if oldlen < len {
        // TODO: set errno
        // TODO: write partial data
        log!("sysctl(byname) for '{name_str}': the buffer of size {oldlen} is too low to fit the value of size {len}, returning -1");
        return -1;
    }
    match val {
        SysInfoType::String(str) => {
            let sysctl_str = env.mem.alloc_and_write_cstr(str);
            env.mem.memmove(oldp, sysctl_str.cast().cast_const(), len);
            env.mem.free(sysctl_str.cast());
        }
        SysInfoType::Int32(num) => {
            env.mem.write(oldp.cast(), num);
        }
        SysInfoType::Int64(num) => {
            env.mem.write(oldp.cast(), num);
        }
        SysInfoType::BootTime => {
            // KERN_BOOTTIME is a 32-bit Darwin timeval. A plausible boot time
            // is sufficient for legacy callers that use it to derive uptime.
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let tv_sec = now.saturating_sub(3600).min(i32::MAX as u64) as i32;
            env.mem.write(
                oldp.cast(),
                GuestTimeval {
                    tv_sec,
                    tv_usec: 0,
                },
            );
        }
        _ => unimplemented!(),
    }
    env.mem.write(oldlenp, len);
    0 // success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sysctl(_, _, _, _, _, _)),
    export_c_func!(sysctlbyname(_, _, _, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libc::errno::get_errno;
    use crate::mem::Ptr;

    #[test]
    fn development_255_l3_cache_query_returns_historical_absence() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.l3cachesize");
        let output = env.mem.alloc_and_write(0xfeed_beefu32);
        let output_len = env.mem.alloc_and_write(guest_size_of::<u32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output.cast(),
                output_len,
                Ptr::null(),
                0,
            ),
            -1
        );
        assert_eq!(get_errno(&mut env), ENOENT);
        assert_eq!(env.mem.read(output), 0xfeed_beefu32);
    }

    fn kern_proc_pid_name(env: &mut Environment, pid: i32) -> MutPtr<i32> {
        let name = env.mem.alloc(guest_size_of::<i32>() * 4).cast::<i32>();
        for (index, value) in [CTL_KERN, KERN_PROC, KERN_PROC_PID, pid]
            .into_iter()
            .enumerate()
        {
            env.mem.write(name + index as u32, value);
        }
        name
    }

    fn route_iflist_en0_name(env: &mut Environment) -> MutPtr<i32> {
        let values = [
            CTL_NET,
            PF_ROUTE,
            0,
            AF_LINK,
            NET_RT_IFLIST,
            EMULATED_EN0_INDEX,
        ];
        let name = env
            .mem
            .alloc(guest_size_of::<i32>() * values.len() as GuestUSize)
            .cast::<i32>();
        for (index, value) in values.into_iter().enumerate() {
            env.mem.write(name + index as u32, value);
        }
        name
    }

    #[test]
    fn active_cpu_reports_one_32_bit_guest_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.activecpu");
        let output = env.mem.alloc(guest_size_of::<i32>());
        let output_len = env.mem.alloc_and_write(guest_size_of::<i32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i32>());
        assert_eq!(env.mem.read(output.cast::<i32>()), 1);
    }

    #[test]
    fn physical_cpu_max_reports_one_32_bit_guest_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.physicalcpu_max");
        let output = env.mem.alloc(guest_size_of::<i32>());
        let output_len = env.mem.alloc_and_write(guest_size_of::<i32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i32>());
        assert_eq!(env.mem.read(output.cast::<i32>()), 1);
    }

    #[test]
    fn development_256_logical_cpu_max_reports_one_32_bit_guest_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.logicalcpu_max");
        let output_len = env.mem.alloc_and_write(0);

        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                Ptr::null(),
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i32>());

        let output = env.mem.alloc_and_write(0x7f7f_7f7fi32);
        env.mem.write(output_len, guest_size_of::<i32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output.cast(),
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i32>());
        assert_eq!(env.mem.read(output), 1);
    }

    #[test]
    fn cache_line_size_matches_the_reported_original_iphone_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.cachelinesize");
        let output = env.mem.alloc(guest_size_of::<i32>());
        let output_len = env.mem.alloc_and_write(guest_size_of::<i32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i32>());
        assert_eq!(env.mem.read(output.cast::<i32>()), 32);
    }

    #[test]
    fn l1_data_cache_size_matches_the_reported_original_iphone_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.l1dcachesize");
        let output = env.mem.alloc(guest_size_of::<i64>());
        let output_len = env.mem.alloc_and_write(guest_size_of::<i64>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), guest_size_of::<i64>());
        assert_eq!(env.mem.read(output.cast::<i64>()), 16 * 1024);
    }

    #[test]
    fn l2_cache_size_is_absent_on_the_reported_original_iphone_cpu() {
        let mut env = Environment::new_fake_for_test();
        let name = env.mem.alloc_and_write_cstr(b"hw.l2cachesize");
        let output = env.mem.alloc_and_write(0xfeed_beefu32);
        let output_len = env.mem.alloc_and_write(guest_size_of::<u32>());
        assert_eq!(
            sysctlbyname(
                &mut env,
                name.cast_const(),
                output.cast(),
                output_len,
                Ptr::null(),
                0,
            ),
            -1
        );
        assert_eq!(get_errno(&mut env), ENOENT);
        assert_eq!(env.mem.read(output), 0xfeed_beefu32);
        assert_eq!(env.mem.read(output_len), guest_size_of::<u32>());
    }

    #[test]
    fn route_iflist_returns_a_32_bit_virtual_en0_record() {
        let mut env = Environment::new_fake_for_test();
        let name = route_iflist_en0_name(&mut env);
        let output_len = env.mem.alloc_and_write(0);

        assert_eq!(
            sysctl(&mut env, name, 6, Ptr::null(), output_len, Ptr::null(), 0,),
            0
        );
        assert_eq!(env.mem.read(output_len), ROUTE_IFLIST_RECORD_SIZE);

        let output = env.mem.alloc(ROUTE_IFLIST_RECORD_SIZE);
        env.mem.write(output_len, ROUTE_IFLIST_RECORD_SIZE);
        assert_eq!(
            sysctl(&mut env, name, 6, output, output_len, Ptr::null(), 0,),
            0
        );
        let bytes = env
            .mem
            .bytes_at(output.cast::<u8>(), ROUTE_IFLIST_RECORD_SIZE);
        assert_eq!(u16::from_le_bytes(bytes[0..2].try_into().unwrap()), 132);
        assert_eq!(&bytes[112..120], &[20, 18, 2, 0, 6, 3, 6, 0]);
        assert_eq!(&bytes[120..123], b"en0");
        assert_eq!(&bytes[123..129], &VIRTUAL_EN0_MAC);
    }

    #[test]
    fn route_iflist_rejects_a_short_buffer_without_a_partial_record() {
        let mut env = Environment::new_fake_for_test();
        let name = route_iflist_en0_name(&mut env);
        let output = env.mem.alloc(ROUTE_IFLIST_RECORD_SIZE - 1);
        let output_len = env.mem.alloc_and_write(ROUTE_IFLIST_RECORD_SIZE - 1);

        assert_eq!(
            sysctl(&mut env, name, 6, output, output_len, Ptr::null(), 0,),
            -1
        );
        assert_eq!(env.mem.read(output_len), 0);
        assert_eq!(get_errno(&mut env), ENOMEM);
    }

    #[test]
    fn kern_proc_pid_returns_untraced_current_process() {
        let mut env = Environment::new_fake_for_test();
        let name = kern_proc_pid_name(&mut env, EMULATED_PID);
        let output = env.mem.alloc(USER32_KINFO_PROC_SIZE);
        let output_len = env.mem.alloc_and_write(USER32_KINFO_PROC_SIZE);

        assert_eq!(
            sysctl(
                &mut env,
                name,
                4,
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            0
        );
        assert_eq!(env.mem.read(output_len), USER32_KINFO_PROC_SIZE);
        let bytes = env
            .mem
            .bytes_at(output.cast::<u8>(), USER32_KINFO_PROC_SIZE);
        assert_eq!(
            i32::from_le_bytes(
                bytes[USER32_KINFO_PROC_FLAG_OFFSET..USER32_KINFO_PROC_FLAG_OFFSET + 4]
                    .try_into()
                    .unwrap()
            ),
            0
        );
        assert_eq!(bytes[USER32_KINFO_PROC_STAT_OFFSET], SRUN);
        assert_eq!(
            i32::from_le_bytes(
                bytes[USER32_KINFO_PROC_PID_OFFSET..USER32_KINFO_PROC_PID_OFFSET + 4]
                    .try_into()
                    .unwrap()
            ),
            EMULATED_PID
        );
    }

    #[test]
    fn kern_proc_pid_rejects_a_short_buffer() {
        let mut env = Environment::new_fake_for_test();
        let name = kern_proc_pid_name(&mut env, EMULATED_PID);
        let output = env.mem.alloc(USER32_KINFO_PROC_SIZE - 1);
        let output_len = env.mem.alloc_and_write(USER32_KINFO_PROC_SIZE - 1);

        assert_eq!(
            sysctl(
                &mut env,
                name,
                4,
                output,
                output_len,
                Ptr::null(),
                0,
            ),
            -1
        );
        assert_eq!(env.mem.read(output_len), 0);
        assert_eq!(get_errno(&mut env), ENOMEM);
    }
}
