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
    _marker: core::marker::PhantomData<T>,
}

impl<T> LinkedList<T> {
    /// Creates a new, empty [`LinkedList`].
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    /// Provides a forward iterator.
    pub fn iter(&self) -> Iter<T> {
        todo!()
    }

    /// Returns `true` if the [`LinkedList`] is empty.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Returns the number of [`Node`]s in the [`LinkedList`].
    /// Runs in O(n) in the number of [`Node`]s.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Returns a reference to the first [`Node`] of the [`LinkedList`], or
    /// `None` if it is empty.
    pub fn front(&self) -> Option<&'static mut Node<T>> {
        todo!()
    }

    /// Returns a reference to the last [`Node`] of the [`LinkedList`], or
    /// `None` if it is empty.
    pub fn back(&self) -> Option<&'static mut Node<T>> {
        todo!()
    }

    /// Inserts a [`Node`] at the front of the [`LinkedList`].
    pub fn push_front(&mut self, _node: &'static mut Node<T>) {
        todo!()
    }

    /// Inserts a [`Node`] at the back of the [`LinkedList`].
    pub fn push_back(&mut self, _node: &'static mut Node<T>) {
        todo!()
    }

    /// Removes the first [`Node`] from the [`LinkedList`] and returns it, or
    /// `None` if it is empty.
    pub fn pop_front(&mut self) -> Option<&'static mut Node<T>> {
        todo!()
    }

    /// Removes the last [`Node`] from the [`LinkedList`] and returns it, or
    /// `None` if it is empty.
    pub fn pop_back(&mut self) -> Option<&'static mut Node<T>> {
        todo!()
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
    pub fn insert_ordered(&mut self, _node: &'static mut Node<T>) {
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
    type Item = Node<T>;
    type IntoIter = Iter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Node<T> {
    _marker: core::marker::PhantomData<T>,
}

impl<T> Node<T> {
    /// Creates a new [`Node`].
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    /// Inserts `node` after `self`.
    pub fn insert(&mut self, _node: &'static mut Node<T>) {
        todo!()
    }

    /// Removes elements `from` through `last` (exclusive) from their current
    /// list, then inserts them after `self`.
    pub fn insert_slice(&mut self, _from: &'static mut Node<T>, _to: &'static mut Node<T>) {
        todo!()
    }

    /// Removes `self` from its current list.
    pub fn remove(&mut self) {
        todo!()
    }
}

impl<T> core::ops::Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<T> core::ops::DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
    }
}

macro_rules! get_element {
    () => {};
}

/// An iterator over the elements of a [`LinkedList`].
///
/// There are no distinguish between `Iter` and `IterMut` for
/// [`LinkedList`]: since the list does not own the nodes, and the nodes are
/// meant to be statically allocated, the user should ensure the list elements
/// are properly synchronized.
#[derive(Debug)]
pub struct Iter<T> {
    _marker: core::marker::PhantomData<T>,
}

impl<T> core::iter::Iterator for Iter<T> {
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
