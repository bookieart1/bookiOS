/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Opt-in, bounded flight recorder. Recording never calls guest code, formats
//! text, allocates, writes logs, or changes guest state. This records
//! boundaries, not instructions, and is not proof that a nil return is an
//! error.

use crate::frameworks::foundation::ns_string;
use crate::fs::GuestPath;
use crate::mem::{ConstVoidPtr, Mem};
use crate::objc::{id, ObjC};
use crate::Environment;
use std::time::{Duration, Instant};

const RECENT: usize = 96;
const SLOW_RECENT: usize = 32;
const SIGNALS: usize = 24;
const PROGRESS: usize = 24;
const CRASH_CALLERS: usize = 8;
const PROVENANCE: usize = 128;
const VALUE_ORIGINS: usize = 512;
const VALUE_ORIGIN_WAYS: usize = 4;
const BOUNDARY_DETAILS: usize = 8;
const BUNDLE_CANDIDATES: usize = 16;
const BUNDLE_DIRECTORY_ENTRIES: usize = 24;
const COLLECTION_EVENTS: usize = 48;
const INVOCATION_EVENTS: usize = 32;
const ARRAY_EVENTS: usize = 256;
const WATCHED_OBJECTS: usize = 16;
const OBJECT_LIFETIME_EVENTS: usize = 64;
const DICTIONARY_ORIGINS: usize = 1024;
const UNRESOLVED_RELOCATIONS: usize = 32;
const STRING_SEARCHES: usize = 32;
const TOUCH_EVENTS: usize = 64;
const SELECTOR_EVENTS: usize = 64;
const HEALTHY_PRESENT_INTERVAL: Duration = Duration::from_millis(100);
const SLOW_PRESENT_INTERVAL: Duration = Duration::from_millis(180);
const HEALTHY_PRESENT_STREAK: u16 = 30;
const SLOW_PRESENT_STREAK: u8 = 3;
#[cfg(test)]
const MAX_RECORDER_BYTES: usize = 384 * 1024;

#[derive(Clone, Copy, Debug)]
struct Text {
    bytes: [u8; 80],
    len: usize,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            bytes: [0; 80],
            len: 0,
        }
    }
}
impl Text {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut text = Self::default();
        text.len = bytes.len().min(text.bytes.len());
        text.bytes[..text.len].copy_from_slice(&bytes[..text.len]);
        text
    }
    fn guest(mem: &Mem, address: u32) -> Self {
        let mut text = Self::default();
        for i in 0..text.bytes.len() {
            let Some(address) = address.checked_add(i as u32) else {
                break;
            };
            let Some(byte) = mem.get_bytes_fallible(ConstVoidPtr::from_bits(address), 1) else {
                break;
            };
            if byte[0] == 0 {
                break;
            }
            text.bytes[i] = byte[0];
            text.len += 1;
        }
        text
    }
    fn path_tail(path: &str) -> Self {
        let bytes = path.as_bytes();
        if bytes.len() < 80 {
            return Self::from_bytes(bytes);
        }
        let mut text = Self::default();
        text.append(b"[tail] ");
        text.append(&bytes[bytes.len() - 72..]);
        text
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
    fn append(&mut self, bytes: &[u8]) {
        let remaining = self.bytes.len().saturating_sub(self.len);
        let copy = bytes.len().min(remaining);
        self.bytes[self.len..self.len + copy].copy_from_slice(&bytes[..copy]);
        self.len += copy;
    }
}
impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Escape control characters so guest data cannot impersonate log lines.
        for &b in self.bytes() {
            if (32..127).contains(&b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:02x}")?;
            }
        }
        if self.len == self.bytes.len() {
            write!(f, "[truncated]")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ShortText {
    bytes: [u8; 48],
    len: u8,
}
impl Default for ShortText {
    fn default() -> Self {
        Self {
            bytes: [0; 48],
            len: 0,
        }
    }
}
impl ShortText {
    fn from_text(text: Text) -> Self {
        let mut short = Self::default();
        let len = text.len.min(short.bytes.len());
        short.bytes[..len].copy_from_slice(&text.bytes[..len]);
        short.len = len as u8;
        short
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
}
impl std::fmt::Display for ShortText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &b in self.bytes() {
            if (32..127).contains(&b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:02x}")?;
            }
        }
        if usize::from(self.len) == self.bytes.len() {
            write!(f, "[truncated]")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum DefaultsDomain {
    #[default]
    None,
    Application,
    Global,
    Registration,
    Missing,
}
impl DefaultsDomain {
    fn from_name(name: &str) -> Self {
        match name {
            "application" => Self::Application,
            "global" => Self::Global,
            "registration" => Self::Registration,
            "missing" => Self::Missing,
            _ => Self::None,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Application => "application",
            Self::Global => "global",
            Self::Registration => "registration",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Call {
    sequence: u64,
    thread: usize,
    kind: &'static str,
    name: Text,
    pc: u32,
    lr: u32,
    sp: u32,
    stack: [u32; 4],
    stack_len: usize,
    args: [u32; 4],
    receiver: u32,
    isa: u32,
    detail: Text,
    mib: [u32; 8],
    mib_len: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Event {
    sequence: u64,
    call: Call,
    result: Option<[u32; 2]>,
    note: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
struct Provenance {
    object: u32,
    event: Event,
    context: DefaultsDomain,
    source_argument: ShortText,
}

/// Compact, bounded last-return provenance for non-nil Objective-C results.
/// This deliberately survives both the recent-event ring and object
/// deallocation so a later stale consumption can still be tied to its most
/// recent API producer. It is not an allocation or guest-store trace.
#[derive(Clone, Copy, Debug, Default)]
struct ValueOrigin {
    sequence: u64,
    value: u32,
    kind: &'static str,
    name: ShortText,
    receiver: u32,
    detail: ShortText,
    pc: u32,
    lr: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct BoundaryDetail {
    call_sequence: u64,
    context: DefaultsDomain,
    arguments: [Text; 2],
}

#[derive(Clone, Copy, Debug, Default)]
struct DefaultsLookup {
    receiver: u32,
    key: u32,
    result: u32,
    domain: DefaultsDomain,
}

#[derive(Clone, Copy, Debug, Default)]
struct CollectionEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    dictionary: u32,
    key: u32,
    value: u32,
    key_retain: u32,
    key_release: u32,
    value_retain: u32,
    value_release: u32,
    value_class: Text,
    raw_words: [u32; 3],
    raw_word_count: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct ObjectLifetimeEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    object: u32,
    class: Text,
    before: u32,
    after: u32,
    pc: u32,
    lr: u32,
    context: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct DictionaryOrigin {
    sequence: u64,
    object: u32,
    mutable: bool,
    context: &'static str,
    option: u32,
    pc: u32,
    lr: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct InvocationEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    invocation: u32,
    target: u32,
    selector: u32,
    selector_name: Text,
    target_class: Text,
    argument_index: i32,
    argument_count: u32,
    set_mask: u32,
    pc: u32,
    lr: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct ArrayEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    array: u32,
    index: u32,
    object: u32,
    before: u32,
    after: u32,
    pc: u32,
    lr: u32,
    caller_code: [u8; 16],
    caller_code_len: u8,
    array_class: Text,
    object_class: Text,
}

#[derive(Clone, Copy, Debug, Default)]
struct SelectorEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    run_loop: u32,
    target: u32,
    selector: Text,
    argument: u32,
    delay_ms: i64,
    pc: u32,
    lr: u32,
}

struct Ring<T: Copy + Default, const N: usize> {
    items: [T; N],
    written: u64,
}
impl<T: Copy + Default, const N: usize> Default for Ring<T, N> {
    fn default() -> Self {
        Self {
            items: [T::default(); N],
            written: 0,
        }
    }
}
impl<T: Copy + Default, const N: usize> Ring<T, N> {
    fn push(&mut self, item: T) {
        self.items[self.written as usize % N] = item;
        self.written += 1;
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        let count = self.written.min(N as u64);
        (self.written - count..self.written).map(|i| &self.items[i as usize % N])
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Progress {
    ms: u128,
    thread: usize,
    pc: u32,
    lr: u32,
    threads: usize,
    runnable: usize,
    api: u64,
    objc: u64,
    run_loops: u64,
    presents: u64,
    yields: u64,
    sources: [usize; 4],
}

#[derive(Default)]
struct SlowCapture {
    ms: u128,
    gap_ms: u128,
    boundary_delta: u64,
    presents: u64,
    had_healthy_baseline: bool,
    recent: Ring<Event, SLOW_RECENT>,
}

struct PendingSlowCapture {
    capture: SlowCapture,
    streak: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct UnresolvedRelocation {
    image: Text,
    symbol: Text,
    first_address: u32,
    count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct StringSearch {
    thread: usize,
    pc: u32,
    lr: u32,
    haystack: u32,
    needle: u32,
    result: u32,
    text: Text,
    pattern: Text,
}

#[derive(Clone, Copy, Debug, Default)]
struct TouchEvent {
    sequence: u64,
    thread: usize,
    action: &'static str,
    touch: u32,
    view: u32,
    active: usize,
    x: f32,
    y: f32,
    pc: u32,
    lr: u32,
}

pub struct Recorder {
    recent: Ring<Event, RECENT>,
    signals: Ring<Event, SIGNALS>,
    progress: Ring<Progress, PROGRESS>,
    /// Most recent selected object-producing boundary for a bounded set of
    /// live object pointers. This is deliberately not a general allocation
    /// trace: it exists to retain collection/coder provenance after the main
    /// event ring wraps.
    provenance: [Provenance; PROVENANCE],
    value_origins: [ValueOrigin; VALUE_ORIGINS],
    boundary_details: Ring<BoundaryDetail, BOUNDARY_DETAILS>,
    bundle_candidates: Ring<Text, BUNDLE_CANDIDATES>,
    bundle_directory_entries: Ring<Text, BUNDLE_DIRECTORY_ENTRIES>,
    collection_events: Ring<CollectionEvent, COLLECTION_EVENTS>,
    invocation_events: Ring<InvocationEvent, INVOCATION_EVENTS>,
    array_events: Ring<ArrayEvent, ARRAY_EVENTS>,
    watched_objects: [u32; WATCHED_OBJECTS],
    object_lifetime_events: Ring<ObjectLifetimeEvent, OBJECT_LIFETIME_EVENTS>,
    dictionary_origins: [DictionaryOrigin; DICTIONARY_ORIGINS],
    dictionary_origin_sequence: u64,
    defaults_lookup: DefaultsLookup,
    slow_capture: Option<SlowCapture>,
    pending_slow_capture: Option<PendingSlowCapture>,
    /// Largest individual present gap. Unlike the sustained-slow capture,
    /// this preserves evidence for a one-off jank transition.
    worst_present_gap: Option<SlowCapture>,
    unresolved_relocations: Ring<UnresolvedRelocation, UNRESOLVED_RELOCATIONS>,
    string_searches: Ring<StringSearch, STRING_SEARCHES>,
    touch_events: Ring<TouchEvent, TOUCH_EVENTS>,
    selector_events: Ring<SelectorEvent, SELECTOR_EVENTS>,
    sequence: u64,
    api: u64,
    objc: u64,
    filtered: u64,
    pub run_loops: u64,
    pub presents: u64,
    /// Last observed main-loop source counts: timers, selectors, queues, units.
    pub sources: [usize; 4],
    yields: u64,
    started: Instant,
    next_progress: Instant,
    last_present: Option<Instant>,
    last_present_boundaries: u64,
    healthy_present_streak: u16,
    dumped: bool,
}
impl Recorder {
    pub fn new() -> Box<Self> {
        let now = Instant::now();
        Box::new(Self {
            recent: Ring::default(),
            signals: Ring::default(),
            progress: Ring::default(),
            provenance: [Provenance::default(); PROVENANCE],
            value_origins: [ValueOrigin::default(); VALUE_ORIGINS],
            boundary_details: Ring::default(),
            bundle_candidates: Ring::default(),
            bundle_directory_entries: Ring::default(),
            collection_events: Ring::default(),
            invocation_events: Ring::default(),
            array_events: Ring::default(),
            watched_objects: [0; WATCHED_OBJECTS],
            object_lifetime_events: Ring::default(),
            dictionary_origins: [DictionaryOrigin::default(); DICTIONARY_ORIGINS],
            dictionary_origin_sequence: 0,
            defaults_lookup: DefaultsLookup::default(),
            slow_capture: None,
            pending_slow_capture: None,
            worst_present_gap: None,
            unresolved_relocations: Ring::default(),
            string_searches: Ring::default(),
            touch_events: Ring::default(),
            selector_events: Ring::default(),
            sequence: 0,
            api: 0,
            objc: 0,
            filtered: 0,
            run_loops: 0,
            presents: 0,
            sources: [0; 4],
            yields: 0,
            started: now,
            next_progress: now + Duration::from_secs(5),
            last_present: None,
            last_present_boundaries: 0,
            healthy_present_streak: 0,
            dumped: false,
        })
    }
    fn begin(&mut self, mut call: Call) -> Call {
        self.sequence += 1;
        call.sequence = self.sequence;
        let event = Event {
            sequence: self.sequence,
            call,
            result: None,
            note: "enter",
        };
        self.recent.push(event);
        if call.name.bytes() == b"dealloc" {
            self.forget_provenance(call.receiver);
        }
        if call.name.bytes() == b"raise" {
            self.signals.push(event);
        }
        call
    }
    fn finish(
        &mut self,
        call: Call,
        result: [u32; 2],
        object_result: bool,
        dictionary_result: bool,
    ) {
        if call.detail.bytes() == b"NSUserDefaults"
            && matches!(call.name.bytes(), b"objectForKey:" | b"dictionaryForKey:")
            && self.defaults_lookup.receiver == call.receiver
            && self.defaults_lookup.key == call.args[2]
            && self.defaults_lookup.result == result[0]
        {
            self.set_boundary_context(call.sequence, self.defaults_lookup.domain);
        }
        self.sequence += 1;
        let signal = (object_result && result[0] == 0)
            || (call.name.bytes() == b"_dlsym" && result[0] == 0)
            || (call.name.bytes() == b"_open" && result[0] == u32::MAX)
            || (call.name.bytes() == b"_fopen" && result[0] == 0);
        let event = Event {
            sequence: self.sequence,
            call,
            result: Some(result),
            note: if object_result && result[0] == 0 {
                "nil-object-return (may be normal)"
            } else if signal {
                "lookup/resource-failure"
            } else {
                "return (raw ABI)"
            },
        };
        self.recent.push(event);
        if object_result && result[0] != 0 {
            self.remember_value_origin(result[0], event);
        }
        if dictionary_result && Self::tracks_provenance(&call) {
            // A dictionary obtained from another tracked dictionary inherits
            // the source container's property-list mutability contract. This
            // keeps nested-container provenance available even when the
            // original creation record has rolled out of the fixed table.
            if matches!(call.name.bytes(), b"objectForKey:" | b"dictionaryForKey:") {
                if let Some(mut origin) = self.dictionary_origin_for(call.receiver).copied() {
                    self.dictionary_origin_sequence =
                        self.dictionary_origin_sequence.saturating_add(1);
                    origin.sequence = self.dictionary_origin_sequence;
                    origin.object = result[0];
                    self.remember_dictionary_origin(origin);
                }
            }
            self.remember_provenance(result[0], event);
        }
        if signal {
            self.signals.push(event);
        }
    }
    fn tracks_provenance(call: &Call) -> bool {
        let name = call.name.bytes();
        matches!(
            name,
            b"objectForKey:"
                | b"dictionaryForKey:"
                | b"decodeObjectForKey:"
                | b"unarchiveObjectWithData:"
                | b"propertyListFromData:mutabilityOption:format:errorDescription:"
                | b"mutableCopy"
                | b"mutableCopyWithZone:"
                | b"copy"
                | b"copyWithZone:"
        ) || (call
            .detail
            .bytes()
            .windows(b"Dictionary".len())
            .any(|window| window == b"Dictionary")
            && (name.starts_with(b"initWith") || name.starts_with(b"dictionaryWith")))
    }
    fn remember_value_origin(&mut self, value: u32, event: Event) {
        let bucket_count = VALUE_ORIGINS / VALUE_ORIGIN_WAYS;
        let bucket = ((value as usize).wrapping_mul(0x9e37_79b1) >> 4) % bucket_count;
        let range = bucket * VALUE_ORIGIN_WAYS..(bucket + 1) * VALUE_ORIGIN_WAYS;
        let slots = &mut self.value_origins[range];
        let index = slots
            .iter()
            .position(|slot| slot.value == value)
            .unwrap_or_else(|| {
                slots
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, slot)| slot.sequence)
                    .unwrap()
                    .0
            });
        slots[index] = ValueOrigin {
            sequence: event.sequence,
            value,
            kind: event.call.kind,
            name: ShortText::from_text(event.call.name),
            receiver: event.call.receiver,
            detail: ShortText::from_text(event.call.detail),
            pc: event.call.pc,
            lr: event.call.lr,
        };
    }
    fn value_origin_for(&self, value: u32) -> Option<&ValueOrigin> {
        if value == 0 {
            return None;
        }
        let bucket_count = VALUE_ORIGINS / VALUE_ORIGIN_WAYS;
        let bucket = ((value as usize).wrapping_mul(0x9e37_79b1) >> 4) % bucket_count;
        self.value_origins
            [bucket * VALUE_ORIGIN_WAYS..(bucket + 1) * VALUE_ORIGIN_WAYS]
            .iter()
            .find(|slot| slot.value == value)
    }
    fn remember_provenance(&mut self, object: u32, event: Event) {
        let detail = self
            .boundary_detail_for(event.call.sequence)
            .copied()
            .unwrap_or_default();
        let provenance = Provenance {
            object,
            event,
            context: detail.context,
            source_argument: ShortText::from_text(detail.arguments[0]),
        };
        if let Some(slot) = self
            .provenance
            .iter_mut()
            .find(|slot| slot.object == object)
        {
            *slot = provenance;
            return;
        }
        let slot = self
            .provenance
            .iter_mut()
            .min_by_key(|slot| slot.event.sequence)
            .unwrap();
        *slot = provenance;
    }
    fn remember_boundary_detail(&mut self, detail: BoundaryDetail) {
        self.boundary_details.push(detail);
    }
    fn boundary_detail_for(&self, call_sequence: u64) -> Option<&BoundaryDetail> {
        self.boundary_details
            .iter()
            .rev()
            .find(|detail| detail.call_sequence == call_sequence)
    }
    fn set_boundary_context(&mut self, call_sequence: u64, context: DefaultsDomain) {
        if let Some(detail) = self
            .boundary_details
            .items
            .iter_mut()
            .find(|detail| detail.call_sequence == call_sequence)
        {
            detail.context = context;
        }
    }
    fn forget_provenance(&mut self, object: u32) {
        if object == 0 {
            return;
        }
        if let Some(slot) = self
            .provenance
            .iter_mut()
            .find(|slot| slot.object == object)
        {
            *slot = Provenance::default();
        }
    }
    fn provenance_for(&self, object: u32) -> Option<&Provenance> {
        if object == 0 {
            return None;
        }
        self.provenance.iter().find(|slot| slot.object == object)
    }
    fn last_boundary_returning(&self, value: u32) -> Option<&Event> {
        if value == 0 {
            return None;
        }
        self.recent
            .iter()
            .rev()
            .find(|event| event.result.is_some_and(|result| result[0] == value))
    }
    fn watch_object(&mut self, object: u32) {
        if object == 0 || self.watched_objects.contains(&object) {
            return;
        }
        if let Some(slot) = self.watched_objects.iter_mut().find(|slot| **slot == 0) {
            *slot = object;
            return;
        }
        let index = self.object_lifetime_events.written as usize % WATCHED_OBJECTS;
        self.watched_objects[index] = object;
    }
    fn watches_object(&self, object: u32) -> bool {
        object != 0 && self.watched_objects.contains(&object)
    }
    fn remember_dictionary_origin(&mut self, origin: DictionaryOrigin) {
        if let Some(slot) = self
            .dictionary_origins
            .iter_mut()
            .find(|slot| slot.object == origin.object)
        {
            *slot = origin;
            return;
        }
        let slot = self
            .dictionary_origins
            .iter_mut()
            .min_by_key(|slot| slot.sequence)
            .unwrap();
        *slot = origin;
    }
    fn dictionary_origin_for(&self, object: u32) -> Option<&DictionaryOrigin> {
        if object == 0 {
            return None;
        }
        self.dictionary_origins
            .iter()
            .find(|origin| origin.object == object)
    }
    fn note_present(&mut self) {
        self.note_present_at(Instant::now());
    }
    fn note_present_at(&mut self, now: Instant) {
        self.presents += 1;
        let boundaries = self.api.saturating_add(self.objc);
        if let Some(previous) = self.last_present {
            let gap = now.saturating_duration_since(previous);
            if gap >= SLOW_PRESENT_INTERVAL
                && self
                    .worst_present_gap
                    .as_ref()
                    .is_none_or(|capture| gap.as_millis() > capture.gap_ms)
            {
                let mut recent = Ring::default();
                let skip = self.recent.iter().count().saturating_sub(SLOW_RECENT);
                for event in self.recent.iter().skip(skip) {
                    recent.push(*event);
                }
                self.worst_present_gap = Some(SlowCapture {
                    ms: now.duration_since(self.started).as_millis(),
                    gap_ms: gap.as_millis(),
                    boundary_delta: boundaries.saturating_sub(self.last_present_boundaries),
                    presents: self.presents,
                    had_healthy_baseline: self.healthy_present_streak >= HEALTHY_PRESENT_STREAK,
                    recent,
                });
            }
            if gap <= HEALTHY_PRESENT_INTERVAL {
                self.pending_slow_capture = None;
                self.healthy_present_streak = self.healthy_present_streak.saturating_add(1);
            } else {
                if self.slow_capture.is_none() && gap >= SLOW_PRESENT_INTERVAL {
                    if let Some(pending) = &mut self.pending_slow_capture {
                        pending.streak = pending.streak.saturating_add(1);
                        if pending.streak >= SLOW_PRESENT_STREAK {
                            self.slow_capture = self
                                .pending_slow_capture
                                .take()
                                .map(|pending| pending.capture);
                        }
                    } else if self.healthy_present_streak >= HEALTHY_PRESENT_STREAK
                        || self.presents > 1
                    {
                        let mut recent = Ring::default();
                        let skip = self.recent.iter().count().saturating_sub(SLOW_RECENT);
                        for event in self.recent.iter().skip(skip) {
                            recent.push(*event);
                        }
                        self.pending_slow_capture = Some(PendingSlowCapture {
                            capture: SlowCapture {
                                ms: now.duration_since(self.started).as_millis(),
                                gap_ms: gap.as_millis(),
                                boundary_delta: boundaries
                                    .saturating_sub(self.last_present_boundaries),
                                presents: self.presents,
                                had_healthy_baseline: self.healthy_present_streak
                                    >= HEALTHY_PRESENT_STREAK,
                                recent,
                            },
                            streak: 1,
                        });
                    }
                } else if gap < SLOW_PRESENT_INTERVAL {
                    self.pending_slow_capture = None;
                }
                self.healthy_present_streak = 0;
            }
        }
        self.last_present = Some(now);
        self.last_present_boundaries = boundaries;
    }

    /// Preserve a bounded summary of unresolved dyld relocations so a later
    /// crash report does not depend on the launch log still being available.
    pub fn note_unresolved_relocation(&mut self, image: &str, symbol: &str, addresses: &[u32]) {
        self.unresolved_relocations.push(UnresolvedRelocation {
            image: Text::from_bytes(image.as_bytes()),
            symbol: Text::from_bytes(symbol.as_bytes()),
            first_address: addresses.first().copied().unwrap_or(0),
            count: addresses.len(),
        });
    }
    pub fn checkpoint(&mut self, thread: usize, regs: &[u32], threads: usize, runnable: usize) {
        self.yields += 1;
        let now = Instant::now();
        if now < self.next_progress {
            return;
        }
        self.next_progress = now + Duration::from_secs(5);
        let sample = Progress {
            ms: now.duration_since(self.started).as_millis(),
            thread,
            pc: regs[15],
            lr: regs[14],
            threads,
            runnable,
            api: self.api,
            objc: self.objc,
            run_loops: self.run_loops,
            presents: self.presents,
            yields: self.yields,
            sources: self.sources,
        };
        self.progress.push(sample);
        // Persist limited evidence even when a host kill prevents a final dump.
        if self.progress.written <= 12 {
            Self::print_progress(&sample);
        }
    }
    fn print_progress(p: &Progress) {
        echo_no_panic!("flight progress ms={} tid={} pc={:#x} lr={:#x} thread-slots={} runnable={} api={} objc={} main-loops={} EAGL-returns={} yields={} main-sources[timer,selector,queue,unit]={:?}",
            p.ms, p.thread, p.pc, p.lr, p.threads, p.runnable, p.api, p.objc, p.run_loops, p.presents, p.yields, p.sources);
    }
    fn print_event(&self, e: &Event) {
        let c = &e.call;
        echo_no_panic!("flight #{} call={} tid={} {} {} {} pc={:#x} lr={:#x} sp={:#x} args={:08x?} stack={:08x?} receiver={:#x} isa={:#x} result={:08x?} text=\"{}\" mib={:?}",
            e.sequence, c.sequence, c.thread, c.kind, c.name, e.note, c.pc, c.lr, c.sp,
            c.args, &c.stack[..c.stack_len], c.receiver, c.isa, e.result, c.detail, &c.mib[..c.mib_len]);
        if let Some(detail) = self.boundary_detail_for(c.sequence) {
            echo_no_panic!(
                "flight #{} decoded context=\"{}\" arguments=[\"{}\", \"{}\"]",
                e.sequence,
                detail.context.as_str(),
                detail.arguments[0],
                detail.arguments[1]
            );
        }
    }
    pub fn dump(&mut self, reason: &str) {
        if self.dumped {
            return;
        }
        self.dumped = true;
        echo_no_panic!("flight recorder v1: {} elapsed-ms={} api={} objc={} filtered={} main-loops={} EAGL-returns={} (raw registers; LR may be inherited for host-internal calls)",
            reason, self.started.elapsed().as_millis(), self.api, self.objc, self.filtered, self.run_loops, self.presents);
        echo_no_panic!(
            "flight progress history: newest {PROGRESS}; total={}",
            self.progress.written
        );
        for p in self.progress.iter() {
            Self::print_progress(p);
        }
        let elapsed_ms = self.started.elapsed().as_millis();
        if elapsed_ms != 0 {
            let present_rate = self.presents as f64 * 1000.0 / elapsed_ms as f64;
            let loops_per_present = if self.presents == 0 {
                None
            } else {
                Some(self.run_loops as f64 / self.presents as f64)
            };
            echo_no_panic!(
                "flight absolute presentation cadence: elapsed-ms={} EAGL-returns={} rate-hz={:.3} main-loops={} loops-per-present={:?} (descriptive only; a low rate may be intentional)",
                elapsed_ms,
                self.presents,
                present_rate,
                self.run_loops,
                loops_per_present,
            );
        }
        if let Some(capture) = &self.slow_capture {
            echo_no_panic!("flight first sustained slow-present capture: mode={} elapsed-ms={} first-gap-ms={} boundaries-during-first-gap={} present={} (frozen after {} consecutive slow intervals; transition mode requires {} healthy intervals)",
                if capture.had_healthy_baseline { "healthy-to-slow" } else { "slow-from-start" },
                capture.ms, capture.gap_ms, capture.boundary_delta, capture.presents,
                SLOW_PRESENT_STREAK, HEALTHY_PRESENT_STREAK);
            echo_no_panic!(
                "flight slow-transition boundaries: newest {SLOW_RECENT}; total={}",
                capture.recent.written
            );
            for event in capture.recent.iter() {
                self.print_event(event);
            }
        } else {
            echo_no_panic!("flight first sustained slow-present capture: not observed");
        }
        if let Some(capture) = &self.worst_present_gap {
            echo_no_panic!("flight worst single present gap: elapsed-ms={} gap-ms={} boundaries-during-gap={} present={} healthy-baseline={} (bounded transition evidence; may be an intentional load)",
                capture.ms, capture.gap_ms, capture.boundary_delta, capture.presents, capture.had_healthy_baseline);
            echo_no_panic!(
                "flight worst-gap boundaries: newest {SLOW_RECENT}; total={}",
                capture.recent.written
            );
            for event in capture.recent.iter() {
                self.print_event(event);
            }
        } else {
            echo_no_panic!("flight worst single present gap: not observed");
        }
        echo_no_panic!(
            "flight unresolved relocation summaries: newest {UNRESOLVED_RELOCATIONS}; total={}",
            self.unresolved_relocations.written
        );
        for relocation in self.unresolved_relocations.iter() {
            echo_no_panic!("flight unresolved relocation image=\"{}\" symbol=\"{}\" first-address={:#x} count={}",
                relocation.image, relocation.symbol, relocation.first_address, relocation.count);
        }
        echo_no_panic!(
            "flight candidate signals: newest {SIGNALS}; total={} (not diagnoses)",
            self.signals.written
        );
        for e in self.signals.iter() {
            self.print_event(e);
        }
        echo_no_panic!(
            "flight failed bundle candidates: newest {BUNDLE_CANDIDATES}; total={} (attempted guest paths; each entry text-bounded)",
            self.bundle_candidates.written
        );
        for candidate in self.bundle_candidates.iter() {
            echo_no_panic!("flight bundle candidate miss path=\"{}\"", candidate);
        }
        echo_no_panic!(
            "flight bundle directory samples: newest {BUNDLE_DIRECTORY_ENTRIES}; total={} (bounded names from failed resource directories)",
            self.bundle_directory_entries.written
        );
        for entry in self.bundle_directory_entries.iter() {
            echo_no_panic!("flight bundle directory entry name=\"{}\"", entry);
        }
        echo_no_panic!("flight strstr searches: newest {STRING_SEARCHES}; total={} (80-byte prefixes; misses are not errors)", self.string_searches.written);
        for entry in self.string_searches.iter() {
            echo_no_panic!("flight strstr tid={} pc={:#x} lr={:#x} haystack={:#x} needle={:#x} result={:#x} text=\"{}\" pattern=\"{}\"", entry.thread, entry.pc, entry.lr, entry.haystack, entry.needle, entry.result, entry.text, entry.pattern);
        }
        echo_no_panic!(
            "flight touch lifecycle: newest {TOUCH_EVENTS}; total={} (bounded host-to-UIKit delivery; no input behavior changes)",
            self.touch_events.written
        );
        for event in self.touch_events.iter() {
            echo_no_panic!("flight touch #{} tid={} action={} touch={:#x} view={:#x} active={} point=({:.2},{:.2}) pc={:#x} lr={:#x}",
                event.sequence, event.thread, event.action, event.touch, event.view,
                event.active, event.x, event.y, event.pc, event.lr);
        }
        echo_no_panic!(
            "flight delayed-selector lifecycle: newest {SELECTOR_EVENTS}; total={} (bounded schedule/delivery evidence; no scheduling changes)",
            self.selector_events.written
        );
        for event in self.selector_events.iter() {
            echo_no_panic!("flight selector #{} tid={} action={} run-loop={:#x} target={:#x} selector=\"{}\" argument={:#x} delay-ms={} pc={:#x} lr={:#x}",
                event.sequence, event.thread, event.action, event.run_loop, event.target,
                event.selector, event.argument, event.delay_ms, event.pc, event.lr);
        }
        echo_no_panic!(
            "flight collection ownership events: newest {COLLECTION_EVENTS}; total={} (bounded observations, not proof of ownership correctness)",
            self.collection_events.written
        );
        for event in self.collection_events.iter() {
            echo_no_panic!("flight collection #{} tid={} action={} dictionary={:#x} key={:#x} value={:#x} value-class=\"{}\" raw-words={:08x?}/{} callbacks[key-retain={:#x},key-release={:#x},value-retain={:#x},value-release={:#x}]",
                event.sequence, event.thread, event.action, event.dictionary, event.key, event.value,
                event.value_class, event.raw_words, event.raw_word_count,
                event.key_retain, event.key_release, event.value_retain, event.value_release);
        }
        echo_no_panic!(
            "flight watched-object lifetime events: newest {OBJECT_LIFETIME_EVENTS}; total={} (bounded values observed in non-retaining dictionaries or recent array transfers)",
            self.object_lifetime_events.written
        );
        for event in self.object_lifetime_events.iter() {
            echo_no_panic!("flight lifetime #{} tid={} action={} object={:#x} class=\"{}\" refcount={}->{} context={:#x} pc={:#x} lr={:#x}",
                event.sequence, event.thread, event.action, event.object, event.class,
                event.before, event.after, event.context, event.pc, event.lr);
        }
        echo_no_panic!(
            "flight invocation lifecycle events: newest {INVOCATION_EVENTS}; total={} (bounded creation/configuration/invoke snapshots)",
            self.invocation_events.written
        );
        for event in self.invocation_events.iter() {
            echo_no_panic!("flight invocation #{} tid={} action={} invocation={:#x} target={:#x} target-class=\"{}\" selector={:#x} selector-name=\"{}\" argument-index={} argument-count={} set-mask={:#010x} pc={:#x} lr={:#x}",
                event.sequence, event.thread, event.action, event.invocation,
                event.target, event.target_class, event.selector, event.selector_name, event.argument_index,
                event.argument_count, event.set_mask, event.pc, event.lr);
        }
        echo_no_panic!(
            "flight array mutation events: newest {ARRAY_EVENTS}; total={} (bounded mutations plus invalid accesses; counts are before/after)",
            self.array_events.written
        );
        for event in self.array_events.iter() {
            echo_no_panic!("flight array #{} tid={} action={} array={:#x} array-class=\"{}\" index={} object={:#x} object-class=\"{}\" count={}->{} pc={:#x} lr={:#x} caller-bytes={:02x?}",
                event.sequence, event.thread, event.action, event.array,
                event.array_class, event.index, event.object, event.object_class,
                event.before, event.after,
                event.pc, event.lr,
                &event.caller_code[..usize::from(event.caller_code_len)]);
        }
        if let Some(last) = self
            .recent
            .iter()
            .rev()
            .find(|event| event.result.is_none())
        {
            if let Some(origin) = self
                .dictionary_origins
                .iter()
                .find(|origin| origin.object == last.call.receiver)
            {
                echo_no_panic!("flight receiver dictionary origin: receiver={:#x} mutable={} context={} option={} sequence={} creation-pc={:#x} creation-lr={:#x}",
                    origin.object, origin.mutable, origin.context, origin.option, origin.sequence,
                    origin.pc, origin.lr);
            } else {
                echo_no_panic!(
                    "flight receiver dictionary origin: no retained origin for receiver {:#x}",
                    last.call.receiver
                );
            }
            if let Some(origin) = self.provenance_for(last.call.receiver) {
                echo_no_panic!("flight receiver provenance: most recent tracked object-producing boundary for receiver {:#x}; this may be a transfer, not the original allocation", last.call.receiver);
                self.print_event(&origin.event);
                echo_no_panic!(
                    "flight receiver provenance decoded: context=\"{}\" source-argument=\"{}\"",
                    origin.context.as_str(),
                    origin.source_argument
                );
            } else {
                echo_no_panic!(
                    "flight receiver provenance: no retained tracked boundary for receiver {:#x}",
                    last.call.receiver
                );
            }
        }
        echo_no_panic!("flight recent boundaries: newest {RECENT}; total={} (older events overwritten; absent return does not prove a stall)", self.recent.written);
        for e in self.recent.iter() {
            self.print_event(e);
        }
        echo_no_panic!("flight recorder end");
    }

    /// Reuse the existing history only at failure time. These are observed
    /// call sites, not a reconstructed stack or proof of a bad value's origin.
    fn crash_callers(&self, thread: usize) -> [u32; CRASH_CALLERS] {
        let mut callers = [0; CRASH_CALLERS];
        let mut count = 0;
        for event in self.recent.iter().rev() {
            let call = event.call;
            if call.thread != thread || call.lr == 0 || callers.contains(&call.lr) {
                continue;
            }
            callers[count] = call.lr;
            count += 1;
            if count == CRASH_CALLERS {
                break;
            }
        }
        callers
    }
    #[cfg(test)]
    pub fn last_entry(&self) -> Option<(&[u8], &[u32])> {
        self.recent
            .iter()
            .filter(|e| e.result.is_none())
            .last()
            .map(|e| (e.call.name.bytes(), &e.call.mib[..e.call.mib_len]))
    }
}

fn read_word(mem: &Mem, addr: u32) -> Option<u32> {
    let bytes = mem.get_bytes_fallible(ConstVoidPtr::from_bits(addr), 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn describe_object(objc: &ObjC, mem: &Mem, address: u32, include_string: bool) -> Text {
    if address == 0 {
        return Text::from_bytes(b"nil");
    }
    let Some(isa) = read_word(mem, address) else {
        return Text::from_bytes(b"<invalid-object>");
    };
    let class_name = objc
        .try_get_class_name(id::from_bits(isa))
        .unwrap_or("<unknown-class>");
    if !include_string {
        return Text::from_bytes(class_name.as_bytes());
    }
    let mut string = [0; 80];
    let Some(written) = ns_string::copy_diagnostic_utf8(objc, id::from_bits(address), &mut string)
    else {
        return Text::from_bytes(class_name.as_bytes());
    };
    let mut result = Text::default();
    result.append(b"\"");
    result.append(&string[..written]);
    result.append(b"\" class=");
    result.append(class_name.as_bytes());
    result
}

impl Environment {
    pub fn diagnostic_string_search(&mut self, haystack: u32, needle: u32, result: u32) {
        let Some(recorder) = &mut self.diagnostics else {
            return;
        };
        recorder.string_searches.push(StringSearch {
            thread: self.current_thread,
            pc: self.cpu.regs()[15],
            lr: self.cpu.regs()[14],
            haystack,
            needle,
            result,
            text: Text::guest(&self.mem, haystack),
            pattern: Text::guest(&self.mem, needle),
        });
    }

    pub fn diagnostic_touch_event(
        &mut self,
        action: &'static str,
        touch: id,
        view: id,
        active: usize,
        x: f32,
        y: f32,
    ) {
        let Some(recorder) = &mut self.diagnostics else {
            return;
        };
        let regs = self.cpu.regs();
        recorder.touch_events.push(TouchEvent {
            sequence: recorder.touch_events.written.saturating_add(1),
            thread: self.current_thread,
            action,
            touch: touch.to_bits(),
            view: view.to_bits(),
            active,
            x,
            y,
            pc: regs[15],
            lr: regs[14],
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn diagnostic_selector_event(
        &mut self,
        action: &'static str,
        run_loop: id,
        target: id,
        selector_name: &str,
        argument: id,
        delay: Option<f64>,
    ) {
        let Some(recorder) = &mut self.diagnostics else {
            return;
        };
        let regs = self.cpu.regs();
        recorder.selector_events.push(SelectorEvent {
            sequence: recorder.selector_events.written.saturating_add(1),
            thread: self.current_thread,
            action,
            run_loop: run_loop.to_bits(),
            target: target.to_bits(),
            selector: Text::from_bytes(selector_name.as_bytes()),
            argument: argument.to_bits(),
            delay_ms: delay.map_or(-1, |seconds| (seconds * 1000.0) as i64),
            pc: regs[15],
            lr: regs[14],
        });
    }

    /// Observe existing guest/host boundaries without ownership changes.
    pub fn diagnostic_api_begin(&mut self, name: &'static str) -> Option<Call> {
        let recorder = self.diagnostics.as_mut()?;
        recorder.api += 1;
        if let Some(history) = &mut self.mem.object_history {
            let regs = self.cpu.regs();
            history.context(crate::mem::object_history::Context {
                thread: self.current_thread,
                pc: regs[15],
                lr: regs[14],
                api: name,
                ..Default::default()
            });
        }
        // ObjC is recorded at its decoded dispatch boundary. Count high-volume
        // graphics/allocator/memory primitives without erasing other history.
        if name.starts_with("_objc_msgSend")
            || name.starts_with("_gl")
            || matches!(
                name,
                "_malloc"
                    | "_free"
                    | "_calloc"
                    | "_realloc"
                    | "_memcpy"
                    | "_memset"
                    | "_memmove"
                    | "_strlen"
            )
        {
            recorder.filtered += 1;
            return None;
        }
        let regs = self.cpu.regs();
        let mut call = Call {
            thread: self.current_thread,
            kind: "API",
            name: Text::from_bytes(name.as_bytes()),
            pc: regs[15],
            lr: regs[14],
            sp: regs[13],
            args: regs[..4].try_into().unwrap(),
            ..Call::default()
        };
        for i in 0..call.stack.len() {
            let Some(addr) = call.sp.checked_add((i * 4) as u32) else {
                break;
            };
            let Some(word) = read_word(&self.mem, addr) else {
                break;
            };
            call.stack[i] = word;
            call.stack_len += 1;
        }
        if name == "_sysctl" {
            for i in 0..(regs[1] as usize).min(call.mib.len()) {
                let Some(addr) = regs[0].checked_add((i * 4) as u32) else {
                    break;
                };
                let Some(word) = read_word(&self.mem, addr) else {
                    break;
                };
                call.mib[i] = word;
                call.mib_len += 1;
            }
        } else if matches!(name, "_dlsym" | "_open" | "_fopen") {
            call.detail = Text::guest(&self.mem, regs[if name == "_dlsym" { 1 } else { 0 }]);
        }
        Some(recorder.begin(call))
    }
    pub fn diagnostic_objc_begin(
        &mut self,
        receiver: u32,
        selector: u32,
        host_internal: bool,
    ) -> Option<Call> {
        let recorder = self.diagnostics.as_mut()?;
        recorder.objc += 1;
        if let Some(history) = &mut self.mem.object_history {
            let regs = self.cpu.regs();
            history.context(crate::mem::object_history::Context {
                thread: self.current_thread,
                pc: regs[15],
                lr: regs[14],
                api: if host_internal { "ObjC(host)" } else { "ObjC" },
                receiver,
                selector,
            });
        }
        let name = Text::guest(&self.mem, selector);
        if matches!(
            name.bytes(),
            b"retain" | b"release" | b"autorelease" | b"retainCount"
        ) {
            recorder.filtered += 1;
            return None;
        }
        let regs = self.cpu.regs();
        let isa = read_word(&self.mem, receiver).unwrap_or(0);
        let class_name = self
            .objc
            .try_get_class_name(crate::objc::id::from_bits(isa))
            .unwrap_or("<unknown-class>");
        let call = recorder.begin(Call {
            thread: self.current_thread,
            kind: if host_internal { "ObjC(host)" } else { "ObjC" },
            name,
            receiver,
            isa,
            detail: Text::from_bytes(class_name.as_bytes()),
            pc: regs[15],
            lr: regs[14],
            sp: regs[13],
            args: regs[..4].try_into().unwrap(),
            ..Call::default()
        });
        let is_defaults_lookup = class_name == "NSUserDefaults"
            && matches!(name.bytes(), b"objectForKey:" | b"dictionaryForKey:");
        let is_immutable_dictionary_mutation =
            class_name == "_touchHLE_NSDictionary" && name.bytes() == b"setObject:forKey:";
        if is_defaults_lookup || is_immutable_dictionary_mutation {
            let arguments = if is_defaults_lookup {
                [
                    describe_object(&self.objc, &self.mem, regs[2], true),
                    Text::default(),
                ]
            } else {
                [
                    describe_object(&self.objc, &self.mem, regs[2], false),
                    describe_object(&self.objc, &self.mem, regs[3], true),
                ]
            };
            recorder.remember_boundary_detail(BoundaryDetail {
                call_sequence: call.sequence,
                arguments,
                ..BoundaryDetail::default()
            });
            if is_defaults_lookup {
                recorder.defaults_lookup = DefaultsLookup::default();
            }
        }
        Some(call)
    }
    /// Associate the host-side domain decision with the already-recorded
    /// NSUserDefaults boundary. No guest calls or ownership changes are made.
    pub fn diagnostic_user_defaults_lookup(
        &mut self,
        receiver: u32,
        key: u32,
        result: u32,
        domain: &'static str,
    ) {
        if let Some(recorder) = &mut self.diagnostics {
            recorder.defaults_lookup = DefaultsLookup {
                receiver,
                key,
                result,
                domain: DefaultsDomain::from_name(domain),
            };
        }
    }
    /// Retain the exact failed candidate without writing a per-lookup log or
    /// calling guest code. The fixed-size ring is emitted only with a normal
    /// flight-recorder dump.
    pub fn diagnostic_bundle_candidate_miss(&mut self, path: &str) {
        if self.diagnostics.is_none() {
            return;
        }
        let mut directory_entries = [Text::default(); 8];
        let mut directory_entry_count = 0;
        if let Some((parent, _)) = path.rsplit_once('/') {
            if let Ok(entries) = self.fs.enumerate(GuestPath::new(parent)) {
                for (entry, slot) in entries.zip(directory_entries.iter_mut()) {
                    *slot = Text::from_bytes(entry.as_bytes());
                    directory_entry_count += 1;
                }
            }
        }
        if let Some(recorder) = &mut self.diagnostics {
            recorder.bundle_candidates.push(Text::path_tail(path));
            for entry in &directory_entries[..directory_entry_count] {
                recorder.bundle_directory_entries.push(*entry);
            }
        }
    }

    /// Record the small amount of state needed to reconstruct how an
    /// NSInvocation was configured. Argument contents are deliberately not
    /// copied; the bit mask records only which slots were supplied.
    #[allow(clippy::too_many_arguments)]
    pub fn diagnostic_invocation_event(
        &mut self,
        action: &'static str,
        invocation: u32,
        target: u32,
        selector: u32,
        argument_index: i32,
        argument_count: u32,
        set_mask: u32,
    ) {
        let regs = self.cpu.regs();
        let selector_name = Text::guest(&self.mem, selector);
        let target_class = read_word(&self.mem, target)
            .and_then(|isa| self.objc.try_get_class_name(id::from_bits(isa)))
            .map(|name| Text::from_bytes(name.as_bytes()))
            .unwrap_or_default();
        if let Some(recorder) = &mut self.diagnostics {
            recorder.invocation_events.push(InvocationEvent {
                sequence: recorder.invocation_events.written.saturating_add(1),
                thread: self.current_thread,
                action,
                invocation,
                target,
                selector,
                selector_name,
                target_class,
                argument_index,
                argument_count,
                set_mask,
                pc: regs[15],
                lr: regs[14],
            });
        }
    }

    /// Record bounded mutable-array transitions and invalid accesses without
    /// changing collection behavior or retaining any objects.
    #[allow(clippy::too_many_arguments)]
    pub fn diagnostic_array_event(
        &mut self,
        action: &'static str,
        array: u32,
        index: u32,
        object: u32,
        before: u32,
        after: u32,
    ) {
        let regs = self.cpu.regs();
        let should_watch = object != 0 && matches!(action, "insert" | "add" | "replace");
        let refcount = should_watch
            .then(|| self.objc.refcount_for_diagnostics(id::from_bits(object)))
            .flatten();
        let class = if should_watch {
            ObjC::try_read_isa(id::from_bits(object), &self.mem)
                .and_then(|class| self.objc.try_get_class_name(class))
                .map(|name| Text::from_bytes(name.as_bytes()))
                .unwrap_or_else(|| Text::from_bytes(b"<invalid-at-array-transfer>"))
        } else {
            Text::default()
        };
        let array_class = ObjC::try_read_isa(id::from_bits(array), &self.mem)
            .and_then(|class| self.objc.try_get_class_name(class))
            .map(|name| Text::from_bytes(name.as_bytes()))
            .unwrap_or_default();
        let object_class = if object == 0 {
            Text::default()
        } else {
            ObjC::try_read_isa(id::from_bits(object), &self.mem)
                .and_then(|class| self.objc.try_get_class_name(class))
                .map(|name| Text::from_bytes(name.as_bytes()))
                .unwrap_or_else(|| Text::from_bytes(b"<invalid>"))
        };
        let mut caller_code = [0; 16];
        let caller_address = (regs[14] & !1).saturating_sub(8);
        let caller_code_len = self
            .mem
            .get_bytes_fallible(ConstVoidPtr::from_bits(caller_address), 16)
            .map(|bytes| {
                caller_code.copy_from_slice(bytes);
                bytes.len() as u8
            })
            .unwrap_or(0);
        if let Some(recorder) = &mut self.diagnostics {
            recorder.array_events.push(ArrayEvent {
                sequence: recorder.array_events.written.saturating_add(1),
                thread: self.current_thread,
                action,
                array,
                index,
                object,
                before,
                after,
                pc: regs[15],
                lr: regs[14],
                caller_code,
                caller_code_len,
                array_class,
                object_class,
            });
            // Preserve retain/release/deallocation provenance for only the
            // most recently transferred array values. The recorder does not
            // retain these guest objects or change collection behavior.
            if should_watch {
                recorder.watch_object(object);
                recorder.object_lifetime_events.push(ObjectLifetimeEvent {
                    sequence: recorder.object_lifetime_events.written.saturating_add(1),
                    thread: self.current_thread,
                    action: "observed-at-array-transfer",
                    object,
                    class,
                    before: refcount.unwrap_or(0),
                    after: refcount.unwrap_or(0),
                    pc: regs[15],
                    lr: regs[14],
                    context: 0,
                });
            }
        }
    }
    /// Record only dictionary ownership-relevant transitions. This is a
    /// bounded diagnostic stream, not a general retain/release trace.
    #[allow(clippy::too_many_arguments)]
    pub fn diagnostic_collection_event(
        &mut self,
        action: &'static str,
        dictionary: u32,
        key: u32,
        value: u32,
        callbacks: [u32; 4],
    ) {
        let refcount = self.objc.refcount_for_diagnostics(id::from_bits(value));
        let regs = self.cpu.regs();
        let (pc, lr) = (regs[15], regs[14]);
        let value_class = if value == 0 {
            Text::default()
        } else {
            ObjC::try_read_isa(id::from_bits(value), &self.mem)
                .and_then(|class| self.objc.try_get_class_name(class))
                .map(|name| Text::from_bytes(name.as_bytes()))
                .unwrap_or_else(|| Text::from_bytes(b"<not-a-live-object>"))
        };
        let mut raw_words = [0; 3];
        let raw_word_count = self
            .mem
            .get_bytes_fallible(ConstVoidPtr::from_bits(value), 12)
            .map(|bytes| {
                for (word, chunk) in raw_words.iter_mut().zip(bytes.chunks_exact(4)) {
                    *word = u32::from_le_bytes(chunk.try_into().unwrap());
                }
                (bytes.len() / 4) as u8
            })
            .unwrap_or(0);
        if let Some(recorder) = &mut self.diagnostics {
            let sequence = recorder.collection_events.written.saturating_add(1);
            recorder.collection_events.push(CollectionEvent {
                sequence,
                thread: self.current_thread,
                action,
                dictionary,
                key,
                value,
                key_retain: callbacks[0],
                key_release: callbacks[1],
                value_retain: callbacks[2],
                value_release: callbacks[3],
                value_class,
                raw_words,
                raw_word_count,
            });
            if action == "insert-callback-dictionary" && callbacks == [0; 4] && value != 0 {
                recorder.watch_object(value);
                recorder.object_lifetime_events.push(ObjectLifetimeEvent {
                    sequence: recorder.object_lifetime_events.written.saturating_add(1),
                    thread: self.current_thread,
                    action: "observed-in-non-retaining-dictionary",
                    object: value,
                    class: Text::default(),
                    before: refcount.unwrap_or(0),
                    after: refcount.unwrap_or(0),
                    pc,
                    lr,
                    context: 0,
                });
            }
        }
    }
    pub fn diagnostic_object_lifetime(
        &mut self,
        action: &'static str,
        object: id,
        before: u32,
        after: u32,
    ) {
        let address = object.to_bits();
        if let Some(history) = &mut self.mem.object_history {
            history.event(address, action, before, after);
        }
        if !self
            .diagnostics
            .as_ref()
            .is_some_and(|recorder| recorder.watches_object(address))
        {
            return;
        }
        let class = ObjC::try_read_isa(object, &self.mem)
            .and_then(|class| self.objc.try_get_class_name(class))
            .map(|name| Text::from_bytes(name.as_bytes()))
            .unwrap_or_else(|| Text::from_bytes(b"<deallocated-or-invalid>"));
        let regs = self.cpu.regs();
        if let Some(recorder) = &mut self.diagnostics {
            recorder.object_lifetime_events.push(ObjectLifetimeEvent {
                sequence: recorder.object_lifetime_events.written.saturating_add(1),
                thread: self.current_thread,
                action,
                object: address,
                class,
                before,
                after,
                pc: regs[15],
                lr: regs[14],
                context: 0,
            });
        }
    }

    /// Correlate a watched object's final autorelease-pool consumption with
    /// the exact pool that releases it. Objects are neither retained nor
    /// messaged by this diagnostic.
    pub fn diagnostic_autorelease_pool_release(&mut self, pool: id, object: id) {
        let address = object.to_bits();
        if !self
            .diagnostics
            .as_ref()
            .is_some_and(|recorder| recorder.watches_object(address))
        {
            return;
        }
        let class = ObjC::try_read_isa(object, &self.mem)
            .and_then(|class| self.objc.try_get_class_name(class))
            .map(|name| Text::from_bytes(name.as_bytes()))
            .unwrap_or_else(|| Text::from_bytes(b"<deallocated-or-invalid>"));
        let refcount = self.objc.refcount_for_diagnostics(object).unwrap_or(0);
        let regs = self.cpu.regs();
        if let Some(recorder) = &mut self.diagnostics {
            recorder.object_lifetime_events.push(ObjectLifetimeEvent {
                sequence: recorder.object_lifetime_events.written.saturating_add(1),
                thread: self.current_thread,
                action: "autorelease-pool-release",
                object: address,
                class,
                before: refcount,
                after: refcount.saturating_sub(1),
                pc: regs[15],
                lr: regs[14],
                context: pool.to_bits(),
            });
        }
    }
    pub fn diagnostic_dictionary_origin(
        &mut self,
        object: id,
        mutable: bool,
        context: &'static str,
        option: u32,
    ) {
        let regs = self.cpu.regs();
        if let Some(recorder) = &mut self.diagnostics {
            recorder.dictionary_origin_sequence =
                recorder.dictionary_origin_sequence.saturating_add(1);
            let sequence = recorder.dictionary_origin_sequence;
            recorder.remember_dictionary_origin(DictionaryOrigin {
                sequence,
                object: object.to_bits(),
                mutable,
                context,
                option,
                pc: regs[15],
                lr: regs[14],
            });
        }
    }
    pub fn diagnostic_end(&mut self, call: Option<Call>, object_result: bool) {
        if let Some(call) = call {
            let regs = self.cpu.regs();
            let result = [regs[0], regs[1]];
            let dictionary_result = object_result
                && result[0] != 0
                && Recorder::tracks_provenance(&call)
                && read_word(&self.mem, result[0])
                    .and_then(|isa| {
                        self.objc
                            .try_get_class_name(crate::objc::id::from_bits(isa))
                    })
                    .is_some_and(|class_name| class_name.contains("Dictionary"));
            let did_present = call.name.bytes() == b"presentRenderbuffer:" && regs[0] != 0;
            if let Some(recorder) = &mut self.diagnostics {
                recorder.finish(call, result, object_result, dictionary_result);
                if did_present {
                    recorder.note_present();
                }
            }
        }
    }
    pub fn diagnostic_dump(&mut self, reason: &str) {
        if let Some(recorder) = &mut self.diagnostics {
            if !recorder.dumped {
                if let Some(history) = &self.mem.object_history {
                    echo_no_panic!("object history enabled: 8192 address slots, 8 recent events/address, evictions={}; heap/ObjC boundaries only, no guest-store tracing", history.evictions);
                    history.dump_coverage();
                    if matches!(reason, "panic" | "scheduler-panic") {
                        // Prefer the final boundary's consumed values over
                        // arbitrary registers. At most eight retained records.
                        let mut candidates = [0; 64];
                        let mut n = 0;
                        for event in recorder.recent.iter().rev().take(8) {
                            for address in [
                                event.call.receiver,
                                event.call.args[2],
                                event.call.args[3],
                            ] {
                                if address != 0 && !candidates[..n].contains(&address) {
                                    candidates[n] = address;
                                    n += 1;
                                }
                            }
                        }
                        for &address in &self.cpu.regs()[..13] {
                            if address != 0 && !candidates[..n].contains(&address) {
                                candidates[n] = address;
                                n += 1;
                            }
                        }
                        let consumed_count = n;
                        // A returned pointer can be the producer that matters,
                        // even when its lifetime record was evicted before the
                        // final invalid consumption. Preserve it as a lookup
                        // candidate without expanding the recorder or tracing
                        // guest stores.
                        for event in recorder.recent.iter().rev().take(8) {
                            if let Some(result) = event.result {
                                let address = result[0];
                                if address != 0 && !candidates[..n].contains(&address) {
                                    candidates[n] = address;
                                    n += 1;
                                }
                            }
                        }
                        let mut producer_hints = 0;
                        for &address in &candidates[..consumed_count] {
                            if let Some(origin) = recorder.value_origin_for(address) {
                                echo_no_panic!(
                                    concat!(
                                        "flight retained value producer address={:#010x} ",
                                        "sequence={} kind={} name={} receiver={:#010x} ",
                                        "class={} pc={:#010x} lr={:#010x}; compact bounded ",
                                        "last-return correlation, not proof of allocation origin"
                                    ),
                                    address,
                                    origin.sequence,
                                    origin.kind,
                                    origin.name,
                                    origin.receiver,
                                    origin.detail,
                                    origin.pc,
                                    origin.lr
                                );
                                producer_hints += 1;
                                if producer_hints == 8 {
                                    break;
                                }
                                continue;
                            }
                            let Some(event) = recorder.last_boundary_returning(address) else {
                                continue;
                            };
                            echo_no_panic!(
                                concat!(
                                    "flight value producer hint address={:#010x} ",
                                    "sequence={} kind={} name={} receiver={:#010x} ",
                                    "class={} pc={:#010x} lr={:#010x}; bounded ",
                                    "boundary correlation, not proof of allocation origin"
                                ),
                                address,
                                event.sequence,
                                event.call.kind,
                                event.call.name,
                                event.call.receiver,
                                event.call.detail,
                                event.call.pc,
                                event.call.lr
                            );
                            producer_hints += 1;
                            if producer_hints == 8 {
                                break;
                            }
                        }
                        let mut shown = 0;
                        for &address in &candidates[..n] {
                            if history.dump(address) {
                                echo_no_panic!("object current address={address:#010x} readable-isa={:?} registered={} (readable bytes do not establish a live object)", read_word(&self.mem, address), self.objc.get_host_object(id::from_bits(address)).is_some());
                                shown += 1;
                                if shown == 8 {
                                    break;
                                }
                            }
                        }
                        echo_no_panic!("object history lookup: {shown} retained candidate records shown; absent/evicted records cannot exclude earlier lifetime faults");
                        let mut heap_shown = 0;
                        for &address in &candidates[..consumed_count] {
                            if history.dump_heap(address) {
                                let bytes = self
                                    .mem
                                    .get_bytes_fallible(ConstVoidPtr::from_bits(address), 16);
                                echo_no_panic!("heap candidate current bytes at {address:#010x}: {bytes:02x?}; registered-object={}", self.objc.get_host_object(id::from_bits(address)).is_some());
                                heap_shown += 1;
                                if heap_shown == 8 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if !recorder.dumped && matches!(reason, "panic" | "scheduler-panic") {
                let pc = self.cpu.regs()[crate::cpu::Cpu::PC];
                let instruction_address = pc & !1;
                if let Some(binary) = self.bins.iter().find(|binary| {
                    binary.text_segment_base.is_some_and(|base| {
                        base <= instruction_address && instruction_address < binary.last_segment_end
                    })
                }) {
                    let start = instruction_address
                        .saturating_sub(16)
                        .max(binary.text_segment_base.unwrap());
                    let end = instruction_address.saturating_add(16).min(binary.last_segment_end);
                    let bytes = self
                        .mem
                        .get_bytes_fallible(ConstVoidPtr::from_bits(start), end - start);
                    echo_no_panic!(
                        "flight fault instruction context: pc={pc:#010x} image={} bytes at {start:#010x}: {bytes:02x?}; raw bounded code, not a decoded root cause",
                        binary.name
                    );
                } else {
                    echo_no_panic!(
                        "flight fault instruction context: pc={pc:#010x} image=<unknown>; code unavailable"
                    );
                }
                let callers = recorder.crash_callers(self.current_thread);
                echo_no_panic!("flight crash caller code: up to {CRASH_CALLERS} distinct recent LR values on the failing thread; raw bytes, not a call stack; Thumb mode comes from LR bit 0");
                for lr in callers.into_iter().filter(|&lr| lr != 0) {
                    let address = lr & !1;
                    let Some(binary) = self.bins.iter().find(|binary| {
                        binary.text_segment_base.is_some_and(|base| {
                            base <= address && address < binary.last_segment_end
                        })
                    }) else {
                        continue;
                    };
                    // Bounded to the loaded image; never follow guest pointers
                    // or call guest methods to obtain this diagnostic data.
                    let start = address
                        .saturating_sub(96)
                        .max(binary.text_segment_base.unwrap());
                    let end = address.saturating_add(16).min(binary.last_segment_end);
                    let ptr = ConstVoidPtr::from_bits(start);
                    if let Some(bytes) = self.mem.get_bytes_fallible(ptr, end - start) {
                        echo_no_panic!("flight caller LR={lr:#010x} image={} bytes at {start:#010x}: {bytes:02x?}", binary.name);
                    } else {
                        echo_no_panic!("flight caller LR={lr:#010x}: code unavailable");
                    }
                }
            }
            recorder.dump(reason);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_history_correlates_a_consumed_value_with_its_last_return() {
        let mut recorder = Recorder::new();
        let call = recorder.begin(Call {
            kind: "ObjC",
            name: Text::from_bytes(b"objectForKey:"),
            receiver: 0x1111,
            ..Call::default()
        });
        recorder.finish(call, [0x3041_3fd0, 0], true, false);
        let producer = recorder.last_boundary_returning(0x3041_3fd0).unwrap();
        assert_eq!(producer.call.name.bytes(), b"objectForKey:");
        assert_eq!(producer.call.receiver, 0x1111);
        assert!(recorder.last_boundary_returning(0).is_none());
        assert!(recorder.last_boundary_returning(0xdead_beef).is_none());
        let retained = recorder.value_origin_for(0x3041_3fd0).unwrap();
        assert_eq!(retained.name.bytes(), b"objectForKey:");
        for i in 0..(RECENT * 4) {
            let call = recorder.begin(Call {
                kind: "ObjC",
                name: Text::from_bytes(b"description"),
                receiver: i as u32 + 1,
                ..Call::default()
            });
            recorder.finish(call, [0x5000_0000 + i as u32, 0], true, false);
        }
        assert!(recorder.last_boundary_returning(0x3041_3fd0).is_none());
        assert_eq!(
            recorder.value_origin_for(0x3041_3fd0).unwrap().receiver,
            0x1111
        );
    }

    #[test]
    fn diagnostic_missing_selector_preserves_failure_and_captures_caller() {
        use crate::mach_o::MachO;
        use crate::objc::{id, msg_class, msg_send, release};
        for enabled in [false, true] {
            let mut env = Environment::new_fake_for_test();
            let env = &mut env;
            let object: id = msg_class![env; NSObject new];
            let selector = env
                .objc
                .register_host_selector("diagnosticMissingSelector".to_owned(), &mut env.mem);
            let code = env.mem.alloc(128);
            // Synthetic A32 caller: mov r3,r6; bl; bx lr. The diagnostic
            // captures bytes; it does not execute or decode this fixture.
            let bytes = env.mem.bytes_at_mut(code.cast(), 128);
            bytes.fill(0);
            bytes[88..100].copy_from_slice(&[
                0x06, 0x30, 0xa0, 0xe1, 0x00, 0x00, 0x00, 0xeb, 0x1e, 0xff, 0x2f, 0xe1,
            ]);
            env.bins.push(MachO {
                name: "synthetic-caller".into(),
                dynamic_libraries: vec![],
                sections: vec![],
                exported_symbols: Default::default(),
                external_relocations: vec![],
                entry_point_pc: None,
                text_segment_base: Some(code.to_bits()),
                last_segment_end: code.to_bits() + 128,
            });
            env.cpu.regs_mut()[crate::cpu::Cpu::LR] = code.to_bits() + 96;
            env.diagnostics = enabled.then(Recorder::new);
            let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _: id = msg_send(env, (object, selector));
            }))
            .expect_err("missing selector must still panic");
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap();
            assert!(message.contains("does not respond to selector \"diagnosticMissingSelector\""));
            let regs = *env.cpu.regs();
            let cpsr = env.cpu.cpsr();
            let before = env.mem.bytes_at(code.cast(), 128).to_vec();
            if let Some(recorder) = &env.diagnostics {
                assert_eq!(
                    recorder.crash_callers(env.current_thread)[0],
                    code.to_bits() + 96
                );
            }
            env.diagnostic_dump("panic");
            env.diagnostic_dump("panic"); // Repeated catch must not duplicate output.
            assert_eq!(*env.cpu.regs(), regs);
            assert_eq!(env.cpu.cpsr(), cpsr);
            assert_eq!(env.mem.bytes_at(code.cast(), 128), before);
            release(env, object);
        }
    }

    #[test]
    fn crash_callers_are_bounded_distinct_and_thread_local_after_wrap() {
        let mut recorder = Recorder::new();
        for i in 1..200 {
            for thread in [0, 1] {
                let call = Call {
                    thread,
                    lr: i * 4 + 1,
                    ..Call::default()
                };
                // Entry/return and nested host calls may repeat an LR.
                recorder.recent.push(Event {
                    call,
                    ..Event::default()
                });
                recorder.recent.push(Event {
                    call,
                    ..Event::default()
                });
            }
        }
        recorder.recent.push(Event {
            call: Call {
                thread: 1,
                lr: 0x1234,
                ..Call::default()
            },
            ..Event::default()
        });
        let expected = [797, 793, 789, 785, 781, 777, 773, 769];
        assert_eq!(recorder.crash_callers(0), expected);
        assert_eq!(recorder.crash_callers(2), [0; CRASH_CALLERS]);
        assert_eq!(recorder.crash_callers(0), expected); // Read-only snapshot.
    }

    #[test]
    fn ring_retains_order_after_wrap() {
        let mut ring = Ring::<u32, 3>::default();
        for i in 0..10 {
            ring.push(i);
        }
        assert_eq!(ring.iter().copied().collect::<Vec<_>>(), [7, 8, 9]);
    }
    #[test]
    fn invalid_pointers_and_interleaved_returns_are_observational() {
        // A real Mach-O load reserves its null segment; the bare test
        // Environment intentionally does not load a binary.
        let mut env = Environment::new_fake_for_test_with_null_segment(0x1000);
        env.cpu.regs_mut()[0..4].copy_from_slice(&[0, 6, 0, 4]);
        let before = *env.cpu.regs();
        assert!(env.diagnostic_api_begin("_sysctl").is_none());
        env.diagnostics = Some(Recorder::new());
        let a = env.diagnostic_api_begin("_sysctl").unwrap();
        env.current_thread = 2;
        let b = env.diagnostic_objc_begin(0, u32::MAX, false).unwrap();
        env.diagnostic_end(Some(b), true);
        env.current_thread = 0;
        env.diagnostic_end(Some(a), false);
        assert_eq!(before, *env.cpu.regs());
        let recorder = env.diagnostics.as_ref().unwrap();
        let events: Vec<_> = recorder.recent.iter().collect();
        assert_eq!(events.len(), 4);
        assert_eq!(events[2].call.thread, 2);
        assert_eq!(events[3].call.sequence, a.sequence);
        assert_eq!(recorder.signals.written, 1);
        assert_eq!(a.mib_len, 0);
    }
    #[test]
    fn objc_dispatch_distinguishes_nil_objects_from_numeric_zero() {
        use crate::objc::{id, msg, msg_class, nil, release};
        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let array: id = msg_class![env; NSArray new];
        env.diagnostics = Some(Recorder::new());
        let count: u32 = msg![env; array count];
        assert_eq!(count, 0);
        assert_eq!(env.diagnostics.as_ref().unwrap().signals.written, 0);
        let last: id = msg![env; array lastObject];
        assert_eq!(last, nil);
        let recorder = env.diagnostics.as_ref().unwrap();
        assert_eq!(recorder.signals.written, 1);
        assert_eq!(
            recorder.signals.iter().last().unwrap().call.name.bytes(),
            b"lastObject"
        );
        release(env, array);
    }
    #[test]
    fn progress_and_output_are_bounded() {
        let mut recorder = Recorder::new();
        for _ in 0..40 {
            recorder.next_progress = Instant::now();
            recorder.checkpoint(0, &[0; 16], 1, 1);
        }
        assert_eq!(recorder.progress.iter().count(), PROGRESS);
        assert_eq!(recorder.progress.written, 40);
        assert!(std::mem::size_of::<Recorder>() < MAX_RECORDER_BYTES);
        let text = Text::from_bytes(b"name\nforged line");
        assert_eq!(format!("{text}"), "name\\x0aforged line");
    }
    #[test]
    fn collection_provenance_survives_history_wrap_and_deallocation_clears_it() {
        let mut recorder = Recorder::new();
        let source = Call {
            sequence: 7,
            kind: "ObjC",
            name: Text::from_bytes(b"objectForKey:"),
            receiver: 0x1000,
            detail: Text::from_bytes(b"NSUserDefaults"),
            ..Call::default()
        };
        recorder.finish(source, [0x2000, 0], true, true);
        for sequence in 0..(RECENT * 3) {
            recorder.recent.push(Event {
                sequence: sequence as u64 + 100,
                ..Event::default()
            });
        }
        let provenance = recorder.provenance_for(0x2000).unwrap();
        assert_eq!(provenance.event.call.name.bytes(), b"objectForKey:");
        assert_eq!(provenance.event.call.receiver, 0x1000);

        recorder.begin(Call {
            name: Text::from_bytes(b"dealloc"),
            receiver: 0x2000,
            ..Call::default()
        });
        assert!(recorder.provenance_for(0x2000).is_none());
    }
    #[test]
    fn collection_provenance_ignores_non_dictionary_results_and_keeps_latest_transfer() {
        let mut recorder = Recorder::new();
        let mut source = Call {
            kind: "ObjC",
            name: Text::from_bytes(b"objectForKey:"),
            receiver: 0x1000,
            detail: Text::from_bytes(b"NSUserDefaults"),
            ..Call::default()
        };
        recorder.finish(source, [0x2000, 0], true, false);
        assert!(recorder.provenance_for(0x2000).is_none());

        recorder.finish(source, [0x3000, 0], true, true);
        source.name = Text::from_bytes(b"mutableCopy");
        source.receiver = 0x3000;
        source.detail = Text::from_bytes(b"_touchHLE_NSDictionary");
        recorder.finish(source, [0x3000, 0], true, true);
        let provenance = recorder.provenance_for(0x3000).unwrap();
        assert_eq!(provenance.event.call.name.bytes(), b"mutableCopy");
        assert_eq!(provenance.event.call.receiver, 0x3000);
    }

    #[test]
    fn nested_dictionary_lookup_inherits_property_list_origin() {
        let mut recorder = Recorder::new();
        recorder.dictionary_origin_sequence = 1;
        recorder.remember_dictionary_origin(DictionaryOrigin {
            sequence: 1,
            object: 0x1000,
            mutable: false,
            context: "property-list",
            option: 0,
            ..DictionaryOrigin::default()
        });
        recorder.finish(
            Call {
                name: Text::from_bytes(b"objectForKey:"),
                receiver: 0x1000,
                detail: Text::from_bytes(b"_touchHLE_NSDictionary"),
                ..Call::default()
            },
            [0x2000, 0],
            true,
            true,
        );

        let inherited = recorder.dictionary_origin_for(0x2000).unwrap();
        assert!(!inherited.mutable);
        assert_eq!(inherited.context, "property-list");
        assert_eq!(inherited.option, 0);
        assert!(inherited.sequence > 1);
    }

    #[test]
    fn decoded_defaults_domain_and_immutable_mutation_are_bounded() {
        use crate::frameworks::foundation::ns_string::from_rust_string;
        use crate::objc::{id, msg_class};

        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let defaults: id = msg_class![env; NSUserDefaults alloc];
        let preference_key = from_rust_string(env, "settings/items".to_owned());
        let mutation_key = from_rust_string(env, "playerchar/gramps".to_owned());
        let value = from_rust_string(env, "owned".to_owned());
        let result: id = msg_class![env; NSDictionary new];
        env.diagnostics = Some(Recorder::new());

        let object_for_key = env.mem.alloc_and_write_cstr(b"objectForKey:").to_bits();
        env.cpu.regs_mut()[0..4].copy_from_slice(&[
            defaults.to_bits(),
            object_for_key,
            preference_key.to_bits(),
            0,
        ]);
        let call = env
            .diagnostic_objc_begin(defaults.to_bits(), object_for_key, false)
            .unwrap();
        env.diagnostic_user_defaults_lookup(
            defaults.to_bits(),
            preference_key.to_bits(),
            result.to_bits(),
            "application",
        );
        env.cpu.regs_mut()[0] = result.to_bits();
        env.diagnostic_end(Some(call), true);

        let provenance = env
            .diagnostics
            .as_ref()
            .unwrap()
            .provenance_for(result.to_bits())
            .unwrap();
        assert_eq!(provenance.context, DefaultsDomain::Application);
        assert!(provenance
            .source_argument
            .bytes()
            .starts_with(b"\"settings/items\""));

        let set_object = env.mem.alloc_and_write_cstr(b"setObject:forKey:").to_bits();
        env.cpu.regs_mut()[0..4].copy_from_slice(&[
            result.to_bits(),
            set_object,
            value.to_bits(),
            mutation_key.to_bits(),
        ]);
        let call = env
            .diagnostic_objc_begin(result.to_bits(), set_object, false)
            .unwrap();
        let detail = env
            .diagnostics
            .as_ref()
            .unwrap()
            .boundary_detail_for(call.sequence)
            .unwrap();
        assert!(detail.arguments[0]
            .bytes()
            .windows(b"NSString".len())
            .any(|window| window == b"NSString"));
        assert!(detail.arguments[1]
            .bytes()
            .starts_with(b"\"playerchar/gramps\""));
        assert!(std::mem::size_of::<Recorder>() < MAX_RECORDER_BYTES);
    }
    #[test]
    fn first_slow_present_freezes_the_transition_history() {
        let mut recorder = Recorder::new();
        let start = recorder.started;
        recorder.note_present_at(start);
        for frame in 1..=HEALTHY_PRESENT_STREAK {
            recorder.api += 10;
            recorder.recent.push(Event {
                sequence: frame as u64,
                note: "synthetic",
                ..Event::default()
            });
            recorder.note_present_at(start + Duration::from_millis(u64::from(frame) * 16));
        }
        recorder.api += 1_000;
        let healthy_end = start + Duration::from_millis(u64::from(HEALTHY_PRESENT_STREAK) * 16);
        recorder.note_present_at(healthy_end + Duration::from_millis(250));
        assert!(recorder.slow_capture.is_none());
        recorder.note_present_at(healthy_end + Duration::from_millis(500));
        assert!(recorder.slow_capture.is_none());
        recorder.note_present_at(healthy_end + Duration::from_millis(750));

        let capture = recorder.slow_capture.as_ref().unwrap();
        assert_eq!(capture.gap_ms, 250);
        assert_eq!(capture.boundary_delta, 1_000);
        assert_eq!(
            capture.recent.iter().count(),
            SLOW_RECENT.min(HEALTHY_PRESENT_STREAK as usize)
        );
        assert!(capture.had_healthy_baseline);
    }

    #[test]
    fn string_search_capture_is_bounded_and_reads_without_guest_messages() {
        let mut env = Environment::new_fake_for_test();
        env.diagnostics = Some(Recorder::new());
        let haystack = env.mem.alloc_and_write_cstr(&[b'x'; 100]);
        let needle = env.mem.alloc_and_write_cstr(b".xml");
        let registers = *env.cpu.regs();
        for index in 0..100 {
            env.diagnostic_string_search(
                haystack.to_bits(),
                needle.to_bits(),
                if index % 2 == 0 { 0 } else { haystack.to_bits() + 4 },
            );
        }
        assert_eq!(*env.cpu.regs(), registers);
        let recorder = env.diagnostics.as_ref().unwrap();
        assert_eq!(
            recorder.string_searches.iter().count(),
            STRING_SEARCHES
        );
        let last = recorder.string_searches.iter().last().unwrap();
        assert_eq!(last.text.bytes(), &[b'x'; 80]);
        assert_eq!(last.pattern.bytes(), b".xml");
        assert_eq!(last.result, haystack.to_bits() + 4);
        assert_eq!(recorder.objc, 0);
        assert!(std::mem::size_of::<Recorder>() < MAX_RECORDER_BYTES);
    }

    #[test]
    fn touch_selector_and_raw_collection_evidence_are_bounded_and_observational() {
        let mut env = Environment::new_fake_for_test_with_thread();
        env.diagnostics = Some(Recorder::new());
        let raw = env.mem.alloc(12);
        env.mem.bytes_at_mut(raw.cast(), 12).copy_from_slice(&[
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
        ]);
        for index in 0..(TOUCH_EVENTS * 2) {
            env.diagnostic_touch_event(
                "began-delivered",
                id::from_bits(0x3000 + index as u32),
                id::from_bits(0x4000),
                1,
                index as f32,
                2.0,
            );
        }
        for index in 0..(SELECTOR_EVENTS * 2) {
            env.diagnostic_selector_event(
                "scheduled",
                id::from_bits(0x5000),
                id::from_bits(0x6000),
                "callback:",
                id::from_bits(index as u32),
                Some(0.25),
            );
        }
        env.diagnostic_collection_event(
            "insert-callback-dictionary",
            0x7000,
            0x7100,
            raw.to_bits(),
            [0; 4],
        );
        let recorder = env.diagnostics.as_ref().unwrap();
        assert_eq!(recorder.touch_events.iter().count(), TOUCH_EVENTS);
        assert_eq!(recorder.selector_events.iter().count(), SELECTOR_EVENTS);
        let selector = recorder.selector_events.iter().last().unwrap();
        assert_eq!(selector.selector.bytes(), b"callback:");
        assert_eq!(selector.delay_ms, 250);
        let collection = recorder.collection_events.iter().last().unwrap();
        assert_eq!(collection.raw_word_count, 3);
        assert_eq!(collection.raw_words, [0x4433_2211, 0x8877_6655, 0xccbb_aa99]);
        assert!(std::mem::size_of::<Recorder>() < MAX_RECORDER_BYTES);
    }

    #[test]
    fn sustained_slow_presents_from_start_freeze_once() {
        let mut recorder = Recorder::new();
        let start = recorder.started;
        recorder.note_present_at(start);
        recorder.recent.push(Event {
            sequence: 1,
            note: "slow-from-start",
            ..Event::default()
        });
        recorder.api = 12;

        recorder.note_present_at(start + Duration::from_millis(250));
        assert!(recorder.slow_capture.is_none());
        recorder.note_present_at(start + Duration::from_millis(500));
        assert!(recorder.slow_capture.is_none());
        recorder.note_present_at(start + Duration::from_millis(750));

        let capture = recorder.slow_capture.as_ref().unwrap();
        assert!(!capture.had_healthy_baseline);
        assert_eq!(capture.gap_ms, 250);
        assert_eq!(capture.boundary_delta, 12);
        assert_eq!(capture.recent.iter().count(), 1);

        recorder.api = 100;
        recorder.note_present_at(start + Duration::from_millis(1_000));
        assert_eq!(recorder.slow_capture.as_ref().unwrap().boundary_delta, 12);
    }
}
