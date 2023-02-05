use core::{
    mem,
    ops::{Index, IndexMut},
    ptr,
    sync::atomic::Ordering,
};

use crate::{div_round_up, offset_of};

type RawElement = u32;
type Element = core::sync::atomic::AtomicU32;

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

/// Returns the index of [`Element`] that contains the bit at `bit_index`.
fn get_element_index(bit_index: usize) -> usize {
    bit_index / ELEMENT_BITS
}

/// Returns an [`RawElement`] where only bit corresponding to `bit_index` is
/// turned on.
fn mask(bit_index: usize) -> RawElement {
    1 << (bit_index % ELEMENT_BITS)
}

#[derive(Debug)]
#[repr(C)]
pub struct BitSet {
    /// Number of bits in this collection.
    cap: usize,

    //// Elements that represet bits.
    elements: *mut Element,
}

impl BitSet {
    /// Creates an empty [`BitSet`].
    pub const fn new() -> Self {
        Self {
            cap: 0,
            elements: ptr::null_mut(),
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

        let buf = buf as *mut Element;
        let bit_set = buf as *mut BitSet;
        (*bit_set).cap = cap;
        (*bit_set).elements = buf.offset(offset_of!(BitSet, elements));

        let bit_set = &mut (*bit_set);

        bit_set.set_all(false);

        bit_set
    }

    /// Returns the number of bytes required to accomodate a bitmap with `cap`
    /// bits (for use with `from_buffer`.)
    pub fn buffer_size(cap: usize) -> usize {
        mem::size_of::<BitSet>() + get_byte_count(cap)
    }

    /// Returns the number of bits.
    pub fn size(&self) -> usize {
        self.cap
    }

    /// Returns the value of the bit at `index`.
    pub fn get(&self, index: usize) -> bool {
        self[get_element_index(index)].load(Ordering::Relaxed) & mask(index) != 0
    }

    /// Atomically sets the bit at `index` to `true`.
    pub fn mark(&mut self, index: usize) {
        self[get_element_index(index)].fetch_or(mask(index), Ordering::Relaxed);
    }

    /// Atomically sets the bit at `index` to `false`.
    pub fn reset(&mut self, index: usize) {
        self[get_element_index(index)].fetch_and(!mask(index), Ordering::Relaxed);
    }

    /// Atomially sets the bit at `index` to `value`.
    pub fn set(&mut self, index: usize, value: bool) {
        if value {
            self.mark(index);
        } else {
            self.reset(index);
        }
    }

    /// Atomically toggles the bit at `index`.
    pub fn flip(&mut self, index: usize) {
        self[get_element_index(index)].fetch_xor(mask(index), Ordering::Relaxed);
    }

    /// Sets all bits to `value`.
    pub fn set_all(&mut self, value: bool) {
        self.set_many(0, self.size(), value);
    }

    /// Sets `count` bits starting from `start` to `value`.
    pub fn set_many(&mut self, start: usize, count: usize, value: bool) {
        for i in start..start + count {
            self.set(i, value);
        }
    }

    /// Finds and returns the starting index of the first group of `count`
    /// consecutive bits at or after `start` that are all set to `value`.
    ///
    /// If there is no such group, returns `None`.
    pub fn scan(&self, start: usize, count: usize, value: bool) -> Option<usize> {
        (start..self.cap - count).find(|i| !self.contains(*i, count, !value))
    }

    /// Returns `true` if any bits in `start..start + count` range are set to
    /// `value`.
    fn contains(&self, start: usize, count: usize, value: bool) -> bool {
        (start..start + count).any(|i| self.get(i) == value)
    }
}

impl Index<usize> for BitSet {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.cap);
        unsafe { &(*self.elements.add(index)) }
    }
}

impl IndexMut<usize> for BitSet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.cap);
        unsafe { &mut (*self.elements.add(index)) }
    }
}
