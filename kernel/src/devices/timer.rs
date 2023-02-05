use crate::{
    println,
    threads::{InterruptMutex, INTERRUPT_REGISTRY, SCHEDULER},
};

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

    /// Returns the number of timer ticks since the OS booted.
    pub fn ticks(&mut self) -> usize {
        self.ticks
    }

    /// Returns the number of timer ticks elapsed since `then`.
    pub fn elapsed(&mut self, then: usize) -> usize {
        self.ticks() - then
    }

    /// Prints timer statistics.
    pub fn print_stats(&mut self) {
        println!("Timer: {} ticks", self.ticks());
    }
}

/// Global timer.
pub static TIMER: InterruptMutex<Timer> = InterruptMutex::new(Timer::new());

/// Timer interrupt handler.
fn interrupt(_frame: x86_64::structures::idt::InterruptStackFrame) {
    TIMER.lock().tick();
    SCHEDULER.lock().tick();
}
