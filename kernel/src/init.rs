use crate::{
    devices, println,
    threads::{self},
};

/// Initializes the kernel.
pub fn init(boot_info: &'static bootloader_api::BootInfo) {
    // Initialize ourselves as a thread so we can use locks.
    threads::thread_init();

    greet(boot_info);

    // Initialize memory system.
    threads::palloc_init(boot_info, usize::MAX);
    threads::alloc_init();

    // Initialize interrupt handlers.
    threads::interrupt_init();
    devices::timer::init();

    // Start thread scheduler and enable interrupts.
    threads::SCHEDULER.lock().start();

    println!("Boot complete.");
    println!();
}

fn greet(boot_info: &bootloader_api::BootInfo) {
    let free_region = boot_info
        .memory_regions
        .iter()
        .find(|region| region.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .unwrap();

    let ram_size = free_region.end - free_region.start;

    println!("cheetos booting with {} kB RAM.", ram_size >> 10);
    println!();
}
