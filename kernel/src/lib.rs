#![no_std]
#![feature(abi_x86_interrupt)]

/// In 64-bit mode, the GDT is mostly used for two things:
/// Switching between kernel space and user space,
/// and loading a TSS structure.
pub mod gdt;
pub mod interrupts;
pub mod vga;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n\n*****");
    println!("Kernel panic: not syncing.");
    println!("{}", info);
    println!("*****\n");
    loop {}
}

pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
}
