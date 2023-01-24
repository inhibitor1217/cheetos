use crate::devices::serial;

/// Console writer for kernel.
/// 
/// For now, it writes bytes to the serial port, but in the future it will also
/// display the characters to the frame buffer.
pub struct Console {
    // Serial port device used for console output.
    serial: serial::Serial,

    // Number of characters written to console.
    write_cnt: usize,
}

impl Console {
    /// Creates a new console writer.
    #[must_use = "Console must be initialized before use"]
    pub fn new() -> Self {
        Self {
            serial: serial::Serial::new(),
            write_cnt: 0,
        }
    }

    /// Initializes the console.
    pub fn init(&mut self) {
        // Initialize serial port to polling mode:
        // so that we can write to it before interrupts are enabled.
        self.serial.init_poll();
    }

    /// Prints console statistics.
    pub fn print_stats(&mut self) {
        use core::fmt::Write;
        let write_cnt = self.write_cnt;
        writeln!(self, "Console: {write_cnt} characters output")
            .expect("Failed to write to console");
    }
}

impl core::default::Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.serial.write_str(s)?;
        self.write_cnt += s.len();
        Ok(())
    }
}
