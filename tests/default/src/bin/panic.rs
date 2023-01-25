#![no_std]
#![no_main]

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    panic!("I panicked!")
}

bootloader_api::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        kernel::println!("{info}");
    }
    kernel::devices::shutdown::power_off()
}
