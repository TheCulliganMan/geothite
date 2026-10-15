//! Location-test diagnostics: distinguish live Rust data from WASM high-water.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;
static STACKS_ENABLED: AtomicBool = AtomicBool::new(false);
static STACK_SIZES: [AtomicUsize; 64] = [const { AtomicUsize::new(0) }; 64];
// Diagnostic-only bridge: the browser supplies Wasm call stacks; Rust owns
// allocation accounting and bounds sampling to one stack per large layout.
#[wasm_bindgen(inline_js = "export function crystalAllocationStack(size) { (globalThis.__crystalAllocationStacks ??= []).push({size, stack: new Error('Rust allocation').stack}); }")]
extern "C" {
    fn crystalAllocationStack(size: usize);
}
pub(crate) fn enable_stack_sampling() { STACKS_ENABLED.store(true, Ordering::Relaxed); }
fn sample_stack(size: usize) {
    if size < 1024*1024 || !STACKS_ENABLED.load(Ordering::Relaxed) { return; }
    for slot in &STACK_SIZES {
        let existing = slot.load(Ordering::Relaxed);
        if existing == size { return; }
        if existing == 0 && slot.compare_exchange(0, size, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            crystalAllocationStack(size);
            return;
        }
    }
}
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
const BUCKETS: usize = 16384;
static SIZES: [AtomicUsize; BUCKETS] = [const { AtomicUsize::new(0) }; BUCKETS];
static COUNTS: [AtomicUsize; BUCKETS] = [const { AtomicUsize::new(0) }; BUCKETS];
static OMITTED: AtomicUsize = AtomicUsize::new(0);
fn record(size: usize, allocate: bool) {
    if size < 8192 { return; }
    let start = (size.wrapping_mul(0x9e3779b1) ^ (size >> 16)) % BUCKETS;
    for step in 0..32 {
        let slot = (start + step) % BUCKETS;
        let key = SIZES[slot].load(Ordering::Relaxed);
        if key == size || (allocate && key == 0 &&
            SIZES[slot].compare_exchange(0,size,Ordering::Relaxed,Ordering::Relaxed).is_ok()) {
            if allocate { COUNTS[slot].fetch_add(1,Ordering::Relaxed); }
            else { let _ = COUNTS[slot].fetch_update(Ordering::Relaxed,Ordering::Relaxed,|n|Some(n.saturating_sub(1))); }
            return;
        }
        if !allocate && key == 0 { return; }
    }
    if allocate { OMITTED.fetch_add(1,Ordering::Relaxed); }
}
struct CountingAllocator;
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
fn added(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(live, Ordering::Relaxed);
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
}
// Delegation preserves System's layout, initialization and ownership contract.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() { added(layout.size()); record(layout.size(),true); sample_stack(layout.size()); }
        ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() { added(layout.size()); record(layout.size(),true); sample_stack(layout.size()); }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(layout.size(),false);
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, bytes: usize) -> *mut u8 {
        let ptr = unsafe { System.realloc(ptr, layout, bytes) };
        if !ptr.is_null() {
            record(layout.size(),false); record(bytes,true);
            sample_stack(bytes);
            if bytes >= layout.size() { added(bytes-layout.size()); }
            else { LIVE.fetch_sub(layout.size()-bytes, Ordering::Relaxed); }
        }
        ptr
    }
}
pub(crate) fn snapshot() -> String {
    let mut large: Vec<_> = (0..BUCKETS).filter_map(|i| {
        let size=SIZES[i].load(Ordering::Relaxed); let count=COUNTS[i].load(Ordering::Relaxed);
        (count>0).then_some((size*count,size,count))
    }).collect();
    large.sort_unstable_by(|a,b|b.0.cmp(&a.0)); large.truncate(20);
    serde_json::json!({"rustLiveBytes":LIVE.load(Ordering::Relaxed),
        "rustPeakBytes":PEAK.load(Ordering::Relaxed),
        "allocations":ALLOCATIONS.load(Ordering::Relaxed),
        "largeLayouts":large.iter().map(|&(bytes,size,count)|serde_json::json!({"bytes":bytes,"size":size,"count":count})).collect::<Vec<_>>(),
        "unclassifiedLargeAllocations":OMITTED.load(Ordering::Relaxed),
        "accounting":"System allocations only; excludes WASM static data, free heap space and GPU/driver memory"}).to_string()
}
