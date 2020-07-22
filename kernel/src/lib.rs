#![no_std]

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
