//! Custom multi-threaded allocator
//!
//! There are up to 4 heaps, each up to 64kiB in size. When allocating, the code will pick the appropriate heap

use core::alloc::{AllocError, Allocator};

use cortex_m_rt::heap_start;
use lock_api::MutexGuard;

use crate::sync::{
    atomic::{Atomic, AtomicBool},
    Mutex, RP2040Mutex,
};

#[derive(Debug)]
pub struct Heap {
    start_addr: usize,
    size: usize,
}

impl Heap {
    const unsafe fn new(start_addr: usize, size: usize) -> Self {
        Self { start_addr, size }
    }
    fn allocate(
        &mut self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, AllocError> {
        todo!()
    }

    unsafe fn deallocate(&mut self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        todo!()
    }
}

#[derive(Debug)]
pub struct HeapRoot {
    heaps: [Mutex<Heap>; 4],
    cur_heap: Atomic<u8>,
}

impl HeapRoot {
    /// Create a new HeapRoot
    ///
    /// This function must only be run once!
    pub unsafe fn new_unsafe() -> Self {
        Self {
            heaps: [
                Mutex::new(Heap::new(
                    heap_start().addr(),
                    heap_start().addr() - 0x2000_0000,
                )),
                Mutex::new(Heap::new(0x2001_0000, 65536)),
                Mutex::new(Heap::new(0x2002_0000, 65536)),
                Mutex::new(Heap::new(0x2003_0000, 65536)),
            ],
            cur_heap: Atomic::new(0),
        }
    }
    /// Create a new HeapRoot
    pub fn new() -> Self {
        static ALREADY_INIT: AtomicBool = AtomicBool::new(false);
        if ALREADY_INIT.swap(true) {
            panic!("Tried to initialize already initialized heap!");
        }
        unsafe { HeapRoot::new_unsafe() }
    }

    pub fn try_get_heap(&self) -> Option<MutexGuard<'_, RP2040Mutex, Heap>> {
        let heap_id = self.cur_heap.fetch_add(1) & 3;
        self.heaps[heap_id as usize].try_lock()
    }
    pub fn get_heap(&self) -> MutexGuard<'_, RP2040Mutex, Heap> {
        loop {
            match self.try_get_heap() {
                Some(v) => return v,
                None => {}
            }
        }
    }
    pub fn try_get_heap_for_pointer<T>(
        &self,
        ptr: *const T,
    ) -> Option<MutexGuard<'_, RP2040Mutex, Heap>> {
        let address = ptr.addr();
        if address < heap_start().addr() || address >= 0x2004_0000 {
            panic!(
                "Invalid pointer passed to the allocation code! (0x{:x} is out of heap range)",
                address
            );
        }
        let heap_id = (address >> 16) & 3;
        self.heaps[heap_id].try_lock()
    }
    pub fn get_heap_for_pointer<T>(&self, ptr: *const T) -> MutexGuard<'_, RP2040Mutex, Heap> {
        loop {
            match self.try_get_heap_for_pointer(ptr) {
                Some(v) => return v,
                None => {}
            }
        }
    }
}

unsafe impl Allocator for HeapRoot {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, AllocError> {
        // try 4 times
        let mut last_res = Err(AllocError);
        for _ in 0..4 {
            last_res = self.get_heap().allocate(layout);
            if last_res.is_ok() {
                break;
            }
        }
        last_res
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        self.get_heap_for_pointer(ptr.as_ptr() as *const _)
            .deallocate(ptr, layout)
    }
}
