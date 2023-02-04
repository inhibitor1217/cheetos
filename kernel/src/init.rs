use crate::{println, threads};

/// Initializes the kernel.
pub fn init(boot_info: &'static bootloader_api::BootInfo) {
    // Initialize ourselves as a thread so we can use locks.
    threads::thread_init();

    greet(boot_info);

    // Initialize memory system.
    threads::palloc_init(usize::MAX);

    // Initialize interrupt handlers.
    threads::interrupt_init();
}

fn greet(boot_info: &bootloader_api::BootInfo) {
    println!("cheetos booting...");

    println!();
    println!("BOOT INFO:");
    println!("memory_regions = {:?}", boot_info.memory_regions.as_ref());
    println!("framebuffer = {:?}", boot_info.framebuffer);
    println!(
        "physical_memory_offset = {:?}",
        boot_info.physical_memory_offset
    );
    println!("recursive_index = {:?}", boot_info.recursive_index);
    println!("rsdp_addr = {:?}", boot_info.rsdp_addr);
    println!("tls_template = {:?}", boot_info.tls_template);
    println!();
}
