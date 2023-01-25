#![no_std]
#![no_main]

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::init(boot_info);

    unsafe {
        kernel::println!("Hello, world!");
    }

    kernel::devices::shutdown::power_off()
}

bootloader_api::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kernel::devices::shutdown::power_off()
}
