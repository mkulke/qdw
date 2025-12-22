use crate::com::COM1_IRQ;
use x2apic::ioapic::{self, IrqFlags, IrqMode, RedirectionTableEntry};

const IOAPIC_BASE: u64 = 0xFEC0_0000;
const VECTOR_OFFSET: u8 = 0x20;
pub const COM1_VECTOR: u8 = VECTOR_OFFSET + COM1_IRQ;
const DEST_CPU: u8 = 0;

pub fn init() {
    let mut ioapic;
    unsafe {
        ioapic = ioapic::IoApic::new(IOAPIC_BASE);
        ioapic.init(VECTOR_OFFSET);
    }

    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_flags(IrqFlags::empty());
    entry.set_dest(DEST_CPU);
    entry.set_vector(COM1_VECTOR);

    unsafe {
        ioapic.set_table_entry(COM1_IRQ, entry);
        ioapic.enable_irq(COM1_IRQ);
    }
}
