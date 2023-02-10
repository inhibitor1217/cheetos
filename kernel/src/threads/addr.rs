/// Physical address.
pub type PhysAddr = x86_64::PhysAddr;

/// Virtual address.
pub type VirtAddr = x86_64::VirtAddr;

/// Page (4 KiB size).
pub type Page = x86_64::structures::paging::Page<x86_64::structures::paging::Size4KiB>;

/// Number of bytes in a page.
pub const PAGE_SIZE: usize = 4096;

/// Base address of the 1:1 physical-to-virtual mapping. Physical memory is
/// mapped starting at this virtual address. Thus, physical address 0 is
/// accessible at [`PHYS_BASE`], physical address 0x1234 at `PHYS_BASE + 0x1234`
/// and so on.
///
/// This address also marks the end of user programs' address space. Up to this
/// point in memory, user programs are allowed to map whatever they like. At
/// this point and above, the virtual address space belongs to the kernel.
pub const PHYS_BASE: u64 = 0x0100_0000_0000;

/// Returns `true` if `vaddr` is a user virtual address.
pub fn is_user_vaddr(vaddr: VirtAddr) -> bool {
    vaddr < VirtAddr::new(PHYS_BASE)
}

/// Returns `true` if `vaddr` is a kernel virtual address.
pub fn is_kernel_vaddr(vaddr: VirtAddr) -> bool {
    vaddr >= VirtAddr::new(PHYS_BASE)
}

/// Returns kernel virtual address at which physical address `paddr` is mapped.
pub fn ptov(paddr: PhysAddr) -> VirtAddr {
    assert!(paddr < PhysAddr::new(PHYS_BASE));
    VirtAddr::new(paddr.as_u64() + PHYS_BASE)
}

/// Returns physical address at which kernel virtual address `vaddr` is mapped.
pub fn vtop(vaddr: VirtAddr) -> PhysAddr {
    assert!(is_kernel_vaddr(vaddr));
    PhysAddr::new(vaddr.as_u64() - PHYS_BASE)
}

/// Returns the page number.
pub fn page_number(page: Page) -> u64 {
    const PAGE_BITS: usize = 12;
    return page.start_address().as_u64() >> PAGE_BITS;
}
