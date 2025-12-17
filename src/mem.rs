// implementation of memcpy, required because of idt.load()

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;

    while i + 8 <= n {
        unsafe {
            *(dest.add(i) as *mut u64) = *(src.add(i) as *const u64);
        }
        i += 8;
    }

    while i < n {
        unsafe {
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}
