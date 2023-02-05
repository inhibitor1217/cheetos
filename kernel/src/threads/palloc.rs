use core::ptr;

use crate::{div_round_up, println, utils::data_structures::bit_set::BitSet};

use super::{
    addr::{ptov, Page, PhysAddr, VirtAddr, PAGE_SIZE},
    sync::lock,
};

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
        let free_start = ptov(PhysAddr::new(free_region.start));
        let free_end = ptov(PhysAddr::new(free_region.end));
        let free_start_page = Page::from_start_address(free_start).unwrap();
        let free_end_page = Page::from_start_address(free_end).unwrap();

        let free_pages = (free_end_page - free_start_page) as usize;
        let user_pages = user_page_limit.min(free_pages / 2);
        let kernel_pages = free_pages - user_pages;

        let kernel_start = free_start;
        let user_start = (free_start_page + kernel_pages as u64).start_address();

        // Give half of memory to kernel, half to user.
        self.kernel_pool
            .lock()
            .init(kernel_start.as_mut_ptr(), kernel_pages, "kernel pool");
        self.user_pool
            .lock()
            .init(user_start.as_mut_ptr(), user_pages, "user pool");
    }
}

/// A memory pool.
struct Pool {
    /// [`BitSet`] of free pages.
    ///
    /// We'll store the data structure itself at the start of the available
    /// region. Therefore it is stored as a pointer, not a owned data structure.
    used_map: Option<ptr::NonNull<BitSet>>,

    /// Starting page of the available region in the pool.
    base: Option<Page>,
}

impl Pool {
    const fn new() -> Self {
        Self {
            used_map: None,
            base: None,
        }
    }

    fn init(&mut self, buf: *mut u8, page_count: usize, name: &str) {
        let pages_used = div_round_up!(BitSet::buffer_size(page_count), PAGE_SIZE);
        let bytes_used = pages_used * PAGE_SIZE;
        let page_count = page_count - pages_used;

        println!("{page_count} pages available in {name}.");

        unsafe {
            // Initialize the pool.
            self.used_map = Some(ptr::NonNull::from(BitSet::from_buffer(
                page_count, buf, bytes_used,
            )));
        }

        let base = VirtAddr::new(buf as u64) + bytes_used;
        self.base = Some(Page::from_start_address(base).unwrap());
    }
}

/// A global page allocator.
static PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Initializes the page allocator. At most `user_page_limit` pages are put
/// into the user pool.
pub fn init(boot_info: &'static bootloader_api::BootInfo, user_page_limit: usize) {
    PAGE_ALLOCATOR.init(boot_info, user_page_limit);
}
