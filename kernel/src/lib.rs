#![no_std]
#![warn(clippy::all)]
#![feature(abi_x86_interrupt)]

pub mod console;
pub mod devices;
pub mod init;
pub mod threads;
pub mod utils;

pub use init::init;

#[macro_export]
macro_rules! entry_point {
    ($main:ident) => {
        const KERNEL_STACK_SIZE: u64 = kernel::threads::thread::Thread::STACK_SIZE as u64;

        /// Configuration to pass into bootloader, to control memory mappings, etc.
        pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
            let mut config = bootloader_api::BootloaderConfig::new_default();
            config.kernel_stack_size = KERNEL_STACK_SIZE;
            config.mappings.physical_memory = Some(bootloader_api::config::Mapping::FixedAddress(
                $crate::threads::addr::PHYS_BASE,
            ));
            config
        };

        bootloader_api::entry_point!($main, config = &BOOTLOADER_CONFIG);
    };
}
