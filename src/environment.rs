/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The core of the emulator: management of state, execution, threading.
//!
//! Unlike its siblings, this module should be considered private and only used
//! via the re-exports one level up.

mod airplay_sdk;
pub mod app_picker;
mod mutex;
mod nullable_box;

use crate::abi::{CallFromHost, GuestFunction};
use crate::audio::openal::OpenALManager;
use crate::cpu::Cpu;
use crate::libc::semaphore::sem_t;
use crate::mem::{GuestUSize, MutPtr, MutVoidPtr};
use crate::{
    abi, bundle, cpu, dyld, frameworks, fs, gdb, image, libc, mach_o, mem, objc, options, stack,
    window,
};
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::TcpListener;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime};

use crate::libc::pthread::cond::pthread_cond_t;
use crate::libc::stdio::FILE;
use crate::window::DeviceFamily;
use corosensei::{Coroutine, Yielder};
pub use mutex::{MutexId, MutexType, PTHREAD_MUTEX_DEFAULT};
use nullable_box::NullableBox;

/// SDL audio subsystem for the mixer: shares the window's context if one
/// exists, else creates a standalone one. `None` if SDL audio is unavailable.
fn build_audio_subsystem(window: Option<&window::Window>) -> Option<sdl2::AudioSubsystem> {
    match window {
        Some(w) => w.create_audio_subsystem().ok(),
        None => sdl2::init().ok().and_then(|sdl| sdl.audio().ok()),
    }
}

/// Index into the [Vec] of threads. Thread 0 is always the main thread.
pub type ThreadId = usize;

pub type HostContext = Coroutine<Environment, Environment, Environment>;

/// Bookkeeping for a thread.
pub struct Thread {
    /// The current running state of the thread. See [ThreadState].
    pub state: ThreadState,
    /// If this is not [ThreadBlock::NotBlocked], the thread is not executing
    /// until a certain condition is fufilled.
    pub blocked_by: ThreadBlock,
    /// Container for thread local state of various child modules
    pub framework_state: frameworks::ThreadLocalState,
    /// After a secondary thread finishes, this is set to the returned value.
    return_value: Option<MutVoidPtr>,
    /// Context object containing the CPU state for this thread.
    ///
    /// There should always be `(threads.len() - 1)` contexts in existence.
    /// When a thread is currently executing, its state is stored directly in
    /// the CPU, rather than in a context object. In that case, this field is
    /// None. See also: [std::mem::take] and [cpu::Cpu::swap_context].
    pub guest_context: Option<Box<cpu::CpuContext>>,
    /// The coroutine associated with this thread.
    ///
    /// In more typical rust, this is equivalent to to a [std::future::Future].
    /// Like a [std::future::Future], it holds the call stack so the inner
    /// function can (cooperatively) suspend execution and be resumed at a
    /// later time. Unlike a [std::future::Future], the call stack is actually
    /// stored as a stack, and not as an anonymous, compiler generated,
    /// (typically heap allocated) object.
    host_context: Option<HostContext>,
    /// Address range of this thread's stack, used to check if addresses are in
    /// range while producing a stack trace.
    pub stack: Option<std::ops::RangeInclusive<u32>>,
}

fn decode_arm_direct_global_getter(target: u32, bytes: &[u8]) -> Option<u32> {
    let first = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?);
    let second = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?);
    let third = u32::from_le_bytes(bytes.get(8..12)?.try_into().ok()?);
    if first & 0xffff_f000 != 0xe59f_0000 || second != 0xe590_0000 || third != 0xe12f_ff1e {
        return None;
    }

    let literal_address = target.checked_add(8)?.checked_add(first & 0xfff)?;
    let literal_offset: usize = literal_address.checked_sub(target)?.try_into().ok()?;
    let literal = bytes.get(literal_offset..literal_offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(literal.try_into().ok()?))
}

fn decode_arm_pc_relative_literal_load(
    instruction_address: u32,
    instruction: u32,
) -> Option<(u8, u32)> {
    // ARM A32 LDR Rt, [PC, #+/-imm12], with pre-indexing and no writeback.
    // Ignore the condition field and the add/subtract bit while checking the
    // fixed opcode fields.
    if instruction & 0x0f7f_0000 != 0x051f_0000 {
        return None;
    }
    let base = instruction_address.checked_add(8)?;
    let offset = instruction & 0x0fff;
    let literal_address = if instruction & 0x0080_0000 != 0 {
        base.checked_add(offset)?
    } else {
        base.checked_sub(offset)?
    };
    Some((((instruction >> 12) & 0xf) as u8, literal_address))
}

impl Thread {
    fn is_blocked(&self) -> bool {
        !matches!(self.blocked_by, ThreadBlock::NotBlocked)
    }
    pub fn is_alive(&self) -> bool {
        !matches!(self.state, ThreadState::Dead)
    }
    pub fn is_running(&self) -> bool {
        matches!(self.state, ThreadState::Running | ThreadState::Stepping)
    }
}

impl std::fmt::Debug for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Thread {{ state: {:?}, blocked_by: {:?}, return_value: {:?} }}",
            self.state, self.blocked_by, self.return_value
        )
    }
}

/// The struct containing the entire emulator state. Methods are provided for
/// execution and management of threads.
pub struct Environment {
    /// Reference point for various timing functions.
    pub startup_time: Instant,
    pub diagnostics: Option<Box<crate::diagnostics::Recorder>>,
    pub bundle: NullableBox<bundle::Bundle>,
    pub fs: NullableBox<fs::Fs>,
    /// The window is only absent when running in headless mode.
    pub window: Option<Box<window::Window>>,
    pub openal_manager: NullableBox<OpenALManager>,
    pub mem: NullableBox<mem::Mem>,
    /// Loaded binaries. Index `0` is always the app binary, other entries are
    /// dynamic libraries.
    pub bins: Vec<mach_o::MachO>,
    pub objc: NullableBox<objc::ObjC>,
    pub dyld: NullableBox<dyld::Dyld>,
    pub cpu: NullableBox<cpu::Cpu>,
    pub current_thread: ThreadId,
    pub threads: Vec<Thread>,
    pub libc_state: NullableBox<libc::State>,
    pub framework_state: NullableBox<frameworks::State>,
    pub mutex_state: NullableBox<mutex::MutexState>,
    pub options: NullableBox<options::Options>,
    gdb_server: Option<Box<gdb::GdbServer>>,
    pub env_vars: HashMap<Vec<u8>, MutPtr<u8>>,
    /// Set to [true] when created using [Environment::new_without_app].
    pub is_app_picker: bool,
    /// Set to [true] to signal a return to the app picker.
    pub return_to_app_picker: bool,
    /// When an app being quit is given up on, like iOS's watchdog does.
    pub quit_deadline: Option<Instant>,
    pub dump_file: Option<std::fs::File>,
    yielder: *const Yielder<Environment, Environment>,
    // The amount of ticks to run for Some(value), or single-stepping for None.
    // Sadly, setting ticks to 1 does not step properly, so Option is required.
    remaining_ticks: Option<u64>,
    panic_cell: Rc<Cell<Option<Environment>>>,
}

/// What to do next when executing this thread.
enum ThreadNextAction {
    /// Continue CPU emulation.
    Continue,
    /// Return to host.
    ReturnToHost,
    /// Debug the current CPU error.
    DebugCpuError(cpu::CpuError),
}

/// If/what a thread is blocked by.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadBlock {
    // Default state. (thread is not blocked)
    NotBlocked,
    // Thread is sleeping. (until Instant)
    Sleeping(Instant),
    // Thread is waiting for a mutex to unlock.
    Mutex(MutexId),
    // Thread is waiting on a semaphore.
    Semaphore(MutPtr<sem_t>),
    // Thread is waiting on a condition variable
    Condition(MutPtr<pthread_cond_t>, Option<Duration>),
    // Thread is waiting for another thread to finish (joining).
    Joining(ThreadId, MutPtr<MutVoidPtr>),
    // Thread has hit a cpu error, and is waiting to be debugged.
    WaitingForDebugger(Option<cpu::CpuError>),
    // Thread is suspended. We keep a suspend count and a previous thread state
    // (boxed to avoid cyclic dependency), which would be restored upon
    // resuming.
    Suspended(usize, Box<ThreadBlock>),
    // Thread is waiting on a FILE object lock.
    FileObjectLock(MutPtr<FILE>),
}

impl std::fmt::Display for ThreadBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadBlock::NotBlocked => write!(f, "Running"),
            ThreadBlock::Sleeping(wake_time) => {
                let remaining = wake_time.checked_duration_since(Instant::now());
                match remaining {
                    Some(dur) => write!(f, "Sleeping for {:.3}s", dur.as_secs_f32()),
                    None => write!(f, "Waking"),
                }
            }
            ThreadBlock::Mutex(ptr) => write!(f, "Blocked on mutex {ptr:?}"),
            ThreadBlock::Semaphore(ptr) => write!(f, "Blocked on semaphore {ptr:?}"),
            ThreadBlock::Condition(ptr, _) => write!(f, "Blocked on condition {ptr:?}"),
            // tid adds 1 to match gdb's thread numbers
            ThreadBlock::Joining(tid, _) => write!(f, "Joining on thread {}", tid + 1),
            ThreadBlock::Suspended(count, old) => {
                write!(f, "Suspended (count {}, previously {})", count, old)
            }
            ThreadBlock::FileObjectLock(ptr) => write!(f, "Waiting for file to unlock {ptr:?}"),
            // Unlikely to be seen
            ThreadBlock::WaitingForDebugger(_) => write!(f, "Waiting for debugger"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadState {
    /// The thread is currently running (but may be blocked by a [ThreadBlock])
    Running,
    /// The thread is being stepped by the gdb server.
    Stepping,
    /// The thread is paused by the gdb server, typically because another
    /// thread is stepping and client has asked for other threads not to run.
    Paused,
    /// The thread has died.
    Dead,
}

struct BinaryDependencyNode {
    name: String,
    dependencies: Vec<String>,
}

/// Topologically sorts the binary dylibs using Kahn's algorithm
/// and returns the sorted list of indices
fn generate_binary_load_order(graph: &[BinaryDependencyNode]) -> Result<Vec<usize>, String> {
    let node_to_index: HashMap<_, _> = graph
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.name.as_str(), idx))
        .collect();

    let mut node_dependents = HashMap::new();
    let mut node_in_degrees: HashMap<_, _> = node_to_index.values().map(|&idx| (idx, 0)).collect();

    for node in graph {
        let &bin_index = node_to_index
            .get(node.name.as_str())
            .ok_or_else(|| format!("Failed to find {:?} name mapping", node.name))?;

        // Bin names dont include prefix while dynamic lib paths do
        for dependency in node
            .dependencies
            .iter()
            .map(|path| path.strip_prefix("/usr/lib/").unwrap_or(path.as_str()))
        {
            // Ignore dependencies that are not included in packaged dylibs
            let Some(&dylib_index) = node_to_index.get(dependency) else {
                continue;
            };
            node_dependents
                .entry(dylib_index)
                .or_insert_with(Vec::new)
                .push(bin_index);

            node_in_degrees
                .entry(bin_index)
                .and_modify(|in_degree| *in_degree += 1);
        }
    }

    let mut leaf_nodes: VecDeque<_> = node_in_degrees
        .iter()
        .filter(|(_, &in_degree)| in_degree == 0)
        .map(|(&node, _)| node)
        .collect();

    let mut sorted_indices = Vec::new();

    while let Some(node) = leaf_nodes.pop_front() {
        sorted_indices.push(node);

        let Some(dependents) = node_dependents.get(&node) else {
            continue;
        };

        for &dependant in dependents {
            let Some(in_degree) = node_in_degrees.get_mut(&dependant) else {
                continue;
            };
            *in_degree -= 1;

            if *in_degree == 0 {
                leaf_nodes.push_back(dependant);
            }
        }
    }

    if let Some((&index, _)) = node_in_degrees.iter().find(|(_, &in_degree)| in_degree > 0) {
        return Err(format!(
            "Failed to sort nodes, cycle with {:?}",
            graph.get(index).unwrap().name
        ));
    }
    log!(
        "Found sorted order {:?}",
        sorted_indices
            .iter()
            .map(|&index| graph.get(index).unwrap().name.as_str())
            .collect::<Vec<_>>()
    );

    Ok(sorted_indices)
}

/// Enforces the one (real) Environment limit. See
/// [Environment::with_yielder] for why this is needed.
static ENVIRONMENT_INSTANCE_EXISTS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

impl Environment {
    /// Loads the binary and sets up the emulator.
    pub fn new(
        bundle: bundle::Bundle,
        fs: fs::Fs,
        mut options: options::Options,
        app_args: Vec<String>,
    ) -> Result<Environment, String> {
        let startup_time = Instant::now();

        // Enforces the one (real) Environment limit. See `with_yielder` for
        // why this is needed.
        if ENVIRONMENT_INSTANCE_EXISTS.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err("Only one (real) Environment can exist at a time!".to_string());
        }
        // If this fails, there's no Environment whose drop would clear it, and
        // the next app could never start.
        struct ClearOnFailure(bool);
        impl Drop for ClearOnFailure {
            fn drop(&mut self) {
                if self.0 {
                    ENVIRONMENT_INSTANCE_EXISTS.store(false, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }
        let mut clear_on_failure = ClearOnFailure(true);

        // Certain apps need to launch in a non-portrait orientation, and this
        // should be handled before creating the window because handling of
        // window rotation after-the-fact is somewhat glitchy.
        // This also ensures the splash screen is correctly oriented.
        if options.initial_orientation == window::DeviceOrientation::Portrait {
            let orientations = bundle.supported_interface_orientations();
            // Many apps list other orientations as well as portrait, but
            // should still start in portrait.
            if !orientations.contains(&"UIInterfaceOrientationPortrait") {
                if let Some(&non_portrait_orientation) = orientations
                    .iter()
                    .find(|&&o| o != "UIInterfaceOrientationPortrait")
                {
                    // TODO: Overwriting the options might not be ideal;
                    //       do we need to distinguish this kind of
                    //       orientation change from others?
                    options.initial_orientation = match non_portrait_orientation {
                        // UIInterfaceOrientation values are flipped relative to
                        // (UI)DeviceOrientation values (content has to rotate
                        // in the opposite direction to how the device rotates).
                        "UIInterfaceOrientationPortraitUpsideDown" => {
                            window::DeviceOrientation::PortraitUpsideDown
                        }
                        "UIInterfaceOrientationLandscapeLeft" => {
                            window::DeviceOrientation::LandscapeRight
                        }
                        "UIInterfaceOrientationLandscapeRight" => {
                            window::DeviceOrientation::LandscapeLeft
                        }
                        // This appears to be an older way set the orientation.
                        // From testing, it seems to correspond to left.
                        "UIInterfaceOrientationLandscape" => {
                            window::DeviceOrientation::LandscapeLeft
                        }
                        other => unimplemented!("Unsupported startup orientation: {:?}", other),
                    };
                    log!("App needs non-portrait user interface orientation {:?}, applying device orientation {:?}.", non_portrait_orientation, options.initial_orientation);
                }
            }
        }

        let device_family_override = options.device_family;
        let device_family_array = bundle.device_family_array();
        let device_family = match device_family_array.len() {
            // iPhone only or iPad only
            1 => {
                let only_supported = device_family_array[0];
                if let Some(dfo) = device_family_override {
                    if dfo != only_supported {
                        log!("Warning: User-defined {:?} device family override is not supported by the app! ignoring", dfo);
                    }
                }
                only_supported
            }
            // iPhone and iPad
            2 => {
                if let Some(dfo) = device_family_override {
                    assert!(device_family_array.contains(&dfo));
                    dfo
                } else if window::host_is_ipad() {
                    // The iPad version makes better use of the screen.
                    DeviceFamily::iPad
                } else {
                    assert!(device_family_array.contains(&DeviceFamily::iPhone));
                    DeviceFamily::iPhone
                }
            }
            _ => unreachable!(),
        };
        log!("{:?} device family is chosen.", device_family);
        options.device_family = Some(device_family);

        let window = if options.headless {
            None
        } else {
            let icon = bundle.load_icon(&fs);
            if let Err(ref e) = icon {
                log!("Warning: {}", e);
            }

            let launch_image_path = bundle.launch_image_path();
            let launch_image = if fs.is_file(&launch_image_path) {
                let res = fs
                    .read(launch_image_path)
                    .map_err(|_| "Could not read launch image file".to_string())
                    .and_then(|bytes| {
                        image::Image::from_bytes(&bytes)
                            .map_err(|e| format!("Could not parse launch image: {e}"))
                    });
                if let Err(ref e) = res {
                    log!("Warning: {}", e);
                };
                res.ok()
            } else {
                None
            };

            Some(Box::new(window::Window::new(
                &format!(
                    "{} (iDared 32bit {}{}{})",
                    bundle.display_name(),
                    super::branding(),
                    if super::branding().is_empty() {
                        ""
                    } else {
                        " "
                    },
                    super::DISPLAY_VERSION
                ),
                icon.ok(),
                launch_image,
                &options,
            )))
        };

        let mut mem = mem::Mem::new();

        // Games that read memory after freeing it, which works on a device
        // because freeing doesn't clear it. (Critter Crunch crashes when a
        // critter explodes; Hairball when a level starts.)
        const KEEP_FREED_MEMORY: &[&str] = &[
            "com.ea.spore",
            "com.capybaragames.CritterCrunch",
            "com.go.starwave.CritterCrunch",
            "com.yourcompany.Hairball",
        ];
        let bundle_id = bundle.bundle_identifier();
        let keep_freed_memory = KEEP_FREED_MEMORY.iter().any(|id| bundle_id.starts_with(id));
        mem.zero_memory_on_free = !keep_freed_memory;
        if keep_freed_memory {
            log!("Applying game-specific hack: zeroing memory on alloc instead of free.");
        }
        let executable = mach_o::MachO::load_from_file(
            bundle.executable_path(),
            &fs,
            &mut mem,
            /* slide: */ 0,
        )
        .map_err(|e| format!("Could not load executable: {e}"))?;

        // This needs the executable's code as it was before linking.
        airplay_sdk::fix_code_hash(&executable, &mut mem);

        let mut dylibs = Vec::new();
        for dylib in &executable.dynamic_libraries {
            // There are some Free Software libraries bundled with touchHLE and
            // exposed via the guest file system (see Fs::new()).
            let dylib_path = fs::GuestPath::new(dylib);
            if fs.is_file(dylib_path) {
                // We use hardcoded slide values for libgcc and libstdc++
                // based on base addresses of those dylibs prior to iOS 3.1
                // TODO: implement some kind of ASLR instead of hardcoding
                assert!(dylib_path.as_str().starts_with("/usr/lib/"));
                let name = dylib_path.file_name().unwrap();
                let dylib_slide = match name {
                    "libstdc++.6.dylib" | "libstdc++.6.0.9.dylib" => 0x3748a000,
                    "libgcc_s.1.dylib" => 0x30000000,
                    "libz.1.dylib" | "libz.1.2.3.dylib" | "libz.dylib" | "libz.1.1.3.dylib" => {
                        // We build `libz` from sources with our OSS toolchain,
                        // the base address is already set and sliding is not
                        // needed.
                        0
                    }
                    "libsqlite3.dylib" | "libsqlite3.0.dylib" => {
                        // We build `libsqlite3` from sources with our OSS
                        // toolchain, the base address is already set and
                        // sliding is not needed.
                        0
                    }
                    "libxml2.2.dylib" | "libxml2.dylib" | "libxml2.2.7.8.dylib" => {
                        // We build `libxml2` from sources with our OSS
                        // toolchain, the base address is already set and
                        // sliding is not needed.
                        0
                    }
                    _ => unimplemented!("Unknown binary slide for {}", name),
                };
                let dylib = mach_o::MachO::load_from_file(
                    fs::GuestPath::new(dylib),
                    &fs,
                    &mut mem,
                    dylib_slide,
                )
                .map_err(|e| format!("Could not load bundled dylib: {e}"))?;
                dylibs.push(dylib);
            // Otherwise, look for it in our host implementations.
            } else if !crate::dyld::DYLIB_LIST
                .iter()
                .any(|d| d.path == dylib || d.aliases.contains(&dylib.as_str()))
            {
                log!(
                    "Warning: app binary depends on unimplemented or missing dylib \"{}\"",
                    dylib
                );
            }
        }

        let entry_point_addr = executable
            .entry_point_pc
            .ok_or_else(|| {
                "Mach-O file does not specify an entry point PC, perhaps it is not an executable?"
                    .to_string()
            })
            .unwrap();
        let entry_point_addr = abi::GuestFunction::from_addr_with_thumb_bit(entry_point_addr);

        log_dbg!("Address of start function: {:?}", entry_point_addr);

        let mut bins = dylibs;
        bins.insert(0, executable);

        let mut objc = objc::ObjC::new();

        let mut diagnostics = options.diagnostics.then(crate::diagnostics::Recorder::new);
        let mut dyld = dyld::Dyld::new();
        dyld.do_initial_linking(
            &bins,
            &mut mem,
            &mut objc,
            diagnostics.as_deref_mut(),
        )?;

        let cpu = cpu::Cpu::new(match options.direct_memory_access {
            true => Some(&mut mem),
            false => None,
        });

        let main_thread_init_routine = Coroutine::new(move |yielder, mut env: Environment| {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                env.with_yielder(yielder, move |env| {
                    echo!("CPU emulation begins now.");
                    // Some apps use the stack inside the static initializer.
                    // While properly behaving apps should be fine, some app
                    // will try to poke the top of the stack, so we'll give
                    // it some room.
                    env.cpu.regs_mut()[Cpu::SP] = 0xFFFFF000;

                    // Call `+load` method on classes where it's defined.
                    // TODO: `+load` methods from our image should take priority
                    // over frameworks ones.
                    // TODO: a category `+load` method should be called after
                    // the class's own +load method.
                    // Note: `+load` is sent without triggering `+initialize`,
                    // matching the runtime's guarantee that `+load` runs first.
                    let mut to_be_loaded = Vec::new();
                    let mut processed = HashSet::new();
                    let load_sel: objc::SEL = env
                        .objc
                        .register_host_selector("load".to_string(), &mut env.mem);
                    for (class_name, &class) in env.objc.all_classes() {
                        if processed.contains(&class) {
                            continue;
                        }
                        if env.objc.is_unimplemented_class(class) || env.objc.is_fake_class(class) {
                            continue;
                        }
                        if env
                            .objc
                            .object_has_uninherited_method(&env.mem, class, load_sel)
                        {
                            log_dbg!("Calling +load on inheritance chain of {} class", class_name);
                            let mut inherited = Vec::new();
                            let mut curr_class = class;
                            while curr_class != objc::nil
                                && !env.objc.is_unimplemented_class(curr_class)
                                && !env.objc.is_fake_class(curr_class)
                            {
                                if !processed.contains(&curr_class)
                                    && env.objc.object_has_uninherited_method(
                                        &env.mem, curr_class, load_sel,
                                    )
                                {
                                    inherited.push(curr_class);
                                    processed.insert(curr_class);
                                }
                                curr_class = env.objc.get_superclass(curr_class);
                            }
                            to_be_loaded.extend(inherited.into_iter().rev());
                        }
                    }
                    for &class in &to_be_loaded {
                        () = objc::msg_send_no_initialize(env, (class, load_sel));
                    }

                    // Static initializers for libraries must be run before
                    // the initializer in the app binary.
                    for bin_idx in env.get_sorted_bin_indices().unwrap() {
                        let Some(bin) = env.bins.get(bin_idx) else {
                            continue;
                        };
                        let Some(section) =
                            bin.get_section(mach_o::SectionType::ModInitFuncPointers)
                        else {
                            continue;
                        };

                        log_dbg!("Calling static initializers for {:?}", bin.name);
                        assert!(section.size % 4 == 0);
                        let base: mem::ConstPtr<abi::GuestFunction> =
                            mem::Ptr::from_bits(section.addr);
                        let count = section.size / 4;
                        for i in 0..count {
                            let func = env.mem.read(base + i);
                            log_dbg!(
                                "Calling static initializer at {:?} from {:?}",
                                func,
                                (base + i)
                            );
                            () = func.call_from_host(env, ());
                        }
                        log_dbg!("Static initialization done");
                    }

                    {
                        let bin_path = env.bundle.executable_path();

                        let envp_list: Vec<String> = env
                            .env_vars
                            .clone()
                            .iter_mut()
                            .map(|tuple| {
                                [
                                    std::str::from_utf8(tuple.0).unwrap(),
                                    "=",
                                    env.mem.cstr_at_utf8(*tuple.1).unwrap(),
                                ]
                                .concat()
                            })
                            .collect();
                        let envp_ref_list: Vec<&str> =
                            envp_list.iter().map(|keyvalue| keyvalue.as_str()).collect();

                        let bin_path_apple_key = format!("executable_path={}", bin_path.as_str());

                        let argv = Vec::from_iter(
                            std::iter::once(bin_path.as_str())
                                .chain(app_args.iter().map(|s| s.as_str())),
                        );
                        let envp = envp_ref_list.as_slice();
                        let apple = &[bin_path_apple_key.as_str()];
                        stack::prep_stack_for_start(&mut env.mem, &mut env.cpu, &argv, envp, apple);
                    }

                    // Manually call here, since running call_from_host pushes
                    // a stack frame and disrupts abi for _start.
                    env.cpu
                        .branch_with_link(entry_point_addr, env.dyld.thread_exit_routine());
                    env.run_call();

                    if !env.return_to_app_picker {
                        panic!("Main function exited unexpectedly!");
                    }
                })
            }));
            if let Err(e) = res {
                let panic_cell = env.panic_cell.clone();
                panic_cell.set(Some(env));
                std::panic::resume_unwind(e);
            }
            env
        });
        let main_thread = Thread {
            state: ThreadState::Running,
            blocked_by: ThreadBlock::NotBlocked,
            return_value: None,
            guest_context: None,
            host_context: Some(main_thread_init_routine),
            stack: Some(mem::Mem::MAIN_THREAD_STACK_LOW_END..=0u32.wrapping_sub(1)),
            framework_state: Default::default(),
        };

        let audio_subsystem = build_audio_subsystem(window.as_deref());

        let mut env = Environment {
            startup_time,
            diagnostics,
            bundle: NullableBox::new(bundle),
            fs: NullableBox::new(fs),
            window,
            openal_manager: NullableBox::new(OpenALManager::new(audio_subsystem)?),
            mem: NullableBox::new(mem),
            bins,
            objc: NullableBox::new(objc),
            dyld: NullableBox::new(dyld),
            cpu: NullableBox::new(cpu),
            current_thread: 0,
            threads: vec![main_thread],
            libc_state: Default::default(),
            mutex_state: Default::default(),
            framework_state: Default::default(),
            options: NullableBox::new(options),
            gdb_server: None,
            env_vars: Default::default(),
            is_app_picker: false,
            return_to_app_picker: false,
            quit_deadline: None,
            dump_file: None,
            yielder: std::ptr::null(),
            remaining_ticks: None,
            panic_cell: Rc::new(Cell::new(None)),
        };

        // Apps get the on-screen Home button (where there is one), the app
        // picker doesn't.
        if let Some(window) = env.window.as_mut() {
            window.enable_on_screen_home_button();
        }

        if env.options.dumping_options.any() {
            env.dump_file =
                Some(std::fs::File::create(&env.options.dumping_file).map_err(|e| e.to_string())?);
        }

        env.set_up_initial_env_vars();
        dyld::Dyld::do_late_linking(&mut env);

        env.cpu.set_cpsr(cpu::Cpu::CPSR_USER_MODE);

        if let Some(addrs) = env.options.gdb_listen_addrs.take() {
            let listener = TcpListener::bind(addrs.as_slice())
                .map_err(|e| format!("Could not bind to {addrs:?}: {e}"))?;
            echo!(
                "Waiting for debugger connection on {}...",
                addrs
                    .into_iter()
                    .map(|a| format!("{a}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            let (client, client_addr) = listener
                .accept()
                .map_err(|e| format!("Could not accept connection: {e}"))?;
            echo!("Debugger client connected on {}.", client_addr);
            env.gdb_server = Some(Box::new(gdb::GdbServer::new(client)));
        }

        if env.options.dumping_options.linking_info {
            let file = env.dump_file.as_mut().unwrap();
            env.objc.dump_classes(file).unwrap();
            env.dyld.dump_lazy_symbols(&env.bins, file).unwrap();
            env.objc
                .dump_selectors(&env.bins[0], &env.mem, file)
                .unwrap();
        }

        env.cpu.branch(entry_point_addr);
        clear_on_failure.0 = false;
        Ok(env)
    }

    /// Set up the emulator environment without loading an app binary.
    ///
    /// This is a special mode that only exists to support the app picker, which
    /// uses the emulated environment to draw its UI and process input. Filling
    /// some of the fields with fake data is a hack, but it means the frameworks
    /// do not need to be aware of the app picker's peculiarities, so it is
    /// cleaner than the alternative!
    pub fn new_without_app(
        options: options::Options,
        icon: image::Image,
    ) -> Result<Environment, String> {
        // Enforces a one (real) Environment limit. See `with_yielder` for
        // why this is needed.
        if ENVIRONMENT_INSTANCE_EXISTS.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("Only one (real) Environment can exist at a time!".to_string());
        }
        ENVIRONMENT_INSTANCE_EXISTS.store(true, std::sync::atomic::Ordering::Relaxed);
        let bundle = bundle::Bundle::new_fake_bundle();
        let fs = fs::Fs::new_fake_fs();

        let startup_time = Instant::now();

        let launch_image = None;

        assert!(!options.headless);
        let mut window = Some(Box::new(window::Window::new(
            &format!(
                "iDared 32bit {}{}{}",
                super::branding(),
                if super::branding().is_empty() {
                    ""
                } else {
                    " "
                },
                super::DISPLAY_VERSION
            ),
            Some(icon),
            launch_image,
            &options,
        )));
        // Only the app picker fills a taller screen, games keep 320×480. It's
        // also drawn at the screen's real resolution.
        window.as_mut().unwrap().fit_screen_to_device();
        window.as_mut().unwrap().enable_hidpi_ui();

        let mut mem = mem::Mem::new();

        let bins = Vec::new();

        let mut objc = objc::ObjC::new();

        let mut dyld = dyld::Dyld::new();
        dyld.do_initial_linking_with_no_bins(&mut mem, &mut objc);

        let cpu = cpu::Cpu::new(match options.direct_memory_access {
            true => Some(&mut mem),
            false => None,
        });

        let main_thread = Thread {
            state: ThreadState::Running,
            blocked_by: ThreadBlock::NotBlocked,
            return_value: None,
            guest_context: None,
            host_context: None,
            stack: Some(mem::Mem::MAIN_THREAD_STACK_LOW_END..=0u32.wrapping_sub(1)),
            framework_state: Default::default(),
        };

        let audio_subsystem = build_audio_subsystem(window.as_deref());

        let mut env = Environment {
            startup_time,
            diagnostics: None,
            bundle: NullableBox::new(bundle),
            fs: NullableBox::new(fs),
            window,
            openal_manager: NullableBox::new(OpenALManager::new(audio_subsystem)?),
            mem: NullableBox::new(mem),
            bins,
            objc: NullableBox::new(objc),
            dyld: NullableBox::new(dyld),
            cpu: NullableBox::new(cpu),
            current_thread: 0,
            threads: vec![main_thread],
            libc_state: Default::default(),
            mutex_state: Default::default(),
            framework_state: Default::default(),
            options: NullableBox::new(options),
            gdb_server: None,
            env_vars: Default::default(),
            is_app_picker: true,
            return_to_app_picker: false,
            quit_deadline: None,
            dump_file: None,
            yielder: std::ptr::null(),
            remaining_ticks: None,
            panic_cell: Rc::new(Cell::new(None)),
        };

        env.set_up_initial_env_vars();

        // Dyld::do_late_linking() would be called here, but it doesn't do
        // anything relevant here, so it's skipped.

        {
            let argv = &[];
            let envp = &[];
            let apple = &[];
            stack::prep_stack_for_start(&mut env.mem, &mut env.cpu, argv, envp, apple);
        }

        env.cpu.set_cpsr(cpu::Cpu::CPSR_USER_MODE);

        // GDB server setup would be done here, but there's no need for it.

        // "CPU emulation begins now" would happen here, but there's nothing
        // to emulate. :)

        Ok(env)
    }

    /// Create a new Environment to swap with.
    ///
    /// SAFETY: You must *NEVER, IN ANY CIRCUMSTANCE* dereference any fields or
    /// call any methods on the environment. This means that you must *NEVER,
    /// IN ANY CIRCUMSTANCE* leak this to safe code. You *MUST* make sure this
    /// includes panic safety - do not allow a panic to accidentally smuggle
    /// out this environment to safe code!
    ///
    /// Admittedly, even if this is leaked, it's very unlikely it would lead to
    /// any real problems, just a null pointer deref.
    unsafe fn new_fake() -> Self {
        Self {
            startup_time: Instant::now(),
            diagnostics: None,
            bundle: NullableBox::null(),
            fs: NullableBox::null(),
            window: None,
            openal_manager: NullableBox::null(),
            mem: NullableBox::null(),
            bins: Vec::new(),
            objc: NullableBox::null(),
            dyld: NullableBox::null(),
            cpu: NullableBox::null(),
            current_thread: 0,
            threads: Vec::new(),
            libc_state: NullableBox::null(),
            framework_state: NullableBox::null(),
            mutex_state: NullableBox::null(),
            options: NullableBox::null(),
            gdb_server: None,
            env_vars: HashMap::new(),
            is_app_picker: true,
            return_to_app_picker: false,
            quit_deadline: None,
            dump_file: None,
            yielder: std::ptr::null(),
            remaining_ticks: None,
            panic_cell: Rc::new(Cell::new(None)),
        }
    }

    /// Add a [corosensei::Yielder] so that it can be used by the passed
    /// Environment in the passed function. This exists to avoid
    /// reannotating all code with an additional lifetime on every use of
    /// Environment.
    ///
    /// The design _would_ be unsound if it wasn't for the one real
    /// Environment limit.
    ///
    /// Theoretically (even if this is extremely unlikely), this could
    /// happen:
    ///      - New thread(coroutine) is created and calls [Self::with_yielder]
    ///      - Coroutine swaps the provided env with another env.
    ///      - Coroutine moves the env to the executor.
    ///      - Coroutine ends (and ends the yielder).
    ///      - Executor uses yielder - oh no, UAF!
    ///  (While this is pretty theoretical, prudent readers will in fact
    ///  notice this is the exact same way we share Environments across
    ///  threads anyways! - just with [Self::new_fake] instead of a "real"
    ///  Environment.)
    ///
    ///  There is however, (seemingly) no way to (safely) move out behind a
    ///  &mut T without another T to replace it - so it is safe as long there
    ///  is only ever one Environment exposed to safe code (this is part of
    ///  the [Self::new_fake] safety requirements).
    pub fn with_yielder<F, T>(&mut self, yielder: &Yielder<Environment, Environment>, block: F) -> T
    where
        F: FnOnce(&mut Environment) -> T + 'static,
        T: 'static,
    {
        assert!(self.yielder.is_null());
        self.yielder = yielder;
        // We need to ensure panic safety here, so make sure to reset the
        // yielder if the inner function panics.
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| block(self)));
        self.yielder = std::ptr::null();
        match res {
            Ok(ret) => ret,
            Err(e) => {
                std::panic::resume_unwind(e);
            }
        }
    }

    /// Get a shared reference to the window. Panics if touchHLE is running in
    /// headless mode.
    pub fn window(&self) -> &window::Window {
        self.window.as_ref().expect(
            "Tried to do something that needs a window, but touchHLE is running in headless mode!",
        )
    }

    /// Get a mutable reference to the window. Panics if touchHLE is running
    /// in headless mode.
    pub fn window_mut(&mut self) -> &mut window::Window {
        self.window.as_mut().expect(
            "Tried to do something that needs a window, but touchHLE is running in headless mode!",
        )
    }

    pub fn stack_for_longjmp(&self, mut lr: u32, fp: u32) -> Vec<u32> {
        let stack_range = self.threads[self.current_thread].stack.clone().unwrap();
        let mut frames = Vec::new();
        let mut fp: mem::ConstPtr<u8> = mem::Ptr::from_bits(fp);
        let return_to_host_routine_addr = self.dyld.return_to_host_routine().addr_with_thumb_bit();
        while stack_range.contains(&fp.to_bits()) && lr != return_to_host_routine_addr {
            frames.push(lr);
            lr = self.mem.read((fp + 4).cast());
            fp = self.mem.read(fp.cast());
        }
        frames
    }

    fn dump_all_regs(&self) {
        echo_no_panic!(
            "Dumping registers for current thread (#{})",
            self.current_thread
        );
        self.cpu.dump_regs();
        for (tid, thread) in self.threads.iter().enumerate() {
            if thread.is_alive() && tid != self.current_thread {
                echo_no_panic!(
                    "Dumping registers for thread #{} (blocked by {:?})",
                    tid,
                    thread.blocked_by
                );
                let Some(ctx) = thread.guest_context.as_ref() else {
                    echo_no_panic!("Could not get registers for thread {}!", tid);
                    return;
                };
                cpu::Cpu::echo_regs(&ctx.regs);
            }
        }
    }

    fn stack_trace_current(&self) {
        if self.current_thread == 0 {
            echo_no_panic!("Attempting to produce stack trace for main thread:");
        } else {
            echo_no_panic!(
                "Attempting to produce stack trace for thread {}:",
                self.current_thread
            );
        }
        self.stack_trace_for_thread(self.current_thread);
    }

    fn stack_trace_all(&self) {
        echo_no_panic!(
            "Attempting to produce stack trace for current thread (#{}):",
            self.current_thread
        );
        self.stack_trace_for_thread(self.current_thread);
        for tid in 0..self.threads.len() {
            if self.threads[tid].is_alive() && tid != self.current_thread {
                echo_no_panic!("Attempting to produce stack trace for thread #{}:", tid);
                self.stack_trace_for_thread(tid);
            }
        }
    }

    fn stack_trace_for_thread(&self, tid: usize) {
        if tid >= self.threads.len() {
            echo_no_panic!(
                "Thread {} is too large ({} threads exist)!",
                tid,
                self.threads.len()
            );
        }
        let Some(stack_range) = self.threads[tid].stack.clone() else {
            echo_no_panic!("Failed to get stack trace!");
            return;
        };
        let (regs, cpsr) = if self.current_thread == tid {
            // Current thread is not stored in context since it is used by cpu,
            // get it from cpu.
            (*self.cpu.regs(), self.cpu.cpsr())
        } else {
            let Some(ctx) = self.threads[tid].guest_context.as_ref() else {
                echo_no_panic!("Failed to get registers for thread {}!", tid);
                return;
            };
            (ctx.regs, ctx.cpsr)
        };
        let pc_nothumb = regs[cpu::Cpu::PC];
        let thumb = (cpsr & cpu::Cpu::CPSR_THUMB) == cpu::Cpu::CPSR_THUMB;
        let pc = GuestFunction::from_addr_and_thumb_flag(pc_nothumb, thumb);
        echo_no_panic!(" 0. {:#x} (PC)", pc.addr_with_thumb_bit());
        let mut lr = regs[cpu::Cpu::LR];
        let return_to_host_routine_addr = self.dyld.return_to_host_routine().addr_with_thumb_bit();
        let thread_exit_routine_addr = self.dyld.thread_exit_routine().addr_with_thumb_bit();
        if lr == return_to_host_routine_addr {
            echo_no_panic!(" 1. [host function] (LR)");
        } else if lr == thread_exit_routine_addr {
            echo_no_panic!(" 1. [thread exit] (LR)");
            return;
        } else {
            echo_no_panic!(" 1. {:#x} (LR)", lr);
        }
        let mut i = 2;
        let mut fp: mem::ConstPtr<u8> = mem::Ptr::from_bits(regs[abi::FRAME_POINTER]);
        loop {
            if !stack_range.contains(&fp.to_bits()) {
                echo_no_panic!("Next FP ({:?}) is outside the stack.", fp);
                break;
            }
            lr = self.mem.read((fp + 4).cast());
            fp = self.mem.read(fp.cast());
            if lr == return_to_host_routine_addr {
                echo_no_panic!("{:2}. [host function]", i);
            } else if lr == thread_exit_routine_addr {
                echo_no_panic!("{:2}. [thread exit]", i);
                return;
            } else {
                echo_no_panic!("{:2}. {:#x}", i, lr);
            }
            i += 1;
        }
    }

    /// Create a new thread and return its ID. The `start_routine` and
    /// `user_data` arguments have the same meaning as the last two arguments to
    /// `pthread_create`.
    pub fn new_thread(
        &mut self,
        start_routine: abi::GuestFunction,
        user_data: mem::MutVoidPtr,
        stack_size: GuestUSize,
    ) -> ThreadId {
        let stack_alloc = self.mem.alloc(stack_size);
        let stack_high_addr = stack_alloc.to_bits() + stack_size;
        assert!(stack_high_addr.is_multiple_of(4));

        let thread_routine = Coroutine::new(move |yielder, mut env: Environment| {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                env.with_yielder(yielder, move |env| {
                    let regs = env.cpu.regs_mut();
                    regs[cpu::Cpu::LR] = env.dyld.thread_exit_routine().addr_with_thumb_bit();
                    regs[cpu::Cpu::SP] = stack_high_addr;
                    regs[0] = user_data.to_bits();

                    env.cpu.set_cpsr(
                        cpu::Cpu::CPSR_USER_MODE
                            | ((start_routine.is_thumb() as u32) * cpu::Cpu::CPSR_THUMB),
                    );
                    let return_value: mem::MutVoidPtr =
                        start_routine.call_from_host(env, (user_data,));
                    let curr_thread = &mut env.threads[env.current_thread];
                    curr_thread.return_value = Some(return_value);
                    curr_thread.state = ThreadState::Dead;
                });
            }));
            if let Err(e) = res {
                let panic_cell = env.panic_cell.clone();
                panic_cell.set(Some(env));
                std::panic::resume_unwind(e);
            }
            env
        });

        self.threads.push(Thread {
            state: ThreadState::Running,
            blocked_by: ThreadBlock::NotBlocked,
            return_value: None,
            guest_context: Some(Box::new(cpu::CpuContext::new())),
            host_context: Some(thread_routine),
            stack: Some(stack_alloc.to_bits()..=(stack_high_addr - 1)),
            framework_state: Default::default(),
        });

        let new_thread_id = self.threads.len() - 1;

        log_dbg!("Created new thread {} with stack {:#x}–{:#x}, will execute function {:?} with data {:?}", new_thread_id, stack_alloc.to_bits(), (stack_high_addr - 1), start_routine, user_data);

        new_thread_id
    }

    #[allow(unused)]
    pub fn get_tl_framework_state(&mut self) -> &mut frameworks::ThreadLocalState {
        &mut self.threads[self.current_thread].framework_state
    }

    /// Put the current thread to sleep for some duration, running other threads
    /// in the meantime as appropriate. Functions that call sleep right before
    /// they return back to the main run loop ([Environment::run]) should set
    /// `tail_call`.
    pub fn sleep(&mut self, duration: Duration) {
        log_dbg!(
            "Thread {} is going to sleep for {:?}.",
            self.current_thread,
            duration
        );
        let until = Instant::now().checked_add(duration).unwrap();
        self.yield_thread(ThreadBlock::Sleeping(until));
    }

    pub fn suspend_thread(&mut self, thread: ThreadId) {
        match &mut self.threads[thread].blocked_by {
            ThreadBlock::Suspended(count, _) => {
                *count += 1;
            }
            _ => {
                let previous_thread_state = std::mem::replace(
                    &mut self.threads[thread].blocked_by,
                    ThreadBlock::NotBlocked,
                );
                log_dbg!("Suspend thread {} from {:?}", thread, previous_thread_state);
                self.threads[thread].blocked_by =
                    ThreadBlock::Suspended(1, Box::new(previous_thread_state));
            }
        }
    }

    pub fn resume_thread(&mut self, thread: ThreadId) {
        let old = std::mem::replace(
            &mut self.threads[thread].blocked_by,
            ThreadBlock::NotBlocked,
        );
        match old {
            ThreadBlock::Suspended(count, previous_thread_state) => {
                assert!(count > 0);
                if count > 1 {
                    self.threads[thread].blocked_by =
                        ThreadBlock::Suspended(count - 1, previous_thread_state);
                } else {
                    log_dbg!("Resume thread {} to {:?}", thread, previous_thread_state);
                    self.threads[thread].blocked_by = *previous_thread_state;
                }
            }
            _ => unreachable!(),
        }
    }

    /// Block the current thread until the given mutex unlocks.
    ///
    /// Other threads also blocking on this mutex may get access first.
    /// Like all other thread blocking functions, this will suspend
    /// execution of the current host thread.
    pub fn block_on_mutex(&mut self, mutex_id: MutexId) {
        log_dbg!(
            "Thread {} blocking on mutex #{}.",
            self.current_thread,
            mutex_id
        );
        self.yield_thread(ThreadBlock::Mutex(mutex_id));
    }

    /// Locks a semaphore (decrements value of a semaphore and blocks
    /// if necessary).
    ///
    /// Like all other thread blocking functions, this will suspend
    /// execution of the current host thread (if the semaphore is
    /// currently at 0).
    pub fn sem_decrement(&mut self, sem: MutPtr<sem_t>, wait_on_lock: bool) -> bool {
        let host_sem_rc: &mut _ = self
            .libc_state
            .semaphore
            .open_semaphores
            .get_mut(&sem)
            .unwrap();
        let mut host_sem = (*host_sem_rc).borrow_mut();

        if host_sem.value > 0 {
            log_dbg!(
                "sem_decrement: semaphore {:?} is now {}",
                sem,
                host_sem.value
            );
            host_sem.value -= 1;
            return true;
        }

        if !wait_on_lock {
            log_dbg!(
                "sem_decrement: semaphore {:?} attempted decrement without waiting, failed",
                sem,
            );
            return false;
        }
        log_dbg!(
            "Thread {} is blocking on semaphore {:?}",
            self.current_thread,
            sem
        );
        host_sem.waiting.insert(self.current_thread);
        std::mem::drop(host_sem);
        // The scheduler will decrement the semaphore value when it unblocks.
        self.yield_thread(ThreadBlock::Semaphore(sem));

        true
    }

    /// Unlock a semaphore (increments value of a semaphore).
    pub fn sem_increment(&mut self, sem: MutPtr<sem_t>) {
        let host_sem_rc: &mut _ = self
            .libc_state
            .semaphore
            .open_semaphores
            .get_mut(&sem)
            .unwrap();
        let mut host_sem = (*host_sem_rc).borrow_mut();

        host_sem.value += 1;
        log_dbg!(
            "sem_increment: semaphore {:?} is now {}",
            sem,
            host_sem.value
        );
    }

    /// Blocks the current thread until the thread given finishes, writing its
    /// return value to ptr (if non-null).
    ///
    /// Note that there are no protections against joining with a detached
    /// thread, joining a thread with itself, or deadlocking joins. Callers
    /// should ensure these do not occur!
    ///
    /// Like all other thread blocking functions, this will suspend
    /// execution of the current host thread.
    pub fn join_with_thread(&mut self, joinee_thread: ThreadId, ptr: MutPtr<MutVoidPtr>) {
        log_dbg!(
            "Thread {} waiting for thread {} to finish.",
            self.current_thread,
            joinee_thread
        );
        self.yield_thread(ThreadBlock::Joining(joinee_thread, ptr));
    }

    /// End the current thread from within it, as `pthread_exit()` does: it
    /// finishes with `return_value`, and [Self::run] then cleans it up as if
    /// its start routine had returned. This never returns.
    pub fn exit_current_thread(&mut self, return_value: mem::MutVoidPtr) -> ! {
        assert_ne!(self.current_thread, 0, "The main thread can't exit");
        log_dbg!("Thread {} exiting", self.current_thread);
        let thread = &mut self.threads[self.current_thread];
        thread.return_value = Some(return_value);
        thread.state = ThreadState::Dead;
        self.yield_thread(ThreadBlock::NotBlocked);
        unreachable!("Thread {} resumed after exiting", self.current_thread);
    }

    pub fn run_app_picker<F, R>(mut self, f: F) -> R
    where
        F: FnOnce(&mut Environment) -> R + 'static,
        R: 'static,
    {
        let panic_cell = Rc::new(Cell::new(None));
        let mut app_picker_coroutine = Coroutine::new(move |yielder, mut env: Environment| {
            env.panic_cell = panic_cell.clone();
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                env.with_yielder(yielder, f)
            }));
            match res {
                // We want the environment to be dropped outside of the
                // coroutine, so send it back when we return.
                Ok(r) => (r, env),
                Err(e) => {
                    let panic_cell = env.panic_cell.clone();
                    panic_cell.set(Some(env));
                    std::panic::resume_unwind(e);
                }
            }
        });
        loop {
            if let Some(w) = self.window.as_mut() {
                w.on_main_stack = false;
            }
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                app_picker_coroutine.resume(self)
            }));
            self = match res {
                Ok(ret) => match ret {
                    corosensei::CoroutineResult::Yield(env) => env,
                    corosensei::CoroutineResult::Return((ret_val, mut env)) => {
                        if let Some(w) = env.window.as_mut() {
                            w.on_main_stack = true;
                        }
                        return ret_val;
                    }
                },
                Err(e) => {
                    log_no_panic!("Crash in app picker!");
                    // No need to get the environment back - It's local to this
                    // function anyways.
                    std::panic::resume_unwind(e);
                }
            };

            if let Some(w) = self.window.as_mut() {
                w.on_main_stack = true;
            }
            self.window
                .as_mut()
                .unwrap()
                .poll_for_events(self.options.as_ref());
            assert!(self.threads.len() == 1);
            match self.threads[0].blocked_by {
                ThreadBlock::NotBlocked => {}
                ThreadBlock::Sleeping(until) => {
                    let duration = until.duration_since(Instant::now());
                    std::thread::sleep(duration);
                }
                _ => {
                    panic!("Unexpected ThreadBlock in app picker!");
                }
            }
            self.threads[0].blocked_by = ThreadBlock::NotBlocked;
        }
    }

    /// Run the emulator. This is the main loop.
    ///
    /// It returns `true` when the app asks to go back to the app picker.
    /// The caller should then drop this `Environment` and show the app picker.
    ///
    /// It does not return `false` when the entire touchHLE process is quitting
    /// (e.g. the user closed the window, producing an `Event::Quit`); instead
    /// the guest termination path in `ui_application::exit` calls
    /// `std::process::exit` to end the process so this function never returns.
    pub fn run(mut self) -> bool {
        if self.diagnostics.is_some() {
            echo!(
                concat!(
                    "flight recorder v1 enabled: bounded API/ObjC history, dictionary ",
                    "provenance, and decoded defaults/mutation context; 5s progress ",
                    "reports (first 12); dump on panic/Home. ",
                    "No instruction tracing."
                )
            );
        }
        let mut curr_host_context = self.threads[0].host_context.take().unwrap();
        let panic_cell = self.panic_cell.clone();
        if let Some(mut gdb_server) = self.gdb_server.take() {
            gdb_server.wait_for_debugger(None, &mut self);
            self.gdb_server = Some(gdb_server);
        }
        loop {
            if self.threads[self.current_thread].state == ThreadState::Stepping {
                self.remaining_ticks = None;
            } else {
                // 100,000 ticks is an arbitrary number. It needs to be
                // reasonably large so we aren't jumping in and out of the CPU
                // interpreter or trying to poll for events too often. At the
                // same time, very large values are bad for responsiveness.
                self.remaining_ticks = Some(100_000);
            }
            let kill_current_thread;

            if let Some(w) = self.window.as_mut() {
                w.on_main_stack = false;
            }
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                curr_host_context.resume(self)
            }));
            self = match res {
                Ok(ret) => match ret {
                    corosensei::CoroutineResult::Yield(env) => {
                        // A thread that ended itself with
                        // exit_current_thread() is cleaned up like one whose
                        // start routine returned.
                        kill_current_thread =
                            env.threads[env.current_thread].state == ThreadState::Dead;
                        env
                    }
                    corosensei::CoroutineResult::Return(env) => {
                        kill_current_thread = true;
                        env
                    }
                },
                Err(e) => {
                    let Some(mut env) = panic_cell.take() else {
                        log_no_panic!("Did not recieve env from coroutine unwind, must abort!");
                        std::process::exit(-1)
                    };
                    if let Some(window) = env.window.as_mut() {
                        window.on_main_stack = true;
                    };

                    env.diagnostic_dump("panic");
                    if Self::is_null_page_access_panic(e.as_ref()) {
                        env.log_memory_fault_context("Host memory-fault");
                    }
                    echo!("Register state immediately after panic:");
                    env.dump_all_regs();
                    env.stack_trace_all();

                    if env.options.popup_errors {
                        let error_string = if let Some(s) = e.downcast_ref::<&str>() {
                            s
                        } else if let Some(s) = e.downcast_ref::<String>() {
                            s
                        } else {
                            "(non-string payload)"
                        };
                        window::show_error_messagebox(env.window.as_deref(), error_string);
                    }
                    // Put the host context back before resuming, the env will
                    // clean it up on drop.
                    let Some(thread) = env.threads.get_mut(env.current_thread) else {
                        log_no_panic!("Bad current_thread, must abort!");
                        std::process::exit(-1)
                    };
                    thread.host_context = Some(curr_host_context);
                    std::panic::resume_unwind(e);
                }
            };

            let mut old_context = if kill_current_thread {
                log_dbg!("Killing thread {}", self.current_thread);
                panic_cell.set(Some(self));
                std::mem::drop(curr_host_context);
                let Some(env) = panic_cell.take() else {
                    log_no_panic!("Did not get env back from coroutine after drop, must abort!");
                    std::process::exit(-1);
                };
                self = env;
                let stack = self.threads[self.current_thread].stack.take().unwrap();
                let stack: mem::MutVoidPtr = mem::Ptr::from_bits(*stack.start());
                log_dbg!("Freeing thread {} stack {:?}", self.current_thread, stack);
                self.mem.free(stack);
                None
            } else {
                Some(curr_host_context)
            };

            if let Some(w) = self.window.as_mut() {
                w.on_main_stack = true;
            }

            if let Some(recorder) = &mut self.diagnostics {
                let runnable = self
                    .threads
                    .iter()
                    .filter(|t| {
                        t.state != ThreadState::Dead
                            && matches!(t.blocked_by, ThreadBlock::NotBlocked)
                    })
                    .count();
                recorder.checkpoint(
                    self.current_thread,
                    self.cpu.regs(),
                    self.threads.len(),
                    runnable,
                );
            }

            if self
                .quit_deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                echo!("App took too long to quit, returning to app picker anyway.");
                self.return_to_app_picker = true;
            }
            if self.return_to_app_picker {
                self.diagnostic_dump("return-to-picker");
                // Put the context back so it's freed when Environment drops.
                let thread = self.threads.get_mut(self.current_thread).unwrap();
                assert!(thread.host_context.is_none());
                thread.host_context = old_context;
                return true;
            }

            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // To maintain responsiveness when moving the window and so on,
                // we need to poll for events occasionally, even if the app
                // isn't actively processing them.
                // Polling for events can be quite expensive, so we shouldn't do
                // this until after we've done some amount of work on the guest
                // thread, lest every single callback call pay this cost.
                if let Some(ref mut window) = self.window {
                    window.poll_for_events(&self.options);
                }
                let curr_thread_block = self.threads[self.current_thread].blocked_by.clone();
                if matches!(
                    self.threads[self.current_thread].state,
                    ThreadState::Stepping
                ) || matches!(curr_thread_block, ThreadBlock::WaitingForDebugger(_))
                {
                    let reason = if let ThreadBlock::WaitingForDebugger(reason) = curr_thread_block
                    {
                        self.threads[self.current_thread].blocked_by = ThreadBlock::NotBlocked;
                        reason.clone()
                    } else {
                        None
                    };
                    let mut gdb_server = self.gdb_server.take().unwrap();
                    gdb_server.wait_for_debugger(reason.clone(), &mut self);
                    self.gdb_server = Some(gdb_server);
                }

                let next_thread = self.schedule_next_thread();
                if next_thread != self.current_thread {
                    self.switch_thread(&mut old_context, next_thread);
                }
                assert!(old_context.is_some());
            }));
            match res {
                Ok(_) => {}
                Err(e) => {
                    if let Some(window) = self.window.as_mut() {
                        window.on_main_stack = true;
                    };
                    if self.options.popup_errors {
                        let error_string = if let Some(s) = e.downcast_ref::<&str>() {
                            s
                        } else if let Some(s) = e.downcast_ref::<String>() {
                            s
                        } else {
                            "(non-string payload)"
                        };
                        window::show_error_messagebox(self.window.as_deref(), error_string);
                    }
                    if Self::is_null_page_access_panic(e.as_ref()) {
                        self.log_memory_fault_context("Host memory-fault");
                    }
                    self.diagnostic_dump("scheduler-panic");
                    echo!("Register state immediately after panic:");
                    self.dump_all_regs();
                    self.stack_trace_all();

                    // Clean up the used host context. The ones inside the env
                    // are cleaned up by the drop handler.
                    let panic_cell = self.panic_cell.clone();
                    if let Some(ctx) = old_context {
                        panic_cell.set(Some(self));
                        std::mem::drop(ctx);
                        self = panic_cell.take().unwrap_or_else(|| {
                            log_no_panic!(
                            "Did not recieve env from coroutine unwind during drop, must abort!"
                            );
                            std::process::exit(-1)
                        });
                        std::mem::drop(self);
                    };
                    std::panic::resume_unwind(e);
                }
            }
            curr_host_context = old_context.unwrap();
        }
    }

    /// Run the emulator until the app returns control to the host. This is for
    /// host-to-guest function calls (see [abi::CallFromHost::call_from_host]).
    ///
    /// Note that this might execute code from other threads while waiting for
    /// the app to return control on the original thread!
    pub fn run_call(&mut self) {
        let old_thread = self.current_thread;
        self.run_inner();
        assert!(self.current_thread == old_thread);
    }

    /// Switch the current thread, putting the old host context (if it exists)
    /// back into its thread, and the new host context where the old one was.
    ///
    /// This also internally switches the currently used guest context.
    fn switch_thread(&mut self, old_context: &mut Option<HostContext>, new_thread: ThreadId) {
        assert!(new_thread != self.current_thread);
        assert!(self.threads[new_thread].is_running());

        log_dbg!(
            "Switching thread: {} => {}",
            self.current_thread,
            new_thread
        );

        let mut guest_ctx = self.threads[new_thread].guest_context.take().unwrap();
        self.cpu.swap_context(&mut guest_ctx);
        assert!(self.threads[self.current_thread].guest_context.is_none());
        assert!(old_context.is_some() || !self.threads[self.current_thread].is_alive());
        self.threads[self.current_thread].guest_context = Some(guest_ctx);

        let new_host_ctx = self.threads[new_thread].host_context.take().unwrap();
        self.threads[self.current_thread].host_context = old_context.take();
        *old_context = Some(new_host_ctx);
        self.current_thread = new_thread;
    }

    #[cold]
    fn is_null_page_access_panic(panic: &(dyn std::any::Any + Send)) -> bool {
        panic
            .downcast_ref::<&str>()
            .is_some_and(|message| message.contains("Attempted null-page access"))
            || panic
                .downcast_ref::<String>()
                .is_some_and(|message| message.contains("Attempted null-page access"))
    }

    #[cold]
    fn log_memory_fault_context(&self, source: &str) {
        // A CPU fault reports the instruction after the fault. A panic from a
        // guest-to-host API call instead leaves PC in the host-call stub and
        // LR at the guest caller, so report both addresses.
        let pc = self.cpu.regs()[cpu::Cpu::PC];
        let lr = self.cpu.regs()[cpu::Cpu::LR];
        let cpsr = self.cpu.cpsr();
        let start = pc.saturating_sub(4);
        let ptr = mem::ConstVoidPtr::from_bits(start);
        if let Some(bytes) = self.mem.get_bytes_fallible(ptr, 8) {
            log_no_panic!(
                "{source} context: pc={pc:#010x}, lr={lr:#010x}, cpsr={cpsr:#010x}, bytes at {start:#010x}: {bytes:02x?}"
            );
        } else {
            log_no_panic!(
                "{source} context: pc={pc:#010x}, lr={lr:#010x}, cpsr={cpsr:#010x}, instruction bytes unavailable"
            );
        }
        if let Some(symbol) = self.dyld.host_function_symbol_at_pc(&self.mem, pc) {
            log_no_panic!("{source} active host function: {symbol}");
        }

        // A wider, still bounded window makes stripped guest code around a
        // terminal fault disassemblable without enabling per-instruction
        // tracing. Include a small stack window for the same reason.
        let code_start = pc.saturating_sub(64);
        let code_ptr = mem::ConstVoidPtr::from_bits(code_start);
        if let Some(bytes) = self.mem.get_bytes_fallible(code_ptr, 96) {
            log_no_panic!(
                "{source} nearby guest code at {code_start:#010x} (96 bytes): {bytes:02x?}"
            );
        }
        // In ARM state, a null value is often produced by a nearby direct
        // function call and dereferenced a few instructions later. Record the
        // closest preceding BL target, still only at the terminal fault, so a
        // stripped producer can be inspected without instruction tracing.
        if (cpsr & cpu::Cpu::CPSR_THUMB) == 0 {
            let fault_instruction = pc.saturating_sub(4);
            for distance in (4..=32).step_by(4) {
                let instruction_address = fault_instruction.saturating_sub(distance);
                let instruction_ptr = mem::ConstVoidPtr::from_bits(instruction_address);
                let Some(bytes) = self.mem.get_bytes_fallible(instruction_ptr, 4) else {
                    continue;
                };
                let instruction = u32::from_le_bytes(bytes.try_into().unwrap());
                if instruction & 0x0f00_0000 != 0x0b00_0000 {
                    continue;
                }
                let offset = ((((instruction & 0x00ff_ffff) << 2) as i32) << 6) >> 6;
                let target = ((instruction_address as i64) + 8 + (offset as i64)) as u32;
                let target_ptr = mem::ConstVoidPtr::from_bits(target);
                if let Some(target_bytes) = self.mem.get_bytes_fallible(target_ptr, 64) {
                    log_no_panic!(
                        "{source} nearest preceding ARM BL at {instruction_address:#010x} targets {target:#010x} (64 bytes): {target_bytes:02x?}"
                    );
                    if let Some(global_slot) = decode_arm_direct_global_getter(target, target_bytes)
                    {
                        let slot_ptr = mem::ConstVoidPtr::from_bits(global_slot);
                        let slot_value = self
                            .mem
                            .get_bytes_fallible(slot_ptr, 4)
                            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()));
                        if let Some(slot_value) = slot_value {
                            log_no_panic!(
                                "{source} ARM BL target is a direct getter for global slot {global_slot:#010x}; current value: {slot_value:#010x}"
                            );
                        } else {
                            log_no_panic!(
                                "{source} ARM BL target is a direct getter for global slot {global_slot:#010x}; current value unavailable"
                            );
                        }

                        // Find a bounded set of other literal references to
                        // the same slot in this image's text. Their surrounding
                        // code can identify the initialization or setter path
                        // without per-instruction tracing.
                        let text_range = self
                            .bins
                            .iter()
                            .find(|binary| {
                                binary.text_segment_base.is_some_and(|base| {
                                    base <= target && target < binary.last_segment_end
                                })
                            })
                            .and_then(|binary| binary.get_section("__text"))
                            .map(|section| (section.addr, section.size.min(16 * 1024 * 1024)));
                        if let Some((text_start, text_size)) = text_range {
                            let needle = global_slot.to_le_bytes();
                            let text = self.mem.get_bytes_fallible(
                                mem::ConstVoidPtr::from_bits(text_start),
                                text_size,
                            );
                            let references: Vec<u32> = text
                                .map(|bytes| {
                                    bytes
                                        .chunks_exact(4)
                                        .enumerate()
                                        .filter_map(|(index, word)| {
                                            (word == needle.as_slice()).then_some(
                                                text_start + (index as u32).saturating_mul(4),
                                            )
                                        })
                                        .take(12)
                                        .collect()
                                })
                                .unwrap_or_default();
                            for &reference in &references {
                                let context_start = reference.saturating_sub(16);
                                let context_ptr = mem::ConstVoidPtr::from_bits(context_start);
                                if let Some(context) = self.mem.get_bytes_fallible(context_ptr, 48)
                                {
                                    log_no_panic!(
                                        "{source} global-slot literal reference at {reference:#010x}, code/data context at {context_start:#010x}: {context:02x?}"
                                    );
                                }
                            }
                            let literal_loads: Vec<(u32, u8, u32)> = text
                                .map(|bytes| {
                                    bytes
                                        .chunks_exact(4)
                                        .enumerate()
                                        .filter_map(|(index, word)| {
                                            let instruction_address =
                                                text_start + (index as u32).saturating_mul(4);
                                            let instruction =
                                                u32::from_le_bytes(word.try_into().unwrap());
                                            let (register, literal_address) =
                                                decode_arm_pc_relative_literal_load(
                                                    instruction_address,
                                                    instruction,
                                                )?;
                                            references.contains(&literal_address).then_some((
                                                instruction_address,
                                                register,
                                                literal_address,
                                            ))
                                        })
                                        .take(24)
                                        .collect()
                                })
                                .unwrap_or_default();
                            for (instruction_address, register, literal_address) in literal_loads {
                                let context_start = instruction_address.saturating_sub(96);
                                let context_ptr = mem::ConstVoidPtr::from_bits(context_start);
                                if let Some(context) = self.mem.get_bytes_fallible(context_ptr, 192)
                                {
                                    log_no_panic!(
                                        "{source} global-slot literal loaded by ARM instruction at {instruction_address:#010x} into R{register} from {literal_address:#010x}; code context at {context_start:#010x} (192 bytes): {context:02x?}"
                                    );
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
        let sp = self.cpu.regs()[cpu::Cpu::SP];
        let stack_ptr = mem::ConstVoidPtr::from_bits(sp);
        if let Some(bytes) = self.mem.get_bytes_fallible(stack_ptr, 64) {
            log_no_panic!("{source} guest stack at SP={sp:#010x} (64 bytes): {bytes:02x?}");
        }

        for (label, address) in [("", pc), (" caller", lr)] {
            if let Some(binary) = self.bins.iter().find(|binary| {
                binary
                    .text_segment_base
                    .is_some_and(|base| base <= address && address < binary.last_segment_end)
            }) {
                let nearest = binary
                    .exported_symbols
                    .iter()
                    .filter(|(_, &symbol_address)| (symbol_address & !1) <= address)
                    .max_by_key(|(_, &symbol_address)| symbol_address & !1);
                if let Some((name, &symbol_address)) = nearest {
                    log_no_panic!(
                        "{source}{label} nearest symbol: {}!{} at {:#010x} (+{:#x})",
                        binary.name,
                        name,
                        symbol_address,
                        address.saturating_sub(symbol_address & !1),
                    );
                } else {
                    log_no_panic!(
                        "{source}{label} image: {} (no preceding symbol available)",
                        binary.name,
                    );
                }
            }
        }

        // A guest null dereference often follows a host API returning nil or
        // receiving a bad pointer. Report host Objective-C objects in the
        // argument registers and decode a selector-shaped R1. This is bounded
        // to the terminal fault and does not change execution.
        for register in 0..=3 {
            let bits = self.cpu.regs()[register];
            let candidate: objc::id = mem::MutPtr::from_bits(bits);
            if let Some(host_object) = self.objc.get_host_object(candidate) {
                let class_name = objc::ObjC::try_read_isa(candidate, &self.mem)
                    .and_then(|class| self.objc.try_get_class_name(class))
                    .unwrap_or("<unavailable>");
                log_no_panic!(
                    "{source} Objective-C candidate: R{register}={bits:#010x} class={class_name:?}, host_type={:?}",
                    host_object.type_name(),
                );
            }
        }
        let selector_ptr = mem::ConstPtr::<u8>::from_bits(self.cpu.regs()[1]);
        if let Some(bytes) = self.mem.get_bytes_fallible(selector_ptr.cast(), 128) {
            if let Some(nul) = bytes.iter().position(|&byte| byte == 0) {
                if let Ok(selector) = std::str::from_utf8(&bytes[..nul]) {
                    log_no_panic!(
                        "{source} selector candidate: R1={:#010x} {:?}",
                        self.cpu.regs()[1],
                        selector
                    );
                }
            }
        }
    }

    #[cold]
    /// Let the debugger handle a CPU error, or panic if there's no debugger
    /// connected. Returns [true] if the CPU should step and then resume
    /// debugging, or [false] if it should resume normal execution.
    fn debug_cpu_error(&mut self, error: cpu::CpuError) {
        if matches!(error, cpu::CpuError::MemoryError) {
            self.log_memory_fault_context("CPU memory-fault");
        }

        if matches!(error, cpu::CpuError::UndefinedInstruction)
            || matches!(error, cpu::CpuError::Breakpoint)
        {
            // Rewind the PC so that it's at the instruction where the error
            // occurred, rather than the next instruction. This is necessary for
            // GDB to detect its software breakpoints. For some reason this
            // isn't correct for memory errors however.
            let instruction_len = if (self.cpu.cpsr() & cpu::Cpu::CPSR_THUMB) != 0 {
                2
            } else {
                4
            };
            self.cpu.regs_mut()[cpu::Cpu::PC] -= instruction_len;
        }

        if self.gdb_server.is_none() {
            panic!("Error during CPU execution: {error:?}");
        }

        echo!("Debuggable error during CPU execution: {:?}.", error);
        self.enter_debugger(Some(error))
    }

    /// Used to check whether a debugger is connected, and therefore whether
    /// [Environment::enter_debugger] will do something.
    pub fn is_debugging_enabled(&self) -> bool {
        self.gdb_server.is_some()
    }

    #[cfg(test)]
    pub fn new_fake_for_test() -> Self {
        Self::new_fake_for_test_with_null_segment(0)
    }

    /// A host-side fixture with thread-local storage for autorelease pools.
    #[cfg(test)]
    pub fn new_fake_for_test_with_thread() -> Self {
        let mut env = Self::new_fake_for_test();
        env.threads.push(Thread {
            state: ThreadState::Running,
            blocked_by: ThreadBlock::NotBlocked,
            framework_state: Default::default(),
            return_value: None,
            guest_context: None,
            host_context: None,
            stack: None,
        });
        env
    }

    #[cfg(test)]
    pub fn new_fake_for_test_with_null_segment(null_segment_size: u32) -> Self {
        let options = options::Options::default();
        let mut mem = mem::Mem::new();
        // Reserve before dyld allocates its stubs, as a Mach-O load does.
        if null_segment_size != 0 {
            mem.set_null_segment_size(null_segment_size);
        }
        let mut objc = objc::ObjC::new();
        let mut dyld = dyld::Dyld::new();
        dyld.do_initial_linking_with_no_bins(&mut mem, &mut objc);
        let cpu = cpu::Cpu::new(None);

        Environment {
            startup_time: Instant::now(),
            diagnostics: None,
            bundle: NullableBox::new(bundle::Bundle::new_fake_bundle()),
            fs: NullableBox::new(fs::Fs::new_fake_fs()),
            window: None,
            openal_manager: unsafe { NullableBox::null() },
            mem: NullableBox::new(mem),
            bins: Vec::new(),
            objc: NullableBox::new(objc),
            dyld: NullableBox::new(dyld),
            cpu: NullableBox::new(cpu),
            current_thread: 0,
            threads: Vec::new(),
            libc_state: Default::default(),
            mutex_state: Default::default(),
            framework_state: Default::default(),
            options: NullableBox::new(options),
            gdb_server: None,
            env_vars: Default::default(),
            dump_file: None,
            is_app_picker: false,
            return_to_app_picker: false,
            quit_deadline: None,
            yielder: std::ptr::null(),
            remaining_ticks: None,
            panic_cell: Rc::new(Cell::new(None)),
        }
    }

    /// Suspend execution and hand control to the connected debugger.
    /// You should precede this call with a log message that explains why the
    /// debugger is being invoked. The return value is the same as
    /// [gdb::GdbServer::wait_for_debugger]'s.
    ///
    /// Note that this also yields the thread - take care!
    pub fn enter_debugger(&mut self, reason: Option<cpu::CpuError>) {
        // GDB doesn't seem to manage to produce a useful stack trace, so
        // let's print our own.
        self.stack_trace_current();

        self.yield_thread(ThreadBlock::WaitingForDebugger(reason));
    }

    #[inline(always)]
    /// Respond to the new CPU state (do nothing, execute an SVC or enter
    /// debugging) and decide what to do next.
    fn handle_cpu_state(&mut self, state: cpu::CpuState) -> ThreadNextAction {
        match state {
            cpu::CpuState::Normal => ThreadNextAction::Continue,
            cpu::CpuState::Svc(svc) => {
                // The program counter is pointing at the
                // instruction after the SVC, but we want the
                // address of the SVC itself.
                let svc_pc = self.cpu.regs()[cpu::Cpu::PC] - 4;
                match svc {
                    dyld::Dyld::SVC_RETURN_TO_HOST => {
                        assert!(
                            svc_pc == self.dyld.return_to_host_routine().addr_without_thumb_bit()
                        );
                        // Normal return from host-to-guest call.
                        ThreadNextAction::ReturnToHost
                    }
                    dyld::Dyld::SVC_LAZY_LINK
                    | dyld::Dyld::SVC_LAZY_LINK_RET_FLAG
                    | dyld::Dyld::SVC_LINKED_FUNCTIONS_BASE.. => {
                        if let Some(f) = self.dyld.get_svc_handler(
                            &self.bins,
                            &mut self.mem,
                            &mut self.cpu,
                            svc_pc,
                            svc,
                        ) {
                            let diagnostic_call = if self.diagnostics.is_some() {
                                let symbol = self
                                    .dyld
                                    .host_function_symbol_at_pc(
                                        &self.mem,
                                        self.cpu.regs()[cpu::Cpu::PC],
                                    )
                                    .unwrap_or("<unresolved-host-call>");
                                self.diagnostic_api_begin(symbol)
                            } else {
                                None
                            };
                            f.call_from_guest(self);
                            self.diagnostic_end(diagnostic_call, false);
                            if let Some(len) = self.options.zero_stack_after_guest_to_host_call {
                                log_once!("Applying zeroing of stack after guest to host call.");
                                let start = self.cpu.regs()[cpu::Cpu::SP] - len;
                                self.mem
                                    .bytes_at_mut(mem::Ptr::from_bits(start), len)
                                    .fill(0);
                            }

                            if self.return_to_app_picker {
                                return ThreadNextAction::ReturnToHost;
                            }

                            // On entry_size 4 return here since there's
                            // no space to add a ret after the svc call
                            if svc & dyld::Dyld::SVC_LAZY_LINK_RET_FLAG != 0 {
                                let lr = self.cpu.regs()[cpu::Cpu::LR];
                                self.cpu.branch(GuestFunction::from_addr_with_thumb_bit(lr));
                            }
                            ThreadNextAction::Continue
                        } else {
                            self.cpu.regs_mut()[cpu::Cpu::PC] = svc_pc;
                            ThreadNextAction::Continue
                        }
                    }
                    dyld::Dyld::SVC_THREAD_EXIT => {
                        unimplemented!("TODO: implement exit routines for threads!")
                    }
                }
            }
            cpu::CpuState::Error(e) => ThreadNextAction::DebugCpuError(e),
        }
    }

    fn run_inner(&mut self) {
        let initial_thread = self.current_thread;
        assert!(self.threads[initial_thread].is_running());
        assert!(self.threads[initial_thread].guest_context.is_none());

        loop {
            while self
                .remaining_ticks
                .is_none_or(|remaining_ticks| remaining_ticks > 0)
            {
                let state = self
                    .cpu
                    .run_or_step(&mut self.mem, self.remaining_ticks.as_mut());

                match self.handle_cpu_state(state) {
                    ThreadNextAction::Continue => {
                        if self.remaining_ticks.is_none() {
                            break;
                        }
                    }
                    ThreadNextAction::ReturnToHost => return,
                    ThreadNextAction::DebugCpuError(e) => {
                        self.debug_cpu_error(e);
                    }
                }
            }
            self.yield_thread(ThreadBlock::NotBlocked);
        }
    }

    /// Yield the current thread, suspending execution and handing control back
    /// to the executor ([Self::run]), waiting until the current `thread_block`
    /// condition is met.
    pub fn yield_thread(&mut self, thread_block: ThreadBlock) {
        assert!(!self.threads[self.current_thread].is_blocked());
        log_dbg!(
            "Thread {} yielding on {:?}",
            self.current_thread,
            thread_block
        );
        unsafe {
            self.threads[self.current_thread].blocked_by = thread_block;
            let yielder = self.yielder.as_ref().unwrap();
            self.yielder = std::ptr::null();
            let panic_cell = self.panic_cell.clone();
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let env = std::mem::replace(self, Self::new_fake());
                yielder.suspend(env)
            }));
            match res {
                Ok(env) => {
                    let _ = std::mem::replace(self, env);
                    self.yielder = yielder;
                }
                Err(payload) => {
                    let Some(env) = panic_cell.take() else {
                        log_no_panic!("Did not recieve env for coroutine unwind, must abort!");
                        std::process::exit(-1)
                    };
                    let _ = std::mem::replace(self, env);
                    self.yielder = yielder;
                    std::panic::resume_unwind(payload);
                }
            }
        }
        assert!(!self.threads[self.current_thread].is_blocked());
    }

    /// Find the next thread to execute, and set it up to be switched to.
    ///
    /// This also handles all the required bookkeeping (unlocking mutexes,
    /// decrementing semaphores, setting the thread to be unblocked, etc.).
    /// It is not required that the thread is switched to immediately.
    fn schedule_next_thread(&mut self) -> ThreadId {
        // GDB can allow schedule_next_thread to be called twice in a row -
        // we make sure that this works by immediately fufilling conditions
        // (relocking mutexes, decrementing semaphores, etc.)!
        loop {
            // Try to find a new thread to execute, starting with the thread
            // following the one currently executing.
            let mut next_awakening: Option<Instant> = None;
            for i in 0..self.threads.len() {
                let thread_id = (self.current_thread + 1 + i) % self.threads.len();
                let candidate = &mut self.threads[thread_id];

                if !candidate.is_running() {
                    continue;
                }
                match candidate.blocked_by {
                    ThreadBlock::Sleeping(sleeping_until) => {
                        if sleeping_until <= Instant::now() {
                            log_dbg!("Thread {} finished sleeping.", thread_id);
                            candidate.blocked_by = ThreadBlock::NotBlocked;
                            return thread_id;
                        } else {
                            next_awakening = match next_awakening {
                                None => Some(sleeping_until),
                                Some(other) => Some(other.min(sleeping_until)),
                            };
                        }
                    }
                    ThreadBlock::Mutex(mutex_id) => {
                        if !self.mutex_state.mutex_is_locked(mutex_id) {
                            log_dbg!("Thread {} was unblocked due to mutex #{} unlocking, relocking mutex.", thread_id, mutex_id);
                            self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                            self.relock_unblocked_mutex_for_thread(thread_id, mutex_id);
                            return thread_id;
                        }
                    }
                    ThreadBlock::Semaphore(sem) => {
                        let host_sem_rc: &mut _ = self
                            .libc_state
                            .semaphore
                            .open_semaphores
                            .get_mut(&sem)
                            .unwrap();
                        let mut host_sem = (*host_sem_rc).borrow_mut();

                        if host_sem.value > 0 {
                            log_dbg!(
                                "Thread {} has awaken on semaphore {:?} with value {}",
                                thread_id,
                                sem,
                                host_sem.value
                            );
                            host_sem.value -= 1;
                            host_sem.waiting.remove(&self.current_thread);
                            self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                            return thread_id;
                        }
                    }
                    ThreadBlock::Condition(cond, deadline) => {
                        let host_cond = self
                            .libc_state
                            .pthread
                            .cond
                            .condition_variables
                            .get_mut(&cond)
                            .unwrap();
                        let mutex = host_cond.curr_mutex.unwrap();
                        if host_cond
                            .waking
                            .front()
                            .is_some_and(|waking_thread| *waking_thread == thread_id)
                            && !self.mutex_state.mutex_is_locked(mutex)
                        {
                            log_dbg!("Thread {} is unblocking on cond var {:?}.", thread_id, cond);
                            host_cond.waking.pop_front();
                            self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                            self.relock_unblocked_mutex_for_thread(thread_id, mutex);
                            return thread_id;
                        } else if let Some(deadline) = deadline {
                            let time = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap();
                            if deadline > time {
                                // Make sure we wake up for the timeout if
                                // every thread is blocked.
                                let timeout_at = Instant::now() + (deadline - time);
                                next_awakening = Some(match next_awakening {
                                    None => timeout_at,
                                    Some(other) => other.min(timeout_at),
                                });
                            } else if !host_cond.waking.contains(&thread_id)
                                && !self.mutex_state.mutex_is_locked(mutex)
                            {
                                // A thread that has already been signalled
                                // will wake up normally, and a timed-out
                                // thread still has to wait to get the mutex
                                // back.
                                log_dbg!(
                                    "Thread {} is timed out on cond var {:?}.",
                                    thread_id,
                                    cond
                                );
                                assert!(!host_cond.timed_out.contains(&thread_id));
                                host_cond.timed_out.insert(thread_id);

                                host_cond.waiting.retain(|&t| t != thread_id);

                                self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                                self.relock_unblocked_mutex_for_thread(thread_id, mutex);
                                return thread_id;
                            }
                        }
                    }
                    ThreadBlock::Joining(joinee_thread, ptr) => {
                        if !self.threads[joinee_thread].is_alive() {
                            log_dbg!(
                                "Thread {} joining with now finished thread {}.",
                                self.current_thread,
                                joinee_thread
                            );
                            // Write the return value, unless the pointer to
                            // write to is null.
                            if !ptr.is_null() {
                                self.mem
                                    .write(ptr, self.threads[joinee_thread].return_value.unwrap());
                            }
                            self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                            return thread_id;
                        }
                    }
                    ThreadBlock::NotBlocked => {
                        return thread_id;
                    }
                    ThreadBlock::WaitingForDebugger(_) => unreachable!(),
                    ThreadBlock::Suspended(cnt, _) => {
                        assert!(cnt > 0);
                    }
                    ThreadBlock::FileObjectLock(file_ptr) => {
                        // TODO: fairness
                        let acquired = self.libc_state.stdio.try_acquire_file_object_lock(
                            &mut self.mem,
                            file_ptr,
                            thread_id,
                        );
                        if acquired {
                            self.threads[thread_id].blocked_by = ThreadBlock::NotBlocked;
                            return thread_id;
                        }
                    }
                }
            }

            // All suitable threads are blocked and at least one is asleep.
            // Sleep until one of them wakes up.
            if let Some(next_awakening) = next_awakening {
                let duration = next_awakening.duration_since(Instant::now());
                log_dbg!("All threads blocked/asleep, sleeping for {:?}.", duration);
                std::thread::sleep(duration);
                // Try again, there should be some thread awake now (or
                // there will be soon, since timing is approximate).
                continue;
            } else {
                // This should hopefully not happen, but if a thread is
                // blocked on another thread waiting for a deferred return,
                // it could.
                panic!("No active threads, program has deadlocked!");
            }
        }
    }

    fn set_up_initial_env_vars(&mut self) {
        // TODO: Provide all the system environment variables an app might
        // expect to find.

        // Initialize HOME envvar
        let home_value_cstr = self
            .mem
            .alloc_and_write_cstr(self.fs.home_directory().as_str().as_bytes());
        self.env_vars.insert(b"HOME".to_vec(), home_value_cstr);

        // The app's tmp folder. As on a device, it ends with a slash.
        let tmp_dir = format!("{}/tmp/", self.fs.home_directory().as_str());
        let tmp_value_cstr = self.mem.alloc_and_write_cstr(tmp_dir.as_bytes());
        self.env_vars.insert(b"TMPDIR".to_vec(), tmp_value_cstr);

        if self.bundle.bundle_identifier() == "com.majesticsoftware.blot" {
            // Blot embeds ImageMagick and performs its effects inside the
            // emulated process. Limit it to the single guest CPU touchHLE
            // exposes; extra ImageMagick workers only add contention and can
            // make an otherwise responsive scene appear to freeze.
            let thread_limit = self.mem.alloc_and_write_cstr(b"1");
            self.env_vars
                .insert(b"MAGICK_THREAD_LIMIT".to_vec(), thread_limit);
        }
    }

    fn get_sorted_bin_indices(&self) -> Result<Vec<usize>, String> {
        let dylib_graph: Vec<BinaryDependencyNode> = self
            .bins
            .iter()
            .map(|bin| BinaryDependencyNode {
                name: bin.name.clone(),
                dependencies: bin.dynamic_libraries.clone(),
            })
            .collect();

        generate_binary_load_order(&dylib_graph)
    }

    /// Run a function using window and options on the parent stack if we are
    /// inside a coroutine, or run it directly if we aren't. Some
    /// [window::Window] functions require to be called inside this function.
    ///
    /// Android's ABI seems to dislike if certain functions aren't called from
    /// the main stack. Since corosensei uses seperate stacks to run
    /// coroutines, Android doesn't recognize it as the main stack, so those
    /// functions need to be run on the main stack. Unfortunately, there's no
    /// documentation of which functions need to be called with this, so we
    /// have to check ourselves.
    pub fn on_parent_stack_in_coroutine<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut window::Window, &mut options::Options) -> R + Send,
    {
        struct WindowWrapper<'a> {
            window: &'a mut window::Window,
        }
        // SAFETY: we're not sending across threads, we're only sending across
        // the coroutine boundary so it's ok.
        unsafe impl Send for WindowWrapper<'_> {}

        if !self.yielder.is_null() {
            unsafe {
                let yielder = self.yielder.as_ref().unwrap();
                let wrapped = WindowWrapper {
                    window: self.window.as_mut().unwrap(),
                };
                let res = yielder.on_parent_stack(|| {
                    let wrapped = wrapped;
                    wrapped.window.on_main_stack = true;
                    f(wrapped.window, self.options.as_mut())
                });
                self.window.as_mut().unwrap().on_main_stack = false;
                res
            }
        } else {
            if let Some(w) = self.window.as_mut() {
                w.on_main_stack = true;
            }
            f(self.window.as_mut().unwrap(), self.options.as_mut())
        }
    }
}

impl Drop for Environment {
    // Clean up all the remaining HostContexts. This isn't strictly required,
    // since this should only occur after a sucessful panic or the app ending,
    // but it is a bit cleaner and avoids confusion inside the logs.
    fn drop(&mut self) {
        if self.objc.is_null() {
            return;
        }
        if let Some(w) = self.window.as_mut() {
            w.on_main_stack = false;
        }
        if self.threads.is_empty()
            || self
                .threads
                .iter()
                .all(|thread| thread.host_context.is_none())
        {
            ENVIRONMENT_INSTANCE_EXISTS.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        }
        unsafe {
            let mut env = std::mem::replace(self, Environment::new_fake());
            let panic_cell = env.panic_cell.clone();
            let threads_len = env.threads.len();
            for i in 0..threads_len {
                let host_context = env.threads[i].host_context.take();
                panic_cell.set(Some(env));
                std::mem::drop(host_context);
                env = panic_cell.take().unwrap_or_else(|| {
                    log_no_panic!(
                        "Did not recieve env from coroutine unwind during drop, must abort!"
                    );
                    std::process::exit(-1)
                });
            }
            *self = env;
        }
        ENVIRONMENT_INSTANCE_EXISTS.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod memory_fault_diagnostic_tests {
    use super::{decode_arm_direct_global_getter, decode_arm_pc_relative_literal_load};

    #[test]
    fn flight_recorder_preserves_real_sysctl_failure_through_svc() {
        use super::*;
        // Keep one unsupported six-component query as a strict diagnostic
        // fixture. B'Blast's recovered AF_LINK query is now implemented, so
        // this synthetic AF_UNSPEC variant must continue to fail unchanged.
        for enabled in [false, true] {
            let mut env = Environment::new_fake_for_test();
            env.diagnostics = enabled.then(crate::diagnostics::Recorder::new);
            let name = env.mem.alloc(24).cast::<u32>();
            let mib = [4, 17, 0, 0, 3, 2];
            for (i, word) in mib.into_iter().enumerate() {
                env.mem.write(name + i as u32, word);
            }
            let stack = env.mem.alloc(16).cast::<u32>();
            for i in 0..4 {
                env.mem.write(stack + i, 0u32);
            }
            let f = env
                .dyld
                .create_proc_address(&mut env.mem, &mut env.cpu, "_sysctl")
                .unwrap();
            let svc: u32 = env
                .mem
                .read(mem::ConstPtr::from_bits(f.addr_without_thumb_bit()));
            env.cpu.regs_mut()[0..4].copy_from_slice(&[name.to_bits(), 6, 0, 0]);
            env.cpu.regs_mut()[cpu::Cpu::SP] = stack.to_bits();
            env.cpu.regs_mut()[cpu::Cpu::PC] = f.addr_without_thumb_bit() + 4;
            let before = *env.cpu.regs();
            let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                env.handle_cpu_state(cpu::CpuState::Svc(svc & 0x00ff_ffff));
            }))
            .expect_err("unsupported query must still panic");
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap();
            assert!(
                message.contains("Unknown six-component sysctl parameter"),
                "{message}"
            );
            assert_eq!(*env.cpu.regs(), before);
            if let Some(recorder) = &mut env.diagnostics {
                assert_eq!(
                    recorder.last_entry(),
                    Some((b"_sysctl".as_slice(), mib.as_slice()))
                );
                recorder.dump("automated six-component sysctl reproducer");
            }
        }
    }

    #[test]
    fn decodes_direct_global_getter() {
        let bytes = [
            0x04, 0x00, 0x9f, 0xe5, 0x00, 0x00, 0x90, 0xe5, 0x1e, 0xff, 0x2f, 0xe1, 0x7c, 0xf8,
            0xac, 0x00,
        ];
        assert_eq!(
            decode_arm_direct_global_getter(0x0047_cff4, &bytes),
            Some(0x00ac_f87c)
        );

        let mut not_a_getter = bytes;
        not_a_getter[4] = 0x04;
        assert_eq!(
            decode_arm_direct_global_getter(0x0047_cff4, &not_a_getter),
            None
        );
    }

    #[test]
    fn decodes_arm_pc_relative_literal_load() {
        // ldr r0, [pc, #8] at 0x47d16c loads the word at 0x47d17c.
        assert_eq!(
            decode_arm_pc_relative_literal_load(0x0047_d16c, 0xe59f_0008),
            Some((0, 0x0047_d17c))
        );
        // ldr r3, [pc, #-16] at 0x1000 loads the word at 0xff8.
        assert_eq!(
            decode_arm_pc_relative_literal_load(0x1000, 0xe51f_3010),
            Some((3, 0x0ff8))
        );
        assert_eq!(
            decode_arm_pc_relative_literal_load(0x1000, 0xe59f_0008 | (1 << 21)),
            None
        );
    }
}

#[cfg(test)]
mod dylib_sorting_tests {
    use std::collections::HashSet;

    use super::*;

    fn create_dylib_graph(bin_configs: &[(&str, &[&str])]) -> Vec<BinaryDependencyNode> {
        bin_configs
            .iter()
            .map(|(name, dependencies)| BinaryDependencyNode {
                name: name.to_string(),
                dependencies: dependencies.iter().map(|s| s.to_string()).collect(),
            })
            .collect()
    }

    /// Verify dylib sort by checking that no dependents are needed
    /// before their import
    fn verify_sort(graph: &[BinaryDependencyNode], sorted_indices: &[usize]) {
        assert_eq!(sorted_indices.len(), graph.len());

        let bin_to_index: HashMap<_, _> = graph
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.name.as_str(), idx))
            .collect();

        let mut loaded_dylibs = HashSet::new();

        for &index in sorted_indices {
            let current_bin = graph.get(index).unwrap();

            for dependency in current_bin
                .dependencies
                .iter()
                .map(|path| path.strip_prefix("/usr/lib/").unwrap_or(path.as_str()))
            {
                // Ignore dependencies that are not included in packaged dylibs
                let Some(&dylib_index) = bin_to_index.get(dependency) else {
                    continue;
                };

                assert!(loaded_dylibs.contains(&dylib_index));
            }

            loaded_dylibs.insert(index);
        }
    }

    #[test]
    fn test_no_dependencies() {
        let dylib_graph = create_dylib_graph(&[]);
        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();
        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_single_bin() {
        let dylib_graph = create_dylib_graph(&[("A", &[])]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_linear_dependencies() {
        // A -> B -> C -> D
        let dylib_graph = create_dylib_graph(&[
            ("A", &[]),
            ("B", &["/usr/lib/A"]),
            ("C", &["/usr/lib/B"]),
            ("D", &["/usr/lib/C"]),
        ]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_diamond_dependencies() {
        // A -> B -> D
        //  \-> C -/
        let dylib_graph =
            create_dylib_graph(&[("A", &[]), ("B", &["A"]), ("C", &["A"]), ("D", &["B", "C"])]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_with_isolated_nodes() {
        // A -> B
        // C
        // D
        let dylib_graph = create_dylib_graph(&[("A", &[]), ("B", &["A"]), ("C", &[]), ("D", &[])]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_complex_dependency_graph() {
        // A -> B -> D
        // A -> C -> E
        // F -> G
        // H
        let dylib_graph = create_dylib_graph(&[
            ("A", &[]),
            ("B", &["A"]),
            ("C", &["A"]),
            ("D", &["B"]),
            ("E", &["C"]),
            ("F", &[]),
            ("G", &["F"]),
            ("H", &[]),
        ]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_with_external_dependencies() {
        let dylib_graph = create_dylib_graph(&[
            ("A", &["external1"]),
            ("B", &["A", "external2"]),
            ("C", &["B"]),
        ]);

        let sorted_indices = generate_binary_load_order(&dylib_graph).unwrap();

        verify_sort(&dylib_graph, &sorted_indices);
    }

    #[test]
    fn test_cycle() {
        // A -> B -> C -> A
        let dylib_graph = create_dylib_graph(&[("A", &["C"]), ("B", &["A"]), ("C", &["B"])]);

        let result = generate_binary_load_order(&dylib_graph);

        assert!(
            result.is_err(),
            "Sort should detect cycle and return an error"
        );
    }

    #[test]
    fn test_self_dependency() {
        let dylib_graph = create_dylib_graph(&[("A", &["A"])]);

        let result = generate_binary_load_order(&dylib_graph);

        assert!(
            result.is_err(),
            "Sort should detect self-dependency as a cycle and return an error"
        );
    }

    /// Runs libstdc++'s static initializers, in batched (ticks-based) mode,
    /// on the real bundled libgcc and libstdc++: real guest code that
    /// exercises `svc` handling, stores through the memory callbacks, and
    /// CP15 barriers, all early enough to fail fast if any of them break.
    mod libstdcxx_static_initializers {
        use crate::cpu::{Cpu, CpuState};
        use crate::mach_o::{MachO, SectionType};
        use crate::mem::{ConstPtr, Mem, Ptr};
        use crate::objc::ObjC;

        #[test]
        fn trace_libstdcxx_static_initializers() {
            let mut mem = Mem::new();
            mem.set_null_segment_size(0x1000);

            let libgcc_bytes = std::fs::read("dylibs/libgcc_s.1.dylib").unwrap();
            let libgcc = MachO::load_from_bytes(
                &libgcc_bytes,
                &mut mem,
                "libgcc_s.1.dylib".to_string(),
                0x30000000,
            )
            .unwrap();

            let libstdcxx_bytes = std::fs::read("dylibs/libstdc++.6.0.9.dylib").unwrap();
            let libstdcxx = MachO::load_from_bytes(
                &libstdcxx_bytes,
                &mut mem,
                "libstdc++.6.0.9.dylib".to_string(),
                0x3748a000,
            )
            .unwrap();

            let mut objc = ObjC::new();
            let mut dyld = crate::dyld::Dyld::new();
            let bins = [libgcc, libstdcxx];
            dyld.do_initial_linking(&bins, &mut mem, &mut objc, None).unwrap();

            let mut cpu = Cpu::new(None);
            cpu.regs_mut()[Cpu::SP] = 0xFFFFF000;
            cpu.regs_mut()[Cpu::LR] = dyld.return_to_host_routine().addr_with_thumb_bit();

            let Some(section) = bins[1].get_section(SectionType::ModInitFuncPointers) else {
                panic!("no ModInitFuncPointers section");
            };
            assert!(section.size % 4 == 0);
            let base: crate::mem::ConstPtr<crate::abi::GuestFunction> =
                Ptr::from_bits(section.addr);
            let count = section.size / 4;
            eprintln!("{count} static initializer(s) in libstdc++");

            for idx in 0..count {
                let func = mem.read(base + idx);
                eprintln!("=== Calling static initializer {idx} at {:?}", func);
                cpu.branch(func);
                cpu.regs_mut()[Cpu::LR] = dyld.return_to_host_routine().addr_with_thumb_bit();

                for step in 0..500000u32 {
                    let pc = cpu.pc_with_thumb_bit();
                    let inst: u32 = mem.read(ConstPtr::from_bits(pc.addr_without_thumb_bit()));
                    let mut ticks = 100000u64;
                    let state = cpu.run_or_step(&mut mem, Some(&mut ticks));
                    match state {
                        CpuState::Normal => {}
                        CpuState::Svc(svc) => {
                            if svc == crate::dyld::Dyld::SVC_RETURN_TO_HOST {
                                eprintln!(
                                "initializer {idx} returned to host cleanly after {step} steps."
                            );
                                break;
                            }
                            eprintln!(
                                "initializer {idx} step {step}: SVC {svc:#x} at pc={:?}, skipping",
                                pc
                            );
                            break;
                        }
                        CpuState::Error(e) => {
                            eprintln!(
                                "initializer {idx} step {step}: CPU ERROR {:?} at pc={:?} \
                             inst={:#010x} sp={:#010x} lr={:#010x} r0={:#010x} r1={:#010x}",
                                e,
                                pc,
                                inst,
                                cpu.regs()[Cpu::SP],
                                cpu.regs()[Cpu::LR],
                                cpu.regs()[0],
                                cpu.regs()[1]
                            );
                            panic!("CPU error during trace: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}
