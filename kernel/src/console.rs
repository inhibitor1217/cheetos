use crate::devices::serial;
use crate::threads::Mutex;

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
    pub const fn new() -> Self {
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

/// Global console writer.
///
/// The serial layer do their own locking, so it's safe to call them at any
/// time. But this [`Mutex`] is useful to prevent simultaneous [`print`] calls
/// from mixing their output, which looks confusing.
pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    CONSOLE
        .lock()
        .write_fmt(args)
        .expect("Failed to write to console");
}

/// Prints to the console.
///
/// # Safety
/// This macro is unsafe because it uses a mutable static reference to the
/// global console writer. The caller should ensure that the console writer is
/// not accessed by other threads. We can remove the `unsafe` keyword after we
/// implement threads and synchronization.
///
/// TODO: implement synchronization and protect the console writer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

/// Prints to the console, with a newline.
///
/// # Safety
/// This macro is unsafe because it uses a mutable static reference to the
/// global console writer. The caller should ensure that the console writer is
/// not accessed by other threads. We can remove the `unsafe` keyword after we
/// implement threads and synchronization.
///
/// TODO: implement synchronization and protect the console writer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
