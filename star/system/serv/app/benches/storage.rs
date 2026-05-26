use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dashmap::DashMap;

use constant_time_utils::bytes::Long;

const ENTRY_COUNTS: &[usize] = &[21_800];
const STORAGE_TABLE_TITLE: &str = "DashMap<[u8; 32], ()> live allocated bytes";

struct CountingAllocator;

static MEASURE_ACTIVE: AtomicBool = AtomicBool::new(false);
static MEASURED_DELTA_BYTES: AtomicIsize = AtomicIsize::new(0);

thread_local! {
    static MEASURE_THIS_THREAD: Cell<bool> = const { Cell::new(false) };
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

// Tracks requested allocation sizes from Rust's allocator API.
// Measured as before/after deltas around map construction.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Delegates to the system allocator with the provided layout.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && should_count_this_allocation() {
            MEASURED_DELTA_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Delegates to the system allocator with the provided layout.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && should_count_this_allocation() {
            MEASURED_DELTA_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if should_count_this_allocation() {
            MEASURED_DELTA_BYTES.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
        // SAFETY: `ptr`/`layout` were returned by the allocator API.
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: Delegates to the system allocator with the provided arguments.
        let new_ptr = unsafe { System.realloc(ptr, old_layout, new_size) };
        if !new_ptr.is_null() && should_count_this_allocation() {
            match new_size.cmp(&old_layout.size()) {
                std::cmp::Ordering::Greater => {
                    MEASURED_DELTA_BYTES
                        .fetch_add((new_size - old_layout.size()) as isize, Ordering::Relaxed);
                }
                std::cmp::Ordering::Less => {
                    MEASURED_DELTA_BYTES
                        .fetch_sub((old_layout.size() - new_size) as isize, Ordering::Relaxed);
                }
                std::cmp::Ordering::Equal => {}
            }
        }
        new_ptr
    }
}

fn token_from_counter(counter: u64) -> Long {
    let mut token = [0u8; 32];
    token[0..8].copy_from_slice(&counter.to_be_bytes());
    token
}

fn build_dashmap(entries: usize) -> DashMap<Long, ()> {
    let map = DashMap::new();

    for idx in 0..entries {
        map.insert(token_from_counter(idx as u64), ());
    }

    map
}

fn should_count_this_allocation() -> bool {
    MEASURE_ACTIVE.load(Ordering::Relaxed) && MEASURE_THIS_THREAD.with(|flag| flag.get())
}

fn measured_delta_bytes<F, R>(f: F) -> (R, usize)
where
    F: FnOnce() -> R,
{
    MEASURED_DELTA_BYTES.store(0, Ordering::Relaxed);
    MEASURE_THIS_THREAD.with(|flag| flag.set(true));
    MEASURE_ACTIVE.store(true, Ordering::Relaxed);

    let result = f();

    MEASURE_ACTIVE.store(false, Ordering::Relaxed);
    MEASURE_THIS_THREAD.with(|flag| flag.set(false));

    let delta = MEASURED_DELTA_BYTES.load(Ordering::Relaxed);
    let delta_bytes = if delta > 0 { delta as usize } else { 0 };
    (result, delta_bytes)
}

fn bytes_per_entry(bytes: usize, entries: usize) -> f64 {
    if entries == 0 {
        0.0
    } else {
        bytes as f64 / entries as f64
    }
}

fn print_storage_header() {
    println!();
    println!("{STORAGE_TABLE_TITLE}");
    println!(
        "{:>12} {:>12} {:>16} {:>14}",
        "entries", "capacity", "alloc_delta", "bytes/entry"
    );
}

fn print_storage_row(entries: usize, capacity: usize, bytes: usize) {
    println!(
        "{:>12} {:>12} {:>16} {:>14.2}",
        entries,
        capacity,
        bytes,
        bytes_per_entry(bytes, entries)
    );
}

fn bench_storage(_c: &mut Criterion) {
    // Warm up one build to reduce one-time init noise.
    let warmup = build_dashmap(32);
    black_box(&warmup);
    drop(warmup);

    print_storage_header();

    for entries in ENTRY_COUNTS {
        let ((map, capacity), alloc_delta) = measured_delta_bytes(|| {
            let map = build_dashmap(*entries);
            let capacity = map.capacity();
            (map, capacity)
        });

        print_storage_row(*entries, capacity, alloc_delta);
        black_box(&map);
        drop(map);
    }

    println!();
}

criterion_group!(benches, bench_storage);
criterion_main!(benches);
