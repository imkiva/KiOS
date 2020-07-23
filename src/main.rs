#![no_std]
#![no_main]

extern crate alloc;
use bootloader::{entry_point, BootInfo};
use kios_kernel::println;

entry_point!(main);

fn main(boot: &'static BootInfo) -> ! {
    println!("Initializing the kernel...");
    kios_kernel::init(boot);

    println!("hello world");
    println!("This is KiOS: an experimental operating-system written in Rust");
    println!("I love Rust!");

    println!("Kernel booted");
    kios_kernel::cpu::forever_hlt();
}
