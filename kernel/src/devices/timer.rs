use crate::threads::{Mutex, INTERRUPT_REGISTRY, SCHEDULER};

use super::pit::{Channel, Mode, PIT};

/// Number of timer interrupts per second.
pub const FREQUENCY: usize = 100;

/// Sets up the timer to interrupt [`FREQUENCY`] times per second, and
/// registers the corresponding interrupt.
pub fn init() {
    PIT.lock()
        .configure(Channel::OUT0, Mode::RateGenerator, FREQUENCY);

    INTERRUPT_REGISTRY
        .lock()
        .register(0x20, interrupt, "8254 Timer");
}

/// Manages the ticks and calibration.
pub struct Timer {
    /// Number of timer ticks since OS booted.
    ticks: usize,
}

impl Timer {
    /// Creates a new [`Timer`].
    pub const fn new() -> Timer {
        Self { ticks: 0 }
    }

    /// Timer tick.
    pub fn tick(&mut self) {
        self.ticks += 1;
    }
}

/// Global timer.
pub static TIMER: Mutex<Timer> = Mutex::new(Timer::new());

/// Timer interrupt handler.
fn interrupt(_frame: x86_64::structures::idt::InterruptStackFrame) {
    TIMER.lock().tick();
    SCHEDULER.lock().tick();
}
