/// Transmission mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SerialMode {
    Uninitialized,
    Poll,
    Queue,
}

bitflags::bitflags! {
    struct InterruptEnable: u8 {
        /// Interrupts when data is available to be read.
        const RECEIVED_DATA_AVAILABLE = 0b0000_0001;

        /// Interrupts when trasmit finishes.
        const TRANSMITTER_HOLDING_EMPTY = 0b0000_0010;
    }

    struct FifoControl: u8 {}

    struct LineControl: u8 {
        /// No parity, 1 stop bit, 8 data bits.
        const N81 = 0b0000_0011;

        /// Enables DLAB.
        const DLAB = 0b1000_0000;
    }

    struct ModemControl: u8 {
        /// Controls OUT2 signal, an auxiliary output pin.
        const OUT2 = 0b0000_1000;
    }

    struct LineStatus: u8 {
        /// Receiver data ready.
        /// This is set when the receiver holding register is full and
        /// the last byte has been received.
        const DATA_READY = 0b0000_0001;

        /// Transmitter holding register empty.
        /// This is set when the transmitter holding register is empty and
        /// the last byte has been sent.
        const TRANSMITTER_EMPTY = 0b0010_0000;
    }
}

/// Register definitions for the 16550A UART used in PCs.
/// The 16550A has a lot more going on than shown here, but this
/// is all we need.
///
/// See [PC16550D](https://www.scs.stanford.edu/10wi-cs140/pintos/specs/pc16550d.pdf)
/// for the full specification.
pub struct Serial {
    mode: SerialMode,

    // DLAB = 0 registers.
    receiver_buffer: x86_64::instructions::port::PortReadOnly<u8>,
    transmitter_holding: x86_64::instructions::port::PortWriteOnly<u8>,
    interrupt_enable: x86_64::instructions::port::Port<u8>,

    // DLAB = 1 registers.
    divisor_latch_low: x86_64::instructions::port::Port<u8>,
    divisor_latch_high: x86_64::instructions::port::Port<u8>,

    // DLAB-insensitive registers.
    interrupt_identification: x86_64::instructions::port::PortReadOnly<u8>,
    fifo_control: x86_64::instructions::port::PortWriteOnly<u8>,
    line_control: x86_64::instructions::port::Port<u8>,
    modem_control: x86_64::instructions::port::Port<u8>,
    line_status: x86_64::instructions::port::PortReadOnly<u8>,
}

impl Serial {
    const BASE_BAUD_RATE: u32 = 1_843_200; // 1.8432 MHz
    const BAUD_RATE: u32 = 9_600; // 9.6 kbps

    /// Creates a new serial port connected to 0x3F8 (COM1).
    #[must_use = "Serial port must be initialized before use"]
    pub const fn new() -> Self {
        let io_base = 0x3F8u16;

        Self {
            mode: SerialMode::Uninitialized,

            receiver_buffer: x86_64::instructions::port::PortReadOnly::new(io_base),
            transmitter_holding: x86_64::instructions::port::PortWriteOnly::new(io_base),
            interrupt_enable: x86_64::instructions::port::Port::new(io_base + 1),

            divisor_latch_low: x86_64::instructions::port::Port::new(io_base),
            divisor_latch_high: x86_64::instructions::port::Port::new(io_base + 1),

            interrupt_identification: x86_64::instructions::port::PortReadOnly::new(io_base + 2),
            fifo_control: x86_64::instructions::port::PortWriteOnly::new(io_base + 2),
            line_control: x86_64::instructions::port::Port::new(io_base + 3),
            modem_control: x86_64::instructions::port::Port::new(io_base + 4),
            line_status: x86_64::instructions::port::PortReadOnly::new(io_base + 5),
        }
    }

    /// Initializes the serial port to a polling mode.
    ///
    /// Polling mode busy-waits for the serial port to become free before
    /// writing to it. It's slow, but until interrupts are enabled, it's all we
    /// can do.
    pub fn init_poll(&mut self) {
        assert_eq!(self.mode, SerialMode::Uninitialized);

        unsafe {
            // Turn off all interrupts.
            self.interrupt_enable.write(InterruptEnable::empty().bits());

            // Disable FIFO.
            self.fifo_control.write(FifoControl::empty().bits());

            // Set baud rate.
            self.set_baud_rate(Self::BAUD_RATE);

            // Enable OUT2 (required for interrupts).
            self.modem_control.write(ModemControl::OUT2.bits());
        }

        self.mode = SerialMode::Poll;
    }

    fn set_baud_rate(&mut self, baud_rate: u32) {
        use core::convert::TryFrom;

        assert!(baud_rate >= 300);
        assert!(baud_rate <= 115_200);

        let divisor = u16::try_from(Self::BASE_BAUD_RATE / baud_rate).expect("baud rate too low");

        unsafe {
            // Enable DLAB.
            self.line_control
                .write((LineControl::N81 | LineControl::DLAB).bits());

            // Set data rate.
            self.divisor_latch_low.write((divisor & 0xFF) as u8);
            self.divisor_latch_high.write((divisor >> 8) as u8);

            // Disable DLAB.
            self.line_control.write(LineControl::N81.bits());
        }
    }

    /// Sends a byte to the serial port.
    pub fn send(&mut self, data: u8) {
        match self.mode {
            SerialMode::Poll => self.send_poll(data),
            SerialMode::Queue => todo!(),
            SerialMode::Uninitialized => {
                self.init_poll();
                self.send_poll(data);
            }
        }
    }

    /// Polls the serial port until it's ready, and then transmits the given
    /// byte.
    fn send_poll(&mut self, data: u8) {
        assert_eq!(self.mode, SerialMode::Poll);

        unsafe {
            // Wait until the transmitter holding register is empty.
            while !LineStatus::from_bits_truncate(self.line_status.read())
                .intersects(LineStatus::TRANSMITTER_EMPTY)
            {}

            // Send the byte.
            self.transmitter_holding.write(data);
        }
    }
}

impl core::default::Default for Serial {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
