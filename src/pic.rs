use pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = 0x28;

pub fn disable() {
    unsafe {
        let mut pics = ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET);
        // Mask all interrupts (0xFF = all bits set = all masked)
        pics.write_masks(0xFF, 0xFF);
    }
}
