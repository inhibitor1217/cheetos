use crate::{console::CONSOLE, println};

use super::timer::TIMER;

// We configured this by running QEMU with
// `-device isa-debug-exit,iobase=0xf4,iosize=0x04`.
const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;

// We use 0x31 as the exit code.
const ISA_DEBUG_EXIT_CODE_SUCCESS: u8 = 0x31;

/// Powers down the machine we're running on,
/// as long as we're running on QEMU.
pub fn power_off() -> ! {
    print_stats();

    println!("Powering off...");

    let mut port = x86_64::instructions::port::Port::new(ISA_DEBUG_EXIT_PORT);
    unsafe {
        port.write(ISA_DEBUG_EXIT_CODE_SUCCESS);
    }

    // If we're not running on QEMU, we'll just loop forever.
    loop {
        x86_64::instructions::hlt();
    }
}

/// Prints statistics about `cheetos` kernel execution.
fn print_stats() {
    TIMER.lock().print_stats();
    CONSOLE.lock().print_stats();
}
