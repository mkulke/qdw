#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use alloc::format;
use core::fmt::{LowerHex, Write};
use core::sync::atomic::{AtomicUsize, Ordering};
use ioapic::COM1_VECTOR;
use lapic::{lapic, ERROR_VECTOR, SPURIOUS_VECTOR, TIMER_VECTOR};
use linked_list_allocator::LockedHeap;
use spin::Once;
use uart_16550::SerialPort;
use x86_64::instructions::{self, interrupts};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

mod com;
mod fpu;
mod ioapic;
mod irq_mutex;
mod lapic;
mod mem;
mod pic;

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static PRINT_EVENTS: AtomicUsize = AtomicUsize::new(0);

const TICKS_PER_3_SECONDS: usize = 55;

unsafe extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    unsafe {
        let start = &__heap_start as *const u8 as *mut u8;
        let end = &__heap_end as *const u8 as *mut u8;
        let size = end as usize - start as usize;

        ALLOCATOR.lock().init(start, size);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    if ticks % TICKS_PER_3_SECONDS == 0 {
        PRINT_EVENTS.fetch_add(1, Ordering::Relaxed);
    }

    lapic().eoi();
}

extern "x86-interrupt" fn error_interrupt_handler(_sf: InterruptStackFrame) {
    lapic().eoi();
}

extern "x86-interrupt" fn spurious_interrupt_handler(_sf: InterruptStackFrame) {
    lapic().eoi();
}

extern "x86-interrupt" fn com1_interrupt_handler(_stack_frame: InterruptStackFrame) {
    com::RX_QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        let (mut prod, _cons) = queue.split();
        while com::uart_rx_ready() {
            let byte = com::uart_read_byte();
            // we want to ignore overflow here
            _ = prod.enqueue(byte);
        }
    });
    lapic().eoi();
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

struct XmmBytes([u8; 16]);

impl LowerHex for XmmBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &b in self.0.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

fn dump_fpu_fxsave(tty: &mut SerialPort) {
    let mut area = fpu::FxSaveAligned::new_zeroed();
    fpu::fxsave64(&mut area);

    writeln!(tty, "=== fxsave64 ===").unwrap();
    writeln!(tty, "mxcsr=0x{:x}", area.0.mxcsr).unwrap();

    // for i in 0..16 {
    for i in [0, 15] {
        let value = XmmBytes(area.0.xmm[i]);
        let line = format!("xmm{:02}={:x}", i, value);
        writeln!(tty, "{}", line).unwrap();
    }
}

fn init() {
    init_heap();

    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt[TIMER_VECTOR].set_handler_fn(timer_interrupt_handler);
        idt[ERROR_VECTOR].set_handler_fn(error_interrupt_handler);
        idt[SPURIOUS_VECTOR].set_handler_fn(spurious_interrupt_handler);
        idt[COM1_VECTOR].set_handler_fn(com1_interrupt_handler);

        idt
    });
    idt.load();

    lapic().init();
    lapic().enable();

    pic::disable();
    ioapic::init();
    interrupts::enable();

    fpu::enable_sse();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    init();

    write_xmm_values();

    let mut tick_counter = 0;
    let mut com1_port = com::new_com1();
    loop {
        instructions::hlt();

        let n = PRINT_EVENTS.swap(0, Ordering::AcqRel);
        for _ in 0..n {
            writeln!(com1_port, "tick 0x{0:02x}", tick_counter).unwrap();
            dump_fpu_fxsave(&mut com1_port);
            tick_counter += 1;
        }

        com::RX_QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            let (_prod, mut cons) = queue.split();

            while let Some(byte) = cons.dequeue() {
                writeln!(com1_port, "COM1 RX: 0x{:02x} ('{}')", byte, byte as char).unwrap();
            }
        });
    }
}
