#![no_std]
#![feature(abi_x86_interrupt)]

pub mod cpu;
/// In 64-bit mode, the GDT is mostly used for two things:
/// Switching between kernel space and user space,
/// and loading a TSS structure.
pub mod gdt;
pub mod idt;
pub mod keyboard;
pub mod memory;
pub mod panic;
pub mod vga;

pub fn init() {
    gdt::init_gdt();
    idt::init_idt();
    idt::init_pics();
    idt::enable_interrupts();
}
