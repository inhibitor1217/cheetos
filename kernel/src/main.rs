#![no_std]
#![no_main]

/// The entry point of the kernel.
///
/// This function is called by the bootloader after the kernel has been loaded
/// into memory. `boot_info` contains information about available memory, the
/// framebuffer, etc.
///
/// See the [`bootloader_api`] crate for more information.
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::init(boot_info);

    kernel::devices::shutdown::power_off()
}

// This macro generates a `_start` entry point symbol that the bootloader looks
// for.
kernel::entry_point!(kernel_main);

/// The kernel panic handler.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Print the panic message and information.
    kernel::println!("{info}");

    // Shut down the system.
    kernel::devices::shutdown::power_off()
}
