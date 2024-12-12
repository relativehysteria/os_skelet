//! Physical memory manager for the bootloader

use core::alloc::{ GlobalAlloc, Layout };
use crate::rangeset::{ RangeSet, Range };
use crate::spinlock::SpinLock;

/// All physical memory which is available for use by the bootloader and the
/// kernel. This memory IS ASSUMED to be used by both at the same time.
pub static FREE_MEMORY: SpinLock<Option<RangeSet>> = SpinLock::new(None);

/// Initialize the global memory allocator using `memory` as the physical memory
/// backlog.
pub fn init(memory: RangeSet) {
    // If the memory has been already initialized, don't reinitialize it
    if FREE_MEMORY.lock().is_some() { return; }

    // Initialize the memory
    let mut free_mem = FREE_MEMORY.lock();
    *free_mem = Some(memory);
}

#[alloc_error_handler]
/// Handler for allocation error, likely OOMs;
/// simply panic, notifying that we can't satisfy the allocation.
fn alloc_error(_layout: Layout) -> ! {
    panic!("Allocation error!");
}

#[global_allocator]
/// Global allocator for the bootloader; this just uses physical memory as a
/// backlog and __doesn't__ handle fragmentation. Only memory that won't have to
/// be freed between soft reboots should be allocated to prevent fragmentation.
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

/// Dummy structure that implements the [`GlobalAlloc`] trait
struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get access to the physical memory, allocate some bytes and return
        // the pointer
        let mut phys_mem = FREE_MEMORY.lock();
        phys_mem.as_mut().and_then(|x| {
            x.allocate(layout.size(), layout.align()).ok()?
        }).unwrap_or(0) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Get access to the physical memory rangeset and try to insert a new
        // range into it. If the pointer was allocated by [`alloc()`], it should
        // be correct. Here's the classical `free()` safety message:
        // ---------------------------------------------
        // If the pointer was not allocated by [`alloc()`], it can 'free up'
        // 1) ranges that can't be satisfied by the backing physical memory
        // 2) ranges that don't belong to the caller
        let mut phys_mem = FREE_MEMORY.lock();
        let ptr = ptr as usize;
        phys_mem.as_mut().and_then(|x| {
            let end = ptr.checked_add(layout.size().checked_sub(1)?)?;
            x.insert(Range::new(ptr, end).unwrap())
                .expect("Couldn't create a free memory range during dealloc");
            Some(())
        }).expect("Cannot free memory without initialized memory manager.");
    }
}
