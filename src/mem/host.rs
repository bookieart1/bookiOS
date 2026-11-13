/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Cross-platform memory management wrappers using the host's system calls.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static NEXT_ALLOCATION_GENERATION: AtomicU64 = AtomicU64::new(1);
static LIVE_MAPPINGS: AtomicUsize = AtomicUsize::new(0);
static GUEST_MEMORY_INFRASTRUCTURE_VALID: AtomicBool = AtomicBool::new(true);

pub(super) struct Allocation {
    address: *mut core::ffi::c_void,
    size: usize,
    generation: u64,
    released: bool,
}

impl Allocation {
    pub(super) fn new(size: usize) -> std::io::Result<Self> {
        let generation = NEXT_ALLOCATION_GENERATION.fetch_add(1, Ordering::Relaxed);
        let result = unsafe { allocate_memory(size) };
        match result {
            Ok(address) => {
                let live = LIVE_MAPPINGS.fetch_add(1, Ordering::SeqCst) + 1;
                echo_no_panic!(
                    "Guest memory allocation: generation={} address={:?} requested_size={} success=true live_mapping_count={} host_vm={}",
                    generation,
                    address,
                    size,
                    live,
                    host_virtual_memory_state()
                );
                Ok(Self {
                    address,
                    size,
                    generation,
                    released: false,
                })
            }
            Err(error) => {
                GUEST_MEMORY_INFRASTRUCTURE_VALID.store(false, Ordering::SeqCst);
                echo_no_panic!(
                    "Guest memory allocation: generation={} address=null requested_size={} success=false error={} live_mapping_count={} host_vm={}",
                    generation,
                    size,
                    error,
                    LIVE_MAPPINGS.load(Ordering::SeqCst),
                    host_virtual_memory_state()
                );
                Err(error)
            }
        }
    }

    pub(super) fn address(&self) -> *mut core::ffi::c_void {
        self.address
    }

    pub(super) fn release(&mut self) -> std::io::Result<()> {
        if self.released {
            return Ok(());
        }
        let result = unsafe { free_memory(self.address, self.size) };
        if result.is_ok() {
            self.released = true;
            let live = LIVE_MAPPINGS.fetch_sub(1, Ordering::SeqCst) - 1;
            echo_no_panic!(
                "Guest memory release: generation={} address={:?} requested_size={} success=true live_mapping_count={} host_vm={}",
                self.generation,
                self.address,
                self.size,
                live,
                host_virtual_memory_state()
            );
        } else if let Err(error) = &result {
            GUEST_MEMORY_INFRASTRUCTURE_VALID.store(false, Ordering::SeqCst);
            echo_no_panic!(
                "Guest memory release: generation={} address={:?} requested_size={} success=false error={} live_mapping_count={} host_vm={}",
                self.generation,
                self.address,
                self.size,
                error,
                LIVE_MAPPINGS.load(Ordering::SeqCst),
                host_virtual_memory_state()
            );
        }
        result
    }
}

impl Drop for Allocation {
    fn drop(&mut self) {
        self.release()
            .expect("Failed to release the guest address-space mapping");
    }
}

pub(super) fn live_mapping_count() -> usize {
    LIVE_MAPPINGS.load(Ordering::SeqCst)
}

pub(super) fn infrastructure_valid() -> bool {
    GUEST_MEMORY_INFRASTRUCTURE_VALID.load(Ordering::SeqCst)
}

#[cfg(target_vendor = "apple")]
fn host_virtual_memory_state() -> String {
    let mut info: libc::mach_task_basic_info = unsafe { std::mem::zeroed() };
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    let status = unsafe {
        libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO,
            (&mut info as *mut libc::mach_task_basic_info).cast(),
            &mut count,
        )
    };
    if status == 0 {
        // Some Apple SDK bindings model this C structure as packed. Copy the
        // fields before formatting so Rust never creates an unaligned reference.
        let virtual_size = info.virtual_size;
        let resident_size = info.resident_size;
        let resident_size_max = info.resident_size_max;
        format!(
            "virtual_size={} resident_size={} resident_size_max={}",
            virtual_size, resident_size, resident_size_max
        )
    } else {
        format!("mach_task_basic_info_error={status}")
    }
}

#[cfg(windows)]
fn host_virtual_memory_state() -> String {
    #[allow(non_snake_case)]
    #[repr(C)]
    struct MemoryStatusEx {
        dwLength: u32,
        dwMemoryLoad: u32,
        ullTotalPhys: u64,
        ullAvailPhys: u64,
        ullTotalPageFile: u64,
        ullAvailPageFile: u64,
        ullTotalVirtual: u64,
        ullAvailVirtual: u64,
        ullAvailExtendedVirtual: u64,
    }
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
    }
    let mut status: MemoryStatusEx = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MemoryStatusEx>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        let total_virtual = status.ullTotalVirtual;
        let available_virtual = status.ullAvailVirtual;
        let memory_load = status.dwMemoryLoad;
        format!(
            "total_virtual={} available_virtual={} memory_load_percent={}",
            total_virtual, available_virtual, memory_load
        )
    } else {
        format!(
            "GlobalMemoryStatusEx_error={}",
            std::io::Error::last_os_error()
        )
    }
}

#[cfg(all(unix, not(target_vendor = "apple")))]
fn host_virtual_memory_state() -> String {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } == 0 {
        format!("max_resident_set={}", usage.ru_maxrss)
    } else {
        format!("getrusage_error={}", std::io::Error::last_os_error())
    }
}

/// Cross-platform memory allocation using the host's system calls.
/// Returns an address aligned to the guest's 4KB page boundaries.
///
/// - The function returns a raw pointer to allocated memory.
///   The caller is responsible for managing that memory.
/// - The returned pointer must be freed using the corresponding
///   [`free_memory`] call.
#[cfg(windows)]
pub(super) unsafe fn allocate_memory(size: usize) -> std::io::Result<*mut core::ffi::c_void> {
    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE,
    };

    let ptr = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        )
    };

    if ptr.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(ptr)
}

#[cfg(unix)]
pub(super) unsafe fn allocate_memory(size: usize) -> std::io::Result<*mut core::ffi::c_void> {
    use libc::{mmap, sysconf, MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};

    const PAGE_SIZE: usize = crate::mem::PAGE_SIZE as usize;
    let host_page_size = unsafe { sysconf(libc::_SC_PAGESIZE) as usize };

    assert!(
        host_page_size >= PAGE_SIZE,
        "Hosts with smaller than 4KiB pages are not supported."
    );

    let ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        return Err(std::io::Error::last_os_error());
    }
    Ok(ptr)
}

/// Cross-platform memory free using the host's system calls.
///
/// # Safety
/// - The address and size should match parameters and result of the
///   [`allocate_memory`] call.
#[cfg(windows)]
pub(super) unsafe fn free_memory(
    address: *mut core::ffi::c_void,
    _size: usize,
) -> std::io::Result<()> {
    use windows_sys::Win32::System::Memory::{VirtualFree, MEM_RELEASE};

    let res = unsafe { VirtualFree(address, 0, MEM_RELEASE) };

    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}

#[cfg(unix)]
pub(super) unsafe fn free_memory(
    address: *mut core::ffi::c_void,
    size: usize,
) -> std::io::Result<()> {
    use libc::munmap;

    let res = unsafe { munmap(address, size) };

    if res == -1 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}
