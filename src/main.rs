#![no_std]
#![no_main]

mod vga;

extern crate rlibc;
use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    write!(
        vga::WRITER.lock(),
        "hello world\nThis is KiOS: an experimental operating-system written in Rust\nI love Rust!"
    ).unwrap();
    loop {}
}
