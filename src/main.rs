#![no_std]
#![no_main]

unsafe extern "C" {
    fn serial_putc(ch: u8);
}

#[panic_handler]
fn panic(__info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    let mut n: u8 = 0;
    write_str("\r\n");
    loop {
        write_str("tick #");
        unsafe {
            serial_putc(b'0' + n);
        }
        write_str("\r\n");
        n = n.wrapping_add(1) % 10;
        delay_cycles(9_000_000_000);
    }
}

fn write_str(s: &str) {
    for &b in s.as_bytes() {
        unsafe {
            serial_putc(b);
        }
    }
}

#[inline(always)]
fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

fn delay_cycles(cycles: u64) {
    let start = rdtsc();
    while rdtsc().wrapping_sub(start) < cycles {
        core::hint::spin_loop();
    }
}
