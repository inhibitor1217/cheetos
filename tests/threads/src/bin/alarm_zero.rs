#![no_std]
#![no_main]

static TEST_NAME: &str = "alarm_zero";

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::init(boot_info);

    kernel::devices::timer::sleep(0);
    kernel_test::pass!(TEST_NAME);

    kernel::devices::shutdown::power_off();
}

kernel::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kernel::println!("{info}");
    kernel::devices::shutdown::power_off_with_failure()
}
