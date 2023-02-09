#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::init(boot_info);

    const NUM_BOXES: usize = 1024;
    const VEC_SIZE: usize = 10000;

    let long_lived_box = Box::new(42);

    for i in 0..NUM_BOXES {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }

    let mut vec = vec![];
    for i in 0..VEC_SIZE {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<usize>(), (VEC_SIZE - 1) * VEC_SIZE / 2);

    assert_eq!(*long_lived_box, 42);

    kernel::devices::shutdown::power_off()
}

kernel::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kernel::println!("{info}");
    kernel::devices::shutdown::power_off()
}
