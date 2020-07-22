use crate::{gdt, println, print};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
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

        idt[Interrupts::Timer as usize].set_handler_fn(int_timer_handler);
        idt[Interrupts::Keyboard as usize].set_handler_fn(int_keyboard_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

lazy_static! {
    static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = spin::Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Interrupts {
    Timer = PIC_1_OFFSET,
    Keyboard,
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
    // TODO: get key and print to screen
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let code = unsafe { port.read() };

    match keyboard.add_byte(code) {
        Ok(Some(event)) => match keyboard.process_keyevent(event) {
            Some(DecodedKey::RawKey(key)) => print!("{:?}", key),
            Some(DecodedKey::Unicode(char)) => print!("{}", char),
            _ => (),
        },
        _ => (),
    }

    Interrupts::Keyboard.end_of_interrupt();
}
