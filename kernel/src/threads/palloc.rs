use crate::println;

use super::sync::lock;

/// Page allocator. Hands out memory in page-size (or page-multiple) chunks.
///
/// System memory is divided into two [`Pool`]s called the kernel and user
/// pools. The user pool is the memory for user (virtual) memory pages, the
/// kernel pool for everything else. The idea here is that the kernel needs to
/// have memory for its own operations even if the user processes are swapping
/// like mad.
///
/// By default, half of system RAM is given to the kernel pool and half to the
/// user pool. That should be huge overkill for the kernel pool, but that's just
/// fine for demonstration purposes.
struct PageAllocator {
    /// Memory pool for kernel data.
    kernel_pool: lock::Mutex<Pool>,

    /// Memory pool for user data.
    user_pool: lock::Mutex<Pool>,
}

type VirtAddr = x86_64::VirtAddr;
type Page = x86_64::structures::paging::Page<x86_64::structures::paging::Size4KiB>;

impl PageAllocator {
    const fn new() -> Self {
        Self {
            kernel_pool: lock::Mutex::new(Pool::new()),
            user_pool: lock::Mutex::new(Pool::new()),
        }
    }

    fn init(&self, boot_info: &'static bootloader_api::BootInfo, user_page_limit: usize) {
        // Retrieve the usable memory region from bootloader metadata.
        let free_region = boot_info
            .memory_regions
            .iter()
            .find(|region| region.kind == bootloader_api::info::MemoryRegionKind::Usable)
            .unwrap();
        let free_start = Page::containing_address(VirtAddr::new(free_region.start));
        let free_end = Page::containing_address(VirtAddr::new(free_region.end));

        let free_pages = (free_end - free_start) as usize;
        let user_pages = user_page_limit.min(free_pages / 2);
        let kernel_pages = free_pages - user_pages;

        // Give half of memory to kernel, half to user.
        self.kernel_pool.lock().init(kernel_pages, "kernel pool");
        self.user_pool.lock().init(user_pages, "user pool");
    }
}

/// A memory pool.
struct Pool {}

impl Pool {
    const fn new() -> Self {
        Self {}
    }

    fn init(&mut self, page_count: usize, name: &str) {
        println!("{page_count} pages available in {name}.");
    }
}

/// A global page allocator.
static PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Initializes the page allocator. At most `user_page_limit` pages are put
/// into the user pool.
pub fn init(boot_info: &'static bootloader_api::BootInfo, user_page_limit: usize) {
    PAGE_ALLOCATOR.init(boot_info, user_page_limit);
}
