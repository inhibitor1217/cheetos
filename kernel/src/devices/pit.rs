use crate::threads::Mutex;

/// In a PC, the PIT's three output channels are hooked up like this:
///
/// - Channel 0 is connected to interrupt line 0, so that it can be used as a
///   periodic time interrupt, as implemented in `cheetos` in `devices::timer`.
///
/// - Channel 1 is used for dynamic RAM refresh (in older PCs). No good can come
///   of messing with this.
///
/// - Channel 2 is connected to the PC speaker, so that it can be used to play a
///   tone, as implemented in `cheetos` in `devices::speaker`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Channel {
    OUT0 = 0,
    OUT2 = 2,
}

/// Represents the form of output:
///
/// - Mode 2 is a periodic pulse: the channel's output is 1 for most of the
///   period, but drops to 0 briefly towards the end of the period. This is
///   useful for hooking up to an interrupt controller to generate a periodic
///   interrupt.
///
/// - Mode 3 is a square wave: for the first half of the period it is 1, for the
///   second half it is 0. This is useful for generating a tone on a speaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    RateGenerator = 2,
    SquareWave = 3,
}

/// Interface to 8254 Programmable Interrupt Timer (PIT).
#[derive(Debug)]
pub struct Pit {
    control: x86_64::instructions::port::Port<u8>,
    out0: x86_64::instructions::port::Port<u8>,
    out2: x86_64::instructions::port::Port<u8>,
}

impl Pit {
    /// Control port.
    const CONTROL: u16 = 0x43;

    /// Counter port for OUT0.
    const COUNTER_OUT0: u16 = 0x40;

    /// Counter port for OUT2.
    const COUNTER_OUT2: u16 = 0x42;

    /// Minimum frequency: the quotient would overflow the 16-bit counter if the
    /// frequency is lower than this.
    const MIN_FREQUENCY: usize = 19;

    /// Maximum frequency: the quotient would underflow to zero, which the PIT
    /// would interpret as 65536.
    const MAX_FREQUENCY: usize = 1193180;

    /// Creates a new interface for [`Pit`].
    pub const fn new() -> Self {
        Self {
            control: x86_64::instructions::port::Port::new(Self::CONTROL),
            out0: x86_64::instructions::port::Port::new(Self::COUNTER_OUT0),
            out2: x86_64::instructions::port::Port::new(Self::COUNTER_OUT2),
        }
    }

    pub fn configure(&mut self, channel: Channel, mode: Mode, frequency: usize) {
        let count = {
            if frequency < Self::MIN_FREQUENCY {
                // Force the count to 0, which the PIT treats as 65536, the
                // highest possible count. This yields a 18.2 Hz timer,
                // approximately.
                0
            } else if frequency > Self::MAX_FREQUENCY {
                // A count of 1 is illegal in mode 2, so we force it to 2, which
                // yield a 596.590 kHz timer, approximately. (This timer rate
                // is probably too fast to be useful anyhow.)
                2
            } else {
                (Self::MAX_FREQUENCY + frequency / 2) / frequency
            }
        } as u16;

        // Configure the PIT mode and load its counters.
        unsafe {
            self.control
                .write(((channel as u8) << 6) | 0x30 | ((mode as u8) << 1));

            let Self { out0, out2, .. } = self;
            let out = match channel {
                Channel::OUT0 => out0,
                Channel::OUT2 => out2,
            };

            let [low, high] = [(count >> 8) as u8, count as u8];
            out.write(low);
            out.write(high);
        }
    }
}

/// Global [`Pit`].
pub static PIT: Mutex<Pit> = Mutex::new(Pit::new());
