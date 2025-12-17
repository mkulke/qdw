#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use spin::Once;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::instructions::{self, interrupts};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

const TIMER_VECTOR: u8 = 0x20;
const ERROR_VECTOR: u8 = 0x21;
const SPURIOUS_VECTOR: u8 = 0xFF;
const TICKS_PER_3_SECONDS: usize = 55;

const APIC_TIMER_DIVIDE: TimerDivide = TimerDivide::Div16;
const APIC_TIMER_INITIAL: u32 = 500_000;

static COUNTER: AtomicU8 = AtomicU8::new(0);
static LAPIC_BASE_VIRT: AtomicU64 = AtomicU64::new(0);

mod mem;

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    if ticks % TICKS_PER_3_SECONDS == 0 {
        write_str("tick #");
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        unsafe {
            serial_putc(b'0' + (counter % 10));
        }
        write_str("\r\n");
    }

    unsafe {
        with_lapic(|lapic| lapic.end_of_interrupt());
    }
}

extern "x86-interrupt" fn error_interrupt_handler(_sf: InterruptStackFrame) {
    write_str("lapic error\r\n");
    unsafe {
        with_lapic(|lapic| lapic.end_of_interrupt());
    }
}

extern "x86-interrupt" fn spurious_interrupt_handler(_sf: InterruptStackFrame) {
    unsafe {
        with_lapic(|lapic| lapic.end_of_interrupt());
    }
}

// we do this b/c LocalApic isn't send and cannot be put behind a mutex
// so we create a new instance each time, which is fine b/c it just maps to the same memory
unsafe fn with_lapic<R>(f: impl FnOnce(&mut LocalApic) -> R) -> R {
    let base = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    debug_assert!(base != 0);

    // This is just a constructor
    let mut lapic = LocalApicBuilder::new()
        .timer_vector(TIMER_VECTOR as usize)
        .error_vector(ERROR_VECTOR as usize)
        .spurious_vector(SPURIOUS_VECTOR as usize)
        .set_xapic_base(base)
        .build()
        .unwrap();

    f(&mut lapic)
}

unsafe extern "C" {
    fn serial_putc(ch: u8);
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

    unsafe {
        let lapic_phys = xapic_base();
        let lapic_vir = lapic_phys;
        LAPIC_BASE_VIRT.store(lapic_vir, Ordering::Relaxed);

        with_lapic(|lapic| {
            lapic.enable();

            lapic.set_timer_divide(APIC_TIMER_DIVIDE);
            lapic.set_timer_mode(TimerMode::Periodic);
            lapic.set_timer_initial(APIC_TIMER_INITIAL);
            lapic.enable_timer();
        });
    }

    write_str("\r\n");
    interrupts::enable();

    loop {
        instructions::hlt();
    }
}

fn write_str(s: &str) {
    for &b in s.as_bytes() {
        unsafe {
            serial_putc(b);
        }
    }
}
