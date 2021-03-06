use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use crate::allocators;

/// Virtual address of kernel heap start
pub const KERNEL_HEAP_START: u64 = 0x_0000_7000_0000;
pub const KERNEL_HEAP_INIT_SIZE: u64 = SIZE_4MIB as u64;

pub const SIZE_1MIB: usize = 1024 * 1024;
pub const SIZE_4MIB: usize = SIZE_1MIB * 4;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("kalloc error: {:?}", layout)
}

pub fn init_kernel_heap(
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(KERNEL_HEAP_START);
    let heap_end = heap_start + KERNEL_HEAP_INIT_SIZE;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);

    let range = Page::range(heap_start_page, heap_end_page);
    for page in range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)?
                .flush()
        };
    }

    // initialize kernel allocator
    unsafe {
        allocators::KERNEL_ALLOCATOR3
            .lock()
            .init(KERNEL_HEAP_START as usize, KERNEL_HEAP_INIT_SIZE as usize);
    }
    Ok(())
}
