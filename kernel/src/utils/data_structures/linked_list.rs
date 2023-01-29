/// A doubly linked list.
///
/// This implementation of a doubly linked list is not intended to be used as
/// an owned structure. It also does not require use of dynamically allocated
/// memory. Rather, it is intended to be used as a static data structure guarded
/// with a synchonization mechanism.
///
/// Each structure that is a potential list element must embed a [`Node`] as a
/// member. All of the list functions operate on the [`Node`] structure. The
/// [`get_element!`] macro allows conversion from `&mut Node` to `&mut T`, where
/// `T` is the type of the structure that embeds the [`Node`].
///
/// # Example
///
/// Suppose there is a needed for a [`LinkedList`] of `Foo`. `Foo` must contain
/// a [`Node`] as a member, like so:
///
/// ```rust
/// struct Foo {
///    list_node: Node,
///   // ...
/// }
/// ```
///
/// Then a [`LinkedList`] can be created and used like so:
///
/// ```rust
/// let list = LinkedList::new();
/// ```
///
/// [`LinkedList`] is an [`Iterator`], so it can be used in a `for` loop:
///
/// ```rust
/// for node in list.iter() {
///    let foo = get_element!(node, Foo, list_node);
///    // ...
/// }
/// ```
#[derive(Debug)]
pub struct LinkedList<T> {
    front: Node,
    back: Node,
    _marker: core::marker::PhantomData<T>,
}

impl<T> LinkedList<T> {
    /// Creates a new, uninitialized [`LinkedList`].
    #[must_use = "initializing a `LinkedList` does nothing without `.init()`"]
    pub const fn new() -> Self {
        Self {
            front: Node::new(),
            back: Node::new(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Initializes the [`LinkedList`].
    ///
    /// # Safety
    /// This function is unsafe because the list must exist in a static
    /// location. This is because the internal pointers of the list is
    /// self-referential. Also, this function must only be called once.
    pub unsafe fn init(&mut self) {
        self.front.next = Some(&mut self.back);
        self.back.prev = Some(&mut self.front);
    }

    /// Provides a forward iterator.
    pub fn iter(&mut self) -> Iter {
        todo!()
    }

    /// Returns `true` if the [`LinkedList`] is empty.
    pub fn is_empty(&mut self) -> bool {
        self.front().is_none()
    }

    /// Returns the number of [`Node`]s in the [`LinkedList`].
    /// Runs in O(n) in the number of [`Node`]s.
    pub fn len(&mut self) -> usize {
        todo!()
    }

    /// Returns `&mut` to the first [`Node`] of the [`LinkedList`], or
    /// `None` if it is empty.
    pub fn front(&mut self) -> Option<&'static mut Node> {
        let first = self
            .front
            .next
            .expect("front node should always have a next node");
        if first == &mut self.back {
            None
        } else {
            Some(unsafe { &mut *first })
        }
    }

    /// Returns `&mut` to the last [`Node`] of the [`LinkedList`], or
    /// `None` if it is empty.
    pub fn back(&mut self) -> Option<&'static mut Node> {
        let last = self
            .back
            .prev
            .expect("back node should always have a previous node");
        if last == &mut self.front {
            None
        } else {
            Some(unsafe { &mut *last })
        }
    }

    /// Inserts a [`Node`] at the front of the [`LinkedList`].
    pub fn push_front(&mut self, node: &'static mut Node) {
        self.front.insert(node);
    }

    /// Inserts a [`Node`] at the back of the [`LinkedList`].
    pub fn push_back(&mut self, node: &'static mut Node) {
        self.back.insert_before(node);
    }

    /// Removes the first [`Node`] from the [`LinkedList`] and returns it, or
    /// `None` if it is empty.
    pub fn pop_front(&mut self) -> Option<&'static mut Node> {
        self.front().map(Node::remove)
    }

    /// Removes the last [`Node`] from the [`LinkedList`] and returns it, or
    /// `None` if it is empty.
    pub fn pop_back(&mut self) -> Option<&'static mut Node> {
        self.back().map(Node::remove)
    }
}

impl<T> LinkedList<T>
where
    T: Ord,
{
    /// Sorts the [`LinkedList`] in place.
    pub fn sort(&mut self) {
        todo!()
    }

    /// Inserts `node` in the proper position in `self` to maintain ordering.
    pub fn insert_ordered(&mut self, _node: &'static mut Node) {
        todo!()
    }

    /// Iterates through `self` and removes all but first in each set of
    /// adjacent elements that are equal each other.
    pub fn remove_duplicates(&mut self) {
        todo!()
    }

    /// Iterates through `self` and removes all but first in each set of
    /// adjacent elements that are equal each other. The removed elements are
    /// appended to `duplicates`.
    pub fn filter_duplicates(&mut self, _duplicates: &mut LinkedList<T>) {
        todo!()
    }
}

impl<T> core::iter::IntoIterator for LinkedList<T> {
    type Item = Node;
    type IntoIter = Iter;

    fn into_iter(mut self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Node {
    prev: Option<*mut Node>,
    next: Option<*mut Node>,
}

impl Node {
    /// Creates a new [`Node`].
    pub const fn new() -> Self {
        Self {
            prev: None,
            next: None,
        }
    }

    /// Inserts `node` after `self`.
    pub fn insert(&mut self, node: &mut Node) {
        // Cannot insert after back node.
        assert!(!self.is_back());

        node.prev = Some(self);
        node.next = self.next;
        unsafe {
            (*self.next.expect("Cannot insert after back node of list")).prev = Some(node);
        }
        self.next = Some(node);
    }

    /// Inserts `node` before `self`.
    pub fn insert_before(&mut self, node: &mut Node) {
        // Cannot insert before front node.
        assert!(!self.is_front());

        node.prev = self.prev;
        node.next = Some(self);
        unsafe {
            (*self.prev.expect("Cannot insert before front node of list")).next = Some(node);
        }
        self.prev = Some(node);
    }

    /// Removes elements `from` through `last` (exclusive) from their current
    /// list, then inserts them after `self`.
    pub fn insert_slice(&mut self, _from: &mut Node, _to: &mut Node) {
        todo!()
    }

    /// Removes `self` from its current list, then returns itself.
    pub fn remove(&mut self) -> &mut Self {
        // Cannot remove the front or back node.
        assert!(self.is_interior());

        unsafe {
            (*self.prev.expect("Cannot remove front node of list")).next = self.next;
            (*self.next.expect("Cannot remove back node of list")).prev = self.prev;
        }

        self.prev = None;
        self.next = None;

        self
    }

    /// Returns `true` if `self` is the first [`Node`] in its list.
    fn is_front(&self) -> bool {
        self.prev.is_none() && self.next.is_some()
    }

    /// Returns `true` if `self` is an interior [`Node`] in its list.
    fn is_interior(&self) -> bool {
        self.prev.is_some() && self.next.is_some()
    }

    /// Returns `true` if `self` is the last [`Node`] in its list.
    fn is_back(&self) -> bool {
        self.prev.is_some() && self.next.is_none()
    }
}

/// Returns `&mut T` from `&mut Node<T>`, given the name of the [`Node`]'s
/// field of the struct `T`.
#[macro_export]
macro_rules! get_element {
    ($node:expr, $container:ty, $field:ident) => {
        unsafe {
            use crate::{offset_of, utils::data_structures::linked_list};

            &mut *(($node as *mut linked_list::Node).offset(-offset_of!($container, $field))
                as *mut $container)
        }
    };
}

/// An iterator over the elements of a [`LinkedList`].
///
/// There are no distinguish between `Iter` and `IterMut` for
/// [`LinkedList`]: since the list does not own the nodes, and the nodes are
/// meant to be statically allocated, the user should ensure the list elements
/// are properly synchronized.
#[derive(Debug)]
pub struct Iter {}

impl core::iter::Iterator for Iter {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
