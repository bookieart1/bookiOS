/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded observations, never an authority for object or allocator validity.
//! Keep tombstones without quarantining storage or retaining guest objects.

// Four times the original address coverage while retaining the same fixed
// four-way lookup cost. This is still bounded and does not add new event
// classes; it reduces loss of the exact stale address in long game sessions.
const SETS: usize = 2048;
const WAYS: usize = 4;
const EVENTS: usize = 8;
const RESOURCE_WATCHES: usize = 64;
const HEAP_EVENTS: usize = 256;

#[derive(Clone, Copy, Debug, Default)]
struct HeapEvent {
    address: u32,
    size: u32,
    event: Event,
}

/// Most recently observed boundary, not a reconstructed allocation stack.
/// A host-internal operation may inherit a nested or earlier guest boundary.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Context {
    pub thread: usize,
    pub pc: u32,
    pub lr: u32,
    pub api: &'static str,
    pub receiver: u32,
    pub selector: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Event {
    pub sequence: u64,
    pub generation: u64,
    pub action: &'static str,
    pub before: u32,
    pub after: u32,
    pub context: Context,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Origin {
    pub event: Event,
    pub class: u32,
    pub size: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Record {
    pub address: u32,
    pub origin: Origin,
    pub previous_origin: Origin,
    pub removed: Event,
    pub freed: Event,
    events: [Event; EVENTS],
    written: usize,
}

impl Record {
    fn push(&mut self, event: Event) {
        self.events[self.written % EVENTS] = event;
        self.written = self.written.wrapping_add(1);
    }

    pub fn events(&self) -> impl Iterator<Item = &Event> {
        let count = self.written.min(EVENTS);
        let start = self.written.saturating_sub(count);
        (start..self.written).map(|i| &self.events[i % EVENTS])
    }
}

pub(crate) struct ObjectHistory {
    records: Box<[Record]>,
    // Up to 8 configured addresses, then dynamically allocated resource names,
    // sharing a hard cap of 64 pinned records. No guest retain,
    // no quarantine, and no eviction by unrelated allocation traffic.
    resource_watches: std::collections::HashMap<u32, Record>,
    resource_watch_overflow: u64,
    heap_events: Box<[HeapEvent]>,
    heap_written: usize,
    context: Context,
    sequence: u64,
    pub evictions: u64,
}

impl ObjectHistory {
    pub fn new() -> Self {
        Self {
            records: vec![Record::default(); SETS * WAYS].into_boxed_slice(),
            resource_watches: std::collections::HashMap::with_capacity(RESOURCE_WATCHES),
            resource_watch_overflow: 0,
            heap_events: vec![HeapEvent::default(); HEAP_EVENTS].into_boxed_slice(),
            heap_written: 0,
            context: Context::default(),
            sequence: 0,
            evictions: 0,
        }
    }

    fn set(address: u32) -> usize {
        // Mix aligned allocator addresses; do not use their low bits alone.
        let word = address >> 4;
        (word.wrapping_mul(0x9e3779b9) as usize & (SETS - 1)) * WAYS
    }

    fn index(&self, address: u32) -> Option<usize> {
        let start = Self::set(address);
        (start..start + WAYS).find(|&i| self.records[i].address == address && address != 0)
    }

    pub fn get(&self, address: u32) -> Option<&Record> {
        self.resource_watches
            .get(&address)
            .or_else(|| self.index(address).map(|i| &self.records[i]))
    }

    pub fn watch_resource(&mut self, address: u32) {
        if self.resource_watches.contains_key(&address) {
            return;
        }
        let Some(record) = self.get(address).copied() else {
            return; // Static strings and unknown addresses have no ObjC origin here.
        };
        if self.resource_watches.len() == RESOURCE_WATCHES {
            self.resource_watch_overflow = self.resource_watch_overflow.saturating_add(1);
            return;
        }
        self.resource_watches.insert(address, record);
        self.event(address, "resource-name-watch-start", 0, 0);
    }

    pub fn watch_address(&mut self, address: u32) {
        if address == 0
            || self.resource_watches.contains_key(&address)
            || self.resource_watches.len() == RESOURCE_WATCHES
        {
            return;
        }
        let record = self.get(address).copied().unwrap_or(Record {
            address,
            ..Default::default()
        });
        self.resource_watches.insert(address, record);
        self.event(address, "configured-watch-start", 0, 0);
    }

    pub fn heap_event(&mut self, address: u32, size: u32, action: &'static str) {
        self.sequence += 1;
        self.heap_events[self.heap_written % HEAP_EVENTS] = HeapEvent {
            address,
            size,
            event: Event {
                sequence: self.sequence,
                action,
                context: self.context,
                ..Event::default()
            },
        };
        self.heap_written = self.heap_written.wrapping_add(1);
        if action == "heap-allocated" {
            if let Some(record) = self.resource_watches.get_mut(&address) {
                record.previous_origin = record.origin;
                record.origin = Origin {
                    event: Event {
                        sequence: self.sequence,
                        generation: self.sequence,
                        action,
                        context: self.context,
                        ..Default::default()
                    },
                    class: 0, // Raw heap allocation; not an Objective-C object.
                    size,
                };
                record.push(record.origin.event);
            }
        }
    }

    fn recent_heap(&self) -> impl DoubleEndedIterator<Item = &HeapEvent> {
        let count = self.heap_written.min(HEAP_EVENTS);
        let start = self.heap_written.saturating_sub(count);
        (start..self.heap_written).map(|i| &self.heap_events[i % HEAP_EVENTS])
    }

    pub fn dump_heap(&self, address: u32) -> bool {
        let mut shown = 0;
        for item in self
            .recent_heap()
            .rev()
            .filter(|item| {
                address != 0
                    && address >= item.address
                    && u64::from(address) < u64::from(item.address) + u64::from(item.size)
            })
            .take(8)
        {
            echo_no_panic!("heap candidate={address:#010x} base={:#010x} size={:#x} (recent allocation boundary, not proof of Objective-C identity)", item.address, item.size);
            Self::print_event(item.event);
            shown += 1;
        }
        shown != 0
    }

    pub fn dump_coverage(&self) {
        echo_no_panic!("object persistent watches: {}/{} pinned addresses (configured or resource names); rejected-at-capacity={}; raw heap history newest {HEAP_EVENTS} of {} events; absence remains inconclusive", self.resource_watches.len(), RESOURCE_WATCHES, self.resource_watch_overflow, self.heap_written);
    }

    pub fn context(&mut self, context: Context) {
        self.context = context;
    }

    pub fn allocated(&mut self, address: u32, class: u32, size: u32) {
        if address == 0 {
            return;
        }
        let index = self.index(address).unwrap_or_else(|| {
            let start = Self::set(address);
            let i = (start..start + WAYS)
                .min_by_key(|&i| self.records[i].origin.event.sequence)
                .unwrap();
            if self.records[i].address != 0 {
                self.evictions += 1;
            }
            self.records[i] = Record::default();
            i
        });
        self.sequence += 1;
        if let Some(pinned) = self.resource_watches.get(&address) {
            self.records[index] = *pinned;
        }
        let record = &mut self.records[index];
        record.address = address;
        // ObjC registration promotes the immediately preceding raw allocation;
        // Preserve the older class when the watched address is reused.
        if record.origin.event.action != "heap-allocated" {
            record.previous_origin = record.origin;
        }
        record.origin = Origin {
            event: Event {
                sequence: self.sequence,
                generation: self.sequence,
                action: "object-allocated",
                context: self.context,
                ..Event::default()
            },
            class,
            size,
        };
        // Keep previous free/removal stamps; their generations distinguish them
        // from operations on the new object at the same guest address.
        record.push(record.origin.event);
        if let Some(pinned) = self.resource_watches.get_mut(&address) {
            *pinned = *record;
        }
    }

    pub fn event(&mut self, address: u32, action: &'static str, before: u32, after: u32) {
        let Some(record) = self.get(address) else {
            return;
        };
        let generation = record.origin.event.generation;
        self.sequence += 1;
        let event = Event {
            sequence: self.sequence,
            generation,
            action,
            before,
            after,
            context: self.context,
        };
        let apply = |record: &mut Record| {
            match action {
                "host-object-removed" => record.removed = event,
                "heap-free-completed" | "vm-free-overlap" => record.freed = event,
                _ => (),
            }
            record.push(event);
        };
        if let Some(index) = self.index(address) {
            apply(&mut self.records[index]);
        }
        if let Some(record) = self.resource_watches.get_mut(&address) {
            apply(record);
        }
    }

    /// Rare VM teardown: bounded scan, including records inside the freed
    /// range.
    pub fn vm_freed(&mut self, address: u32, size: u32) {
        self.heap_event(address, size, "vm-free-range");
        let end = u64::from(address) + u64::from(size);
        for index in 0..self.records.len() {
            let record = self.records[index];
            if record.address != 0
                && u64::from(record.address) < end
                && u64::from(record.address) + u64::from(record.origin.size) > u64::from(address)
            {
                self.event(record.address, "vm-free-overlap", 0, 0);
            }
        }
        // Pinned records may no longer exist in the ordinary table.
        let addresses: Vec<_> = self.resource_watches.keys().copied().collect();
        for candidate in addresses {
            if self.index(candidate).is_some() {
                continue;
            }
            let record = self.resource_watches[&candidate];
            if u64::from(candidate) < end
                && u64::from(candidate) + u64::from(record.origin.size) > u64::from(address)
            {
                self.event(candidate, "vm-free-overlap", 0, 0);
            }
        }
    }

    pub fn dump(&self, address: u32) -> bool {
        let Some(record) = self.get(address) else {
            return false;
        };
        echo_no_panic!("object history address={address:#010x}; evictions={} (absence is inconclusive); contexts are last observed boundaries, not stacks", self.evictions);
        for origin in [record.previous_origin, record.origin] {
            if origin.event.sequence != 0 {
                echo_no_panic!("object origin generation={} class={:#010x} size={:#x}", origin.event.generation, origin.class, origin.size);
                Self::print_event(origin.event);
            }
        }
        // Preserve these even after a long retain/release tail overwrites them.
        for event in [record.removed, record.freed] {
            if event.sequence != 0 {
                Self::print_event(event);
            }
        }
        for &event in record.events() {
            Self::print_event(event);
        }
        true
    }

    fn print_event(event: Event) {
        let c = event.context;
        echo_no_panic!("object event #{} gen={} {} values={}->{} thread={} pc={:#010x} lr={:#010x} boundary={} receiver={:#010x} selector={:#010x}", event.sequence, event.generation, event.action, event.before, event.after, c.thread, c.pc, c.lr, c.api, c.receiver, c.selector);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_watch_survives_eviction_free_reuse_and_vm_teardown() {
        let mut history = ObjectHistory::new();
        history.allocated(0x1000, 0x2000, 32);
        history.watch_resource(0x1000);
        let generation = history.get(0x1000).unwrap().origin.event.generation;
        for i in 1..50 {
            history.allocated(0x1000 + i * 0x8000, 0x3000, 32);
        }
        assert!(history.index(0x1000).is_none());
        history.event(0x1000, "host-object-removed", 0, 0);
        history.event(0x1000, "heap-free-completed", 0, 32);
        assert_eq!(history.get(0x1000).unwrap().removed.generation, generation);
        history.allocated(0x1000, 0x4000, 48);
        assert_eq!(
            history.get(0x1000).unwrap().previous_origin.event.generation,
            generation
        );
        history.vm_freed(0x1010, 16);
        assert_eq!(history.get(0x1000).unwrap().freed.action, "vm-free-overlap");
        for i in 1..=100 {
            let address = 0x1000 + i * 0x8000;
            history.allocated(address, 0x5000, 4);
            history.watch_resource(address);
        }
        assert_eq!(history.resource_watches.len(), RESOURCE_WATCHES);
        assert!(history.resource_watch_overflow > 0);
        assert_eq!(history.get(0x1000).unwrap().origin.class, 0x4000);
    }

    #[test]
    fn raw_heap_capture_includes_non_objects_and_is_bounded() {
        let mut history = ObjectHistory::new();
        history.context(Context {
            lr: 0xb43b,
            api: "_malloc",
            ..Default::default()
        });
        for i in 0..1000 {
            history.heap_event(0x1000 + i * 16, 12, "heap-allocated");
        }
        assert_eq!(history.recent_heap().count(), HEAP_EVENTS);
        assert!(history.get(0x1000 + 999 * 16).is_none());
        let item = history.recent_heap().last().unwrap();
        assert_eq!(item.size, 12);
        assert_eq!(item.event.context.lr, 0xb43b);
        assert!(history.dump_heap(item.address + 8));
        assert!(!history.dump_heap(0));
    }

    #[test]
    fn configured_watch_starts_before_raw_and_object_allocations() {
        let mut history = ObjectHistory::new();
        history.watch_address(0x1230);
        history.heap_event(0x1230, 12, "heap-allocated");
        assert_eq!(history.get(0x1230).unwrap().origin.class, 0);
        assert_eq!(history.get(0x1230).unwrap().origin.size, 12);
        history.allocated(0x1230, 0x8000, 12);
        history.event(0x1230, "release", 1, 0);
        history.event(0x1230, "host-object-removed", 0, 0);
        history.event(0x1230, "heap-free-completed", 0, 12);
        for i in 1..1000 {
            history.heap_event(0x5000 + i * 16, 12, "heap-allocated");
            history.allocated(0x1230 + i * 0x8000, 0x9000, 12);
        }
        let record = history.get(0x1230).unwrap();
        assert_eq!(record.origin.class, 0x8000);
        assert_eq!(record.removed.action, "host-object-removed");
        assert_eq!(record.freed.action, "heap-free-completed");
        assert!(history.index(0x1230).is_none());
        history.heap_event(0x1230, 24, "heap-allocated");
        history.allocated(0x1230, 0xa000, 24);
        let reused = history.get(0x1230).unwrap();
        assert_eq!(reused.previous_origin.class, 0x8000);
        assert_eq!(reused.origin.class, 0xa000);
        assert_ne!(reused.origin.event.generation, reused.freed.generation);
    }

    #[test]
    fn object_history_distinguishes_free_removal_reuse_and_generations() {
        let mut history = ObjectHistory::new();
        history.context(Context {
            thread: 3,
            lr: 0x1235,
            ..Context::default()
        });
        history.allocated(0x1000, 0x2000, 32);
        let first = history.get(0x1000).unwrap().origin.event.generation;
        history.event(0x1000, "heap-free-completed", 0, 0);
        assert_eq!(history.get(0x1000).unwrap().removed.sequence, 0);
        history.event(0x1000, "heap-address-reused", 0, 32);
        history.allocated(0x1000, 0x3000, 32);
        history.event(0x1000, "host-object-removed", 0, 0);
        let r = history.get(0x1000).unwrap();
        assert_eq!(r.previous_origin.event.generation, first);
        assert_eq!(r.freed.generation, first);
        assert_ne!(r.removed.generation, first);
        assert_eq!(r.origin.event.context.thread, 3);
        assert_eq!(r.origin.event.context.lr, 0x1235);
        for _ in 0..100 {
            history.event(0x1000, "retain", 1, 2);
        }
        let r = history.get(0x1000).unwrap();
        assert_eq!(r.events().count(), EVENTS);
        assert_eq!(r.origin.class, 0x3000);
        assert_eq!(r.freed.generation, first);
    }

    #[test]
    fn object_history_is_bounded_and_reports_eviction_and_vm_overlap() {
        let mut history = ObjectHistory::new();
        assert!(std::mem::size_of::<Record>() * SETS * WAYS < 8 * 1024 * 1024);
        assert!(
            std::mem::size_of::<Record>() * RESOURCE_WATCHES
                + std::mem::size_of::<HeapEvent>() * HEAP_EVENTS
                < 256 * 1024
        );
        for i in 1..20_000 {
            history.allocated(i * 16, 0x800000, 32);
        }
        assert_eq!(history.records.len(), SETS * WAYS);
        assert!(history.evictions > 0);
        let address = 19_999 * 16;
        history.vm_freed(address + 16, 4096);
        assert_eq!(history.get(address).unwrap().freed.action, "vm-free-overlap");
        assert!(history.get(0).is_none());
    }
}
