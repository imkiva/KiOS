#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use kios_kernel::println;

entry_point!(main);

fn main(boot: &'static BootInfo) -> ! {
    println!("Initializing the kernel...");
    kios_kernel::init();

    println!("hello world");
    println!("This is KiOS: an experimental operating-system written in Rust");
    println!("I love Rust!");

    println!("Kernel booted");
    kios_kernel::cpu::forever_hlt();
}
