#![no_std]
#![no_main]

mod vga;

extern crate rlibc;
use core::panic::PanicInfo;

static HELLO: &[u8] = b"hello world";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
