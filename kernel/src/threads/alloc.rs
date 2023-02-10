use core::ops::DerefMut;

use crate::get_list_element;
use crate::utils::data_structures::linked_list::{LinkedList, Node};

use super::addr::{Page, VirtAddr, PAGE_SIZE};
use super::palloc::{AllocateFlags, PAGE_ALLOCATOR};
use super::sync::lock::Mutex;

/// Free block.
#[derive(Debug)]
#[repr(C)]
struct Block {
    node: Node,
}

impl Block {
    /// Returns the [`Arena`] which this block lies on.
    fn arena(&mut self) -> &'static mut Arena {
        let page = Page::containing_address(VirtAddr::from_ptr(self));

        unsafe { &mut *page.start_address().as_mut_ptr::<Arena>() }
    }
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
#[repr(C)]
struct Arena {
    /// Always set to `Arena::MAGIC` for detecting corruption.
    magic: u32,

    /// The link to the descriptor which owns this arena.
    /// Set to `None` for large blocks.
    descriptor: Option<core::ptr::NonNull<Descriptor>>,

    /// Free blocks; pages in big block.
    free_cnt: usize,
}

impl Arena {
    /// Magic number for detecting arena corruption.
    const MAGIC: u32 = 0x8a547eed;

    /// Initializes a new [`Arena`] at the start of an allocated `page`,
    /// using the given `descriptor`.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure `page` is
    /// already allocated and not already in use.
    unsafe fn from_descriptor(page: Page, descriptor: *mut Descriptor) -> &'static mut Self {
        let arena: *mut Arena = page.start_address().as_mut_ptr();
        let arena = &mut (*arena);

        arena.magic = Arena::MAGIC;
        arena.descriptor = Some(core::ptr::NonNull::new_unchecked(descriptor));
        arena.free_cnt = (*descriptor).blocks_per_arena;

        arena
    }

    /// Returns the [`Block`]s in this arena.
    fn blocks(&mut self) -> ArenaBlocksIterMut {
        assert!(self.is_arena());

        ArenaBlocksIterMut {
            arena: self,
            index: 0,
        }
    }

    /// Notify that a block has been allocated from this arena.
    fn allocate_block(&mut self) {
        assert!(self.free_cnt > 0);
        self.free_cnt -= 1;
    }

    /// Sanity check.
    fn is_arena(&self) -> bool {
        self.magic == Self::MAGIC
    }
}

struct ArenaBlocksIterMut<'a> {
    arena: &'a mut Arena,
    index: usize,
}

impl<'a> Iterator for ArenaBlocksIterMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        let descriptor = unsafe {
            &(*self
                .arena
                .descriptor
                .expect("Arena with large blocks, cannot iterate")
                .as_ptr())
        };

        if self.index >= descriptor.blocks_per_arena {
            None
        } else {
            let block = unsafe {
                &mut *((self.arena as *mut Arena)
                    .cast::<u8>()
                    .add(core::mem::size_of::<Arena>())
                    .add(self.index * descriptor.block_size))
                .cast::<Block>()
            };
            self.index += 1;

            Some(block)
        }
    }
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

    const DESCRIPTOR_BLOCK_SIZE: [usize; Allocator::DESCRIPTORS_SIZE] =
        [1 << 4, 1 << 5, 1 << 6, 1 << 7, 1 << 8, 1 << 9, 1 << 10];

    /// Creates a new allocator.
    pub const fn new() -> Self {
        Self {
            descriptors: [
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[0]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[1]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[2]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[3]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[4]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[5]),
                make_descriptor!(Self::DESCRIPTOR_BLOCK_SIZE[6]),
            ],
        }
    }

    /// Finds a descriptor which is suitable for allocating block of `size`.
    fn get_descriptor(&self, size: usize) -> Option<&Mutex<Descriptor>> {
        (0..Self::DESCRIPTORS_SIZE)
            .find(|&i| size <= Self::DESCRIPTOR_BLOCK_SIZE[i])
            .map(|i| &self.descriptors[i])
    }
}

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let required_block_size = layout.size().max(layout.align());

        if let Some(descriptor) = self.get_descriptor(required_block_size) {
            let mut descriptor = descriptor.lock();

            // If the free list is empty, create a new `Arena`.
            if descriptor.free_list.is_empty() {
                // Allocate a new page.
                if let Some(page) = PAGE_ALLOCATOR.get_page(AllocateFlags::empty()) {
                    // Initialize an `Arena` and add its blocks to the free list.
                    let arena = Arena::from_descriptor(page, descriptor.deref_mut());
                    for block in arena.blocks() {
                        descriptor.free_list.push_back(&mut block.node);
                    }
                } else {
                    return core::ptr::null_mut();
                }
            }

            // Get a block from free list and return it.
            if let Some(node) = descriptor.free_list.pop_front() {
                let block = get_list_element!(node, Block, node);
                block.arena().allocate_block();
                (block as *mut Block).cast::<u8>()
            } else {
                core::ptr::null_mut()
            }
        } else {
            // `required_block_size` is too big for any descriptor.
            // Allocate enough pages to hold` required_block_size` plus an
            // arena.

            todo!()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();
