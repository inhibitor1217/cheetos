#![no_std]
#![no_main]

/// The entry point of the kernel.
///
/// This function is called by the bootloader after the kernel has been loaded into memory.
/// `boot_info` contains information about available memory, the framebuffer, etc.
/// See the [`bootloader_api`] crate for more information.
fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    loop {}
}

// This macro generates a `_start` entry point symbol that the bootloader looks for.
bootloader_api::entry_point!(kernel_main);

/// The kernel panic handler.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Until we implement a binding to serial port, we just loop forever.
    loop {}
}
