/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded observations, never an authority for object or allocator validity.
//! Keep tombstones without quarantining storage or retaining guest objects.

const SETS: usize = 512;
const WAYS: usize = 4;
const EVENTS: usize = 8;

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
    context: Context,
    sequence: u64,
    pub evictions: u64,
}

impl ObjectHistory {
    pub fn new() -> Self {
        Self {
            records: vec![Record::default(); SETS * WAYS].into_boxed_slice(),
            context: Context::default(),
            sequence: 0,
            evictions: 0,
        }
    }

    fn set(address: u32) -> usize {
        // Mix aligned allocator addresses; do not use their low bits alone.
        let word = address >> 4;
        (word.wrapping_mul(0x9e3779b9) >> 23) as usize * WAYS
    }

    fn index(&self, address: u32) -> Option<usize> {
        let start = Self::set(address);
        (start..start + WAYS).find(|&i| self.records[i].address == address && address != 0)
    }

    pub fn get(&self, address: u32) -> Option<&Record> {
        self.index(address).map(|i| &self.records[i])
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
        let record = &mut self.records[index];
        record.address = address;
        record.previous_origin = record.origin;
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
    }

    pub fn event(&mut self, address: u32, action: &'static str, before: u32, after: u32) {
        let Some(index) = self.index(address) else {
            return;
        };
        self.sequence += 1;
        let record = &mut self.records[index];
        let event = Event {
            sequence: self.sequence,
            generation: record.origin.event.generation,
            action,
            before,
            after,
            context: self.context,
        };
        match action {
            "host-object-removed" => record.removed = event,
            "heap-free-completed" | "vm-free-overlap" => record.freed = event,
            _ => (),
        }
        record.push(event);
    }

    /// Rare VM teardown: bounded scan, including records inside the freed range.
    pub fn vm_freed(&mut self, address: u32, size: u32) {
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
        assert!(std::mem::size_of::<Record>() * SETS * WAYS < 2 * 1024 * 1024);
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
