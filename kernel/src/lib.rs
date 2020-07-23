#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]

extern crate alloc;

use crate::memory::BootInfoFrameAllocator;
use bootloader::BootInfo;
use x86_64::VirtAddr;

pub mod allocators;
pub mod cpu;
/// In 64-bit mode, the GDT is mostly used for two things:
/// Switching between kernel space and user space,
/// and loading a TSS structure.
pub mod gdt;
pub mod idt;
pub mod kalloc;
pub mod ktask;
pub mod memory;
pub mod panic;
pub mod vga;

pub fn init(boot: &'static BootInfo) {
    gdt::init_gdt();
    idt::init_idt();
    idt::init_pics();
    idt::enable_interrupts();

    let mut page_table = memory::init(VirtAddr::new(boot.physical_memory_offset));
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot.memory_map);

    kalloc::init_kernel_heap(&mut page_table, &mut frame_allocator)
        .expect("heap initialization failed");
}
