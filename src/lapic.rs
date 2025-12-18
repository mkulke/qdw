use core::ptr::addr_of_mut;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

pub const TIMER_VECTOR: u8 = 0x20;
pub const ERROR_VECTOR: u8 = 0x21;
pub const SPURIOUS_VECTOR: u8 = 0xFF;

const APIC_TIMER_DIVIDE: TimerDivide = TimerDivide::Div16;
const APIC_TIMER_INITIAL: u32 = 500_000;

static mut LAPIC: Lapic = Lapic::new();

pub fn lapic() -> &'static mut Lapic {
    unsafe { (*addr_of_mut!(LAPIC)).as_mut() }
}

enum LapicState {
    Uninitialized,
    Initialized(LocalApic),
}

pub struct Lapic {
    state: LapicState,
}

impl Lapic {
    const fn new() -> Self {
        Lapic {
            state: LapicState::Uninitialized,
        }
    }

    fn as_mut(&mut self) -> &mut Self {
        self
    }

    pub fn init(&mut self) {
        let base: u64;
        unsafe {
            base = xapic_base();
        }
        let lapic = LocalApicBuilder::new()
            .timer_vector(TIMER_VECTOR as usize)
            .error_vector(ERROR_VECTOR as usize)
            .spurious_vector(SPURIOUS_VECTOR as usize)
            .set_xapic_base(base)
            .build()
            .unwrap();

        self.state = LapicState::Initialized(lapic);
    }

    pub fn enable(&mut self) {
        if let LapicState::Initialized(lapic) = &mut self.state {
            unsafe {
                lapic.enable();
                lapic.set_timer_divide(APIC_TIMER_DIVIDE);
                lapic.set_timer_mode(TimerMode::Periodic);
                lapic.set_timer_initial(APIC_TIMER_INITIAL);
                lapic.enable_timer();
            }
        }
    }

    pub fn eof(&mut self) {
        if let LapicState::Initialized(lapic) = &mut self.state {
            unsafe {
                lapic.end_of_interrupt();
            }
        }
    }
}
