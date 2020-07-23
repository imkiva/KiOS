#![no_std]
#![no_main]

extern crate alloc;
use bootloader::{entry_point, BootInfo};
use kios_kernel::{
    ktask::{executor::Executor, kernel_tasks::keyboard, KernelTask},
    println,
};

entry_point!(main);

fn main(boot: &'static BootInfo) -> ! {
    println!("Initializing the kernel...");
    kios_kernel::init(boot);

    println!("hello world");
    println!("This is KiOS: an experimental operating-system written in Rust");
    println!("I love Rust!");

    println!("Kernel booted");

    let mut executor = Executor::new();
    executor.spawn(KernelTask::new(keyboard::print_keyevents()));
    executor.run();
}
