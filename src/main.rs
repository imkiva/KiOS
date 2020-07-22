#![no_std]
#![no_main]

mod vga;

extern crate rlibc;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n\n*****");
    println!("Kernel panic: not syncing.");
    println!("{}", info);
    println!("*****\n");
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello world");
    println!("This is KiOS: an experimental operating-system written in Rust");
    println!("I love Rust!");

    panic!("No /init found");
}
