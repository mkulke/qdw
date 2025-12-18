unsafe extern "C" {
    fn serial_putc(ch: u8);
}

// pub fn write_char(c: u8) {
//     unsafe {
//         serial_putc(c as u8);
//     }
// }

pub fn write_str(s: &str) {
    for &b in s.as_bytes() {
        unsafe {
            serial_putc(b);
        }
    }
}

pub fn write_hex_u8(b: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    unsafe {
        serial_putc(HEX[(b >> 4) as usize]);
    }
    unsafe {
        serial_putc(HEX[(b & 0xF) as usize]);
    }
}

pub fn write_hex_u32(x: u32) {
    for i in (0..4).rev() {
        write_hex_u8((x >> (i * 8)) as u8);
    }
}

pub fn write_dec_u8(mut v: u8) {
    // prints 0..255 without heap
    let mut buf = [0u8; 3];
    let mut n = 0;

    if v == 0 {
        unsafe { serial_putc(b'0') };
        return;
    }

    while v != 0 {
        buf[n] = b'0' + (v % 10);
        v /= 10;
        n += 1;
    }

    while n != 0 {
        n -= 1;
        unsafe {
            serial_putc(buf[n]);
        }
    }
}

pub fn write_rn() {
    unsafe {
        serial_putc(b'\r');
        serial_putc(b'\n');
    }
}
