use crate::{gdt, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

pub const PIC_OFFSET_DELTA: u8 = 8;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + PIC_OFFSET_DELTA;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);

        idt[Interrupts::Timer as usize].set_handler_fn(int_timer_handler);
        idt[Interrupts::Keyboard as usize].set_handler_fn(int_keyboard_handler);
        idt[Interrupts::Syscall as usize].set_handler_fn(int_syscall_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Interrupts {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Syscall = 0x80,
}

impl Interrupts {
    fn end_of_interrupt(&self) {
        unsafe {
            PICS.lock().notify_end_of_interrupt(*self as u8);
        }
    }
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe { PICS.lock().initialize() }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable()
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Interrupted: breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Interrupted: divide error\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Interrupted: device not available\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Interrupted: invalid opcode\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    let access = x86_64::registers::control::Cr2::read();
    println!("Page fault when accessing {:?}", access);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "Interrupted: double fault (error code {})\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn int_timer_handler(_stack_frame: &mut InterruptStackFrame) {
    // TODO: setup a kernel timer
    Interrupts::Timer.end_of_interrupt();
}

extern "x86-interrupt" fn int_keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let code = unsafe { port.read() };

    crate::ktask::kernel_tasks::keyboard::add_scancode(code);
    Interrupts::Keyboard.end_of_interrupt();
}

extern "x86-interrupt" fn int_syscall_handler(_stack_frame: &mut InterruptStackFrame) {
    Interrupts::Syscall.end_of_interrupt()
}
