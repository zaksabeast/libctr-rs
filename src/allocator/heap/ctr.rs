use crate::result::parse_result;
use core::alloc::Layout;
use ctru_sys::svcControlMemory;
use linked_list_allocator::LockedHeap;

#[alloc_error_handler]
fn handle_out_of_memory(_layout: Layout) -> ! {
    panic!()
}

const HEAP_MB_SIZE: usize = 10;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// # Safety
/// This function should only be used one time.
pub unsafe fn init_heap() {
    let heap_byte_size = HEAP_MB_SIZE * 1000000;
    let aligned_heap_size = heap_byte_size - (heap_byte_size % 0x1000);

    let mut ctru_heap_ptr: u32 = 0;
    parse_result(svcControlMemory(
        &mut ctru_heap_ptr,
        0x8000000,
        0x0,
        aligned_heap_size as u32,
        3, // MEMOP_ALLOC,
        3, // MEMPERM_READ | MEMPERM_WRITE,
    ))
    .unwrap();

    ALLOCATOR
        .lock()
        .init(ctru_heap_ptr as *mut u8, aligned_heap_size)
}
