#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::sync::atomic::{AtomicUsize, Ordering};
use lapic::{lapic, ERROR_VECTOR, SPURIOUS_VECTOR, TIMER_VECTOR};
use serial::{write_hex_u8, write_str};
use spin::Once;
use x86_64::instructions::{self, interrupts};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static PRINT_EVENTS: AtomicUsize = AtomicUsize::new(0);

const TICKS_PER_3_SECONDS: usize = 55;

mod lapic;
mod mem;
mod serial;

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    if ticks % TICKS_PER_3_SECONDS == 0 {
        PRINT_EVENTS.fetch_add(1, Ordering::Relaxed);
    }

    lapic().eof();
}

extern "x86-interrupt" fn error_interrupt_handler(_sf: InterruptStackFrame) {
    write_str("lapic error\r\n");
    lapic().eof();
}

extern "x86-interrupt" fn spurious_interrupt_handler(_sf: InterruptStackFrame) {
    lapic().eof();
}

#[panic_handler]
fn panic(__info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt[TIMER_VECTOR].set_handler_fn(timer_interrupt_handler);
        idt[ERROR_VECTOR].set_handler_fn(error_interrupt_handler);
        idt[SPURIOUS_VECTOR].set_handler_fn(spurious_interrupt_handler);
        idt
    });
    idt.load();

    lapic().init();
    lapic().enable();

    serial::write_str("\r\n");
    interrupts::enable();

    let mut counter = 0;
    loop {
        instructions::hlt();

        let n = PRINT_EVENTS.swap(0, Ordering::AcqRel);
        for _ in 0..n {
            write_str("tick 0x");
            write_hex_u8(counter as u8);
            write_str("\r\n");
            counter += 1;
        }
    }
}
