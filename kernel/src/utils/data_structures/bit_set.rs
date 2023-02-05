use core::{mem, ptr};

use crate::{div_round_up, offset_of};

type Element = u8;

/// Number of bits in an element.
const ELEMENT_BITS: usize = mem::size_of::<Element>() * 8;

/// Returns the number of [`Element`]s required for `bit_count` bits.
fn get_element_count(bit_count: usize) -> usize {
    div_round_up!(bit_count, ELEMENT_BITS)
}

/// Returns the number of bytes required for `bit_count` bits.
fn get_byte_count(bit_count: usize) -> usize {
    get_element_count(bit_count) * mem::size_of::<Element>()
}

#[derive(Debug)]
#[repr(C)]
pub struct BitSet {
    /// Number of bits in this collection.
    cap: usize,

    //// Elements that represet bits.
    bits: *mut Element,
}

impl BitSet {
    /// Creates an empty [`BitSet`].
    pub const fn new() -> Self {
        Self {
            cap: 0,
            bits: ptr::null_mut(),
        }
    }

    /// Creates a [`BitSet`] with capacity `cap` bits in the `buf_size` bytes
    /// of storage preallocated at `buf`. `buf` must be at least the size which
    /// [`BitSet`] requires.
    ///
    /// This function returns a `&'static mut` to the [`BitSet`], meaning that
    /// the caller should manage to free the allocated buffer.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure `buf` is properly
    /// allocated with size larger than `buf_size`.
    pub unsafe fn from_buffer(cap: usize, buf: *mut u8, buf_size: usize) -> &'static mut BitSet {
        assert!(
            buf_size >= Self::buffer_size(cap),
            "Not enough memory for bitmap."
        );

        let bit_set = buf as *mut BitSet;
        (*bit_set).cap = cap;
        (*bit_set).bits = buf.offset(offset_of!(BitSet, bits));

        &mut (*bit_set)
    }

    /// Returns the number of bytes required to accomodate a bitmap with `cap`
    /// bits (for use with `from_buffer`.)
    pub fn buffer_size(cap: usize) -> usize {
        mem::size_of::<BitSet>() + get_byte_count(cap)
    }
}
