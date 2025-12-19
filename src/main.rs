#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::sync::atomic::{AtomicUsize, Ordering};
use lapic::{lapic, ERROR_VECTOR, SPURIOUS_VECTOR, TIMER_VECTOR};
use serial::SerialWriter;
use spin::Once;
use x86_64::instructions::{self, interrupts};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static PRINT_EVENTS: AtomicUsize = AtomicUsize::new(0);

const TICKS_PER_3_SECONDS: usize = 55;

mod fpu;
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
    lapic().eof();
}

extern "x86-interrupt" fn spurious_interrupt_handler(_sf: InterruptStackFrame) {
    lapic().eof();
}

#[panic_handler]
fn panic(__info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn write_xmm_values() {
    let mut xmm = [0u8; 16];
    let a = 0x0011223344556677u64;
    let b = 0x8899AABBCCDDEEFFu64;
    xmm[0..8].copy_from_slice(&a.to_le_bytes());
    xmm[8..16].copy_from_slice(&b.to_le_bytes());
    fpu::set_xmm0_bytes(&xmm);
    xmm.reverse();
    fpu::set_xmm15_bytes(&xmm);
}

fn dump_fpu_fxsave(tty: &mut SerialWriter) {
    let mut area = fpu::FxSaveAligned::new_zeroed();
    fpu::fxsave64(&mut area);

    tty.write_str("=== fxsave64 ===\r\n");
    tty.write_str("mxcsr=0x");
    tty.write_hex_u32(area.0.mxcsr);
    tty.write_str("\r\n");

    // for i in 0..16 {
    for i in [0, 15] {
        tty.write_str("xmm");
        if i < 10 {
            tty.write_char(b'0');
        }
        tty.write_dec_u8(i as u8);
        tty.write_char(b'=');
        for &b in &area.0.xmm[i] {
            tty.write_hex_u8(b);
        }
        tty.write_str("\r\n");
    }
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

    let mut tty = SerialWriter::new();
    tty.write_str("\r\n");
    interrupts::enable();

    fpu::enable_sse();
    write_xmm_values();

    let mut counter = 0;
    loop {
        instructions::hlt();

        let n = PRINT_EVENTS.swap(0, Ordering::AcqRel);
        for _ in 0..n {
            tty.write_str("tick 0x");
            tty.write_hex_u8(counter as u8);
            tty.write_str("\r\n");
            dump_fpu_fxsave(&mut tty);
            counter += 1;
        }
    }
}
