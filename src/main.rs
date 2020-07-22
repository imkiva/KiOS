#![no_std]
#![no_main]

use kios_kernel::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Initializing the kernel...");
    kios_kernel::init();

    println!("hello world");
    println!("This is KiOS: an experimental operating-system written in Rust");
    println!("I love Rust!");

    println!("Kernel booted");
    kios_kernel::cpu::forever_hlt();
}
