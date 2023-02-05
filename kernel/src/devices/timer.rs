use super::pit::{Channel, Mode, PIT};

/// Number of timer interrupts per second.
pub const FREQUENCY: usize = 100;

/// Sets up the timer to interrupt [`TIMER_FREQUENCY`] times per second, and
/// registers the corresponding interrupt.
pub fn init() {
    PIT.lock()
        .configure(Channel::OUT0, Mode::RateGenerator, FREQUENCY);
}
