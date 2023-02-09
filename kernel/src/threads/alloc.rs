use crate::utils::data_structures::linked_list::{LinkedList, Node};

use super::addr::PAGE_SIZE;
use super::sync::lock::Mutex;

/// Free block.
#[derive(Debug)]
#[repr(C)]
struct Block {
    node: Node,
}

/// Descriptor of the list of fixed sized blocks for allocation.
#[derive(Debug)]
struct Descriptor {
    /// Size of each element in bytes.
    block_size: usize,

    /// Number of blocks in an arena.
    blocks_per_arena: usize,

    /// List of free blocks.
    free_list: LinkedList<Block>,
}

/// Arena owning the descriptor.
#[derive(Debug)]
struct Arena {
    /// Always set to `Arena::MAGIC` for detecting corruption.
    magic: u32,

    /// Free blocks; pages in big block.
    free_cnt: usize,
}

impl Arena {
    /// Magic number for detecting arena corruption.
    const MAGIC: u32 = 0x8a547eed;
}

/// A simple memory alloocator.
///
/// In Rust, the global memory allocator can be registered as a standard
/// library's default, through using the `#[global_allocator]` attribute. This
/// allows using heap-allocated structs such as `alloc::boxed::Box` as if we are
/// using the standard library, but providing the actual functionality from the
/// kernel itself.
///
/// The size of each request, in bytes, is rounded up to a power of 2 and
/// assigned to the "descriptor" that manages blocks of that size. The
/// descriptor keeps a list of free blocks. If the free list is nonempty, one of
/// its blocks is used to satisfy the request.
///
/// Otherwise, a new page of memory, called an "arena", is obtained from the
/// page allocator (if none is available, `alloc` returns a null pointer). The
/// new arena is divided into blocks, all of which are added to the descriptor's
/// free list. Then we return one of the new blocks.
///
/// When we free a block, we add it to its descriptor's free list. But if the
/// arena that the block was in now has no in-use blocks, we remove all of the
/// arena's blocks from the free list and give the arena back to the page
/// allocator.
///
/// We can't handle blocks bigger than 2 kB using this scheme, because they're
/// too big to fit in a single page with a descriptor. We handle those by
/// allocating contiguous pages with the page allocator and sticking the
/// allocation size at the beginning of the allocated block's arena header.
pub struct Allocator {
    descriptors: [Mutex<Descriptor>; Self::DESCRIPTORS_SIZE],
}

macro_rules! make_descriptor {
    ($block_size:expr) => {{
        Mutex::new(Descriptor {
            block_size: $block_size,
            blocks_per_arena: (PAGE_SIZE - core::mem::size_of::<Arena>()) / $block_size,
            free_list: LinkedList::new(),
        })
    }};
}

impl Allocator {
    const DESCRIPTORS_SIZE: usize = 7;

    /// Creates a new allocator.
    pub const fn new() -> Self {
        Self {
            descriptors: [
                make_descriptor!(1 << 4),
                make_descriptor!(1 << 5),
                make_descriptor!(1 << 6),
                make_descriptor!(1 << 7),
                make_descriptor!(1 << 8),
                make_descriptor!(1 << 9),
                make_descriptor!(1 << 10),
            ],
        }
    }
}

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();
