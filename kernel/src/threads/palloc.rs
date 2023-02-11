use core::ptr;

use crate::{div_round_up, println, utils::data_structures::bit_set::BitSet};

use super::{
    addr::{page_number, ptov, Page, PhysAddr, VirtAddr, PAGE_SIZE},
    sync::lock,
};

bitflags::bitflags! {
    /// Flags for allocating pages.
    pub struct AllocateFlags: u8 {
        /// Zero the page contents.
        const ZERO = 0b0000_0001;

        /// User page.
        const USER = 0b0000_0010;
    }
}

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
pub struct PageAllocator {
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

    /// Initializes the page allocator. At most `user_page_limit` pages are put
    /// into the user pool.
    pub fn init(&self, boot_info: &'static bootloader_api::BootInfo, user_page_limit: usize) {
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

        println!();
    }

    /// Obtains a single free page and returns the allocated page.
    ///
    /// - If `AllocateFlags::USER` is set, the page is obtained from the user
    /// pool, otherwise from the kernel pool.
    /// - If `AllocateFlags::ZERO` is set, the page is filled with zeros.
    ///
    /// If no pages are available, returns `None`.
    #[must_use = "The allocated page should be used and freed, otherwise it would leak the memory"]
    pub fn get_page(&self, flags: AllocateFlags) -> Option<Page> {
        self.get_pages(1, flags)
    }

    /// Obtains and returns a group of `count` contiguous free pages.
    ///
    /// - If `AllocateFlags::USER` is set, the page is obtained from the user
    /// pool, otherwise from the kernel pool.
    /// - If `AllocateFlags::ZERO` is set, the page is filled with zeros.
    ///
    /// If too few pages are available, returns `None`.
    #[must_use = "The allocated page should be used and freed, otherwise it would leak the memory"]
    pub fn get_pages(&self, count: usize, flags: AllocateFlags) -> Option<Page> {
        if count == 0 {
            return None;
        }

        let Self {
            kernel_pool,
            user_pool,
        } = self;

        let pool = if flags.contains(AllocateFlags::USER) {
            user_pool
        } else {
            kernel_pool
        };

        let pages_start = pool.lock().allocate(count);

        if flags.contains(AllocateFlags::ZERO) {
            if let Some(pages_start) = pages_start {
                let start: *mut u8 = pages_start.start_address().as_mut_ptr();
                // We are allocating the physical memory page already mapped
                // the virtual memory, and it is aligned at the page, so it is
                // okay.
                unsafe {
                    ptr::write_bytes(start, 0, count * PAGE_SIZE);
                }
            }
        }

        pages_start
    }

    /// Frees the `page`.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that `page`
    /// is indeed a [`Page`] allocated from `Allocator::get_page` or
    /// `Allocator::get_pages`.
    pub unsafe fn free_page(&self, page: Page) {
        self.free_pages(page, 1);
    }

    /// Frees `count` pages starting from `page_start`.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that `count`
    /// pages are indeed [`Page`]s allocated from `Allocator::get_page` or
    /// `Allocator::get_pages`.
    pub unsafe fn free_pages(&self, page_start: Page, count: usize) {
        if count == 0 {
            return;
        }

        let pool = if self.kernel_pool.lock().contains_page(page_start) {
            &self.kernel_pool
        } else if self.user_pool.lock().contains_page(page_start) {
            &self.user_pool
        } else {
            panic!("Tried to free the page that has not been allocated");
        };

        // Clear the block to help detect use-after-free bugs.
        let start: *mut u8 = page_start.start_address().as_mut_ptr();
        unsafe {
            ptr::write_bytes(start, 0xcc, PAGE_SIZE * count);
        }

        pool.lock().free(page_start, count);
    }
}

/// A memory pool.
struct Pool {
    inner: Option<PoolInner>,
}

struct PoolInner {
    /// [`BitSet`] of free pages.
    ///
    /// We'll store the data structure itself at the start of the available
    /// region. Therefore it is stored as a pointer, not a owned data structure.
    used_map: ptr::NonNull<BitSet>,

    /// Starting page of the available region in the pool.
    base: Page,
}

impl Pool {
    const fn new() -> Self {
        Self { inner: None }
    }

    fn init(&mut self, buf: *mut u8, page_count: usize, name: &str) {
        let pages_used = div_round_up!(BitSet::buffer_size(page_count), PAGE_SIZE);
        let bytes_used = pages_used * PAGE_SIZE;
        let page_count = page_count - pages_used;

        println!("{page_count} pages available in {name}.");

        let used_map = unsafe {
            // Initialize the pool.
            ptr::NonNull::from(BitSet::from_buffer(page_count, buf, bytes_used))
        };

        let base = VirtAddr::new(buf as u64) + bytes_used;
        let base = Page::from_start_address(base).unwrap();

        self.inner = Some(PoolInner { used_map, base });
    }

    fn page_index(&self, page: Page) -> Option<usize> {
        if let Some(PoolInner { base, .. }) = self.inner {
            Some((page_number(page) - page_number(base)) as usize)
        } else {
            None
        }
    }

    fn contains_page(&self, page: Page) -> bool {
        if let Some(PoolInner { base, used_map }) = self.inner {
            let start_page = page_number(base);
            let end_page = unsafe { start_page + (*used_map.as_ptr()).size() as u64 };
            (start_page..end_page).contains(&page_number(page))
        } else {
            false
        }
    }

    fn allocate(&mut self, count: usize) -> Option<Page> {
        if let Some(PoolInner { base, used_map }) = self.inner {
            let used_map = unsafe { &mut (*used_map.as_ptr()) };
            if let Some(page_index) = used_map.scan(0, count, false) {
                used_map.set_many(page_index, count, true);
                Some(base + (page_index as u64))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn free(&mut self, page_start: Page, count: usize) {
        if let Some(PoolInner { used_map, .. }) = self.inner {
            let page_index = self.page_index(page_start).unwrap();
            let used_map = unsafe { &mut (*used_map.as_ptr()) };
            assert!(!used_map.contains(page_index, count, false));
            used_map.set_many(page_index, count, false);
        } else {
            panic!("Cannot free pages from uninitialized pool");
        }
    }
}

/// A global page allocator.
pub static PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Initialize the page allocation from the physical memory.
pub fn init(boot_info: &'static bootloader_api::BootInfo, user_page_limit: usize) {
    PAGE_ALLOCATOR.init(boot_info, user_page_limit);
}
