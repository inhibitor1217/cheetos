/// Returns `&mut T` from `&mut Node<T>`, given the name of the [`Node`]'s
/// field of the struct `T`.
#[macro_export]
macro_rules! get_list_element {
    ($node:expr, $container:ty, $field:ident) => {
        unsafe {
            use $crate::{offset_of, utils::data_structures::linked_list};

            &mut *(($node as *mut linked_list::Node).offset(-offset_of!($container, $field))
                as *mut $container)
        }
    };
}

/// A type alias for a pointer to the [`Node`].
type Link = Option<core::ptr::NonNull<Node>>;

/// A doubly linked list.
///
/// This implementation of a doubly linked list does not require use of
/// dynamicaly allocated memory. Instead, it is instrusive, that is, a
/// potential list element must embed a [`Node`] member in its struct. All of
/// the list functions operate on [`CursorMut`]s, which can be accessed by the
/// references to the [`Node`]. The [`get_list_element`] macro allows conversion
/// from the [`Node`] member back to the struct that contains it.
///
/// The references returned from the list are `'static`, since the individual
/// nodes are not owned by the list itself. Typical usecases of the
/// [`LinkedList`] are when we are allocating the nodes where the kernel manages
/// it, so its lifetime can be extended indefinitely. The kernel is responsible
/// for allocating and freeing the memory for the nodes.
///
/// ## Example
///
/// Suppose there is a need for a list of `struct Foo`. `Foo` should
/// contain a [`Node`] member, like so:
///
/// ```rust
/// struct Foo {
///     Node elem,
///     // ...
/// }
/// ```
///
/// Then a list of `Foo` can be declared and initialized like so:
///
/// ```rust
/// let mut list = LinkedList<Foo>::new();
/// ```
///
/// Iteration is a typical situation where it is necessary to convert from a
/// [`Node`] back to its enclosing structure. [`LinkedList`] implements the
/// [`Iterator`] trait, so it can be easily done:
///
/// ```rust
/// for node in list {
///     let foo = get_list_element!(node, Foo, elem);
///     // ...
/// }
/// ```
#[derive(PartialEq, Eq)]
pub struct LinkedList<T> {
    head: Link,
    tail: Link,
    _marker: core::marker::PhantomData<T>,
}

/// An element in a list, which should be embeded to the parent structure.
///
/// The list is intrusive, in a sense that the node itself does not contain
/// the content. The nodes are allocated when the elements are allocated,
/// hence the list does not have the ownership over the elements.
#[derive(Debug)]
pub struct Node {
    prev: Link,
    next: Link,
}

/// A mutable cursor over the list, which we can use to insert, remove, or split
/// elements at the cursor.
#[derive(Debug)]
pub struct CursorMut<'a, T> {
    cur: Link,
    list: &'a mut LinkedList<T>,
}

impl<T> LinkedList<T> {
    /// Creates an empty [`LinkedList`].
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns the first node of the list, `None` if the list is empty.
    pub fn front(&self) -> Option<&'static Node> {
        unsafe { self.head.map(|head| &*head.as_ptr()) }
    }

    /// Returns the first node of the list, `None` if the list is empty.
    pub fn front_mut(&mut self) -> Option<&'static mut Node> {
        unsafe { self.head.map(|head| &mut *head.as_ptr()) }
    }

    /// Returns the last node of the list, `None` if the list is empty.
    pub fn back(&self) -> Option<&'static Node> {
        unsafe { self.tail.map(|tail| &*tail.as_ptr()) }
    }

    /// Returns the last node of the list, `None` if the list is empty.
    pub fn back_mut(&mut self) -> Option<&'static mut Node> {
        unsafe { self.tail.map(|tail| &mut *tail.as_ptr()) }
    }

    /// Returns `true` if the list is empty, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.head.is_some()
    }

    /// Clears the list so that it is empty.
    pub fn clear(&mut self) {
        self.head = None;
        self.tail = None;
    }

    /// Inserts a new node at the start of the list, so that the inserted node
    /// becomes the new head.
    pub fn push_front(&mut self, node: &'static mut Node) {
        assert!(!node.is_element());

        unsafe {
            let link = core::ptr::NonNull::new_unchecked(node as *mut Node);
            if let Some(head) = self.head {
                (*head.as_ptr()).prev = Some(link);
                (*link.as_ptr()).next = Some(head);
            } else {
                self.tail = Some(link);
            }
            self.head = Some(link);
        }
    }

    /// Inserts a new node at the end of the list, so that the inserted node
    /// becomes the new tail.
    pub fn push_back(&mut self, node: &'static mut Node) {
        assert!(!node.is_element());

        unsafe {
            let link = core::ptr::NonNull::new_unchecked(node as *mut Node);
            if let Some(tail) = self.tail {
                (*tail.as_ptr()).next = Some(link);
                (*link.as_ptr()).prev = Some(tail);
            } else {
                self.head = Some(link);
            }
            self.tail = Some(link);
        }
    }

    /// Removes a new node from the start of the list, returning the removed
    /// node. Returns `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<&'static mut Node> {
        unsafe {
            self.head.map(|head| {
                let node = &mut *head.as_ptr();

                self.head = node.next;
                if let Some(head) = self.head {
                    (*head.as_ptr()).prev = None;
                } else {
                    self.tail = None;
                }

                node.next = None;

                assert!(!node.is_element());
                node
            })
        }
    }

    /// Removes a new node from the back of the list, returning the removed
    /// node. Returns `None `if the list is empty.
    pub fn pop_back(&mut self) -> Option<&'static mut Node> {
        unsafe {
            self.tail.map(|tail| {
                let node = &mut *tail.as_ptr();

                self.tail = node.prev;
                if let Some(tail) = self.tail {
                    (*tail.as_ptr()).next = None;
                } else {
                    self.head = None;
                }

                node.prev = None;

                assert!(!node.is_element());
                node
            })
        }
    }

    /// Returns an iterator over the elements of the list.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.head,
            tail: self.tail,
            done: self.is_empty(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns a mutable iterator over the elements of the list.
    pub fn iter_mut(&self) -> IterMut<'_, T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            done: self.is_empty(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns a new cursor placed before the first element of the list.
    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            cur: None,
            list: self,
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Extend<&'static mut Node> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = &'static mut Node>>(&mut self, iter: I) {
        for node in iter {
            self.push_back(node)
        }
    }
}

impl<T> core::iter::FromIterator<&'static mut Node> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = &'static mut Node>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T> core::fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

pub struct Iter<'a, T> {
    head: Link,
    tail: Link,
    done: bool,
    _marker: core::marker::PhantomData<&'a T>,
}

pub struct IterMut<'a, T> {
    head: Link,
    tail: Link,
    done: bool,
    _marker: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iter<'a, T> {
    fn check_done(&mut self) {
        self.done = self.head == self.tail;
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'static Node;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'static Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let node = self.head.map(|head| unsafe {
                self.head = (*head.as_ptr()).next;
                &*head.as_ptr()
            });

            self.check_done();

            node
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let node = self.tail.map(|tail| unsafe {
                self.tail = (*tail.as_ptr()).prev;
                &*tail.as_ptr()
            });

            self.check_done();

            node
        }
    }
}

impl<'a, T> IterMut<'a, T> {
    fn check_done(&mut self) {
        self.done = self.head == self.tail;
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'static mut Node;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'static mut Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let node = self.head.map(|head| unsafe {
                self.head = (*head.as_ptr()).next;
                &mut *head.as_ptr()
            });

            self.check_done();

            node
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let node = self.tail.map(|tail| unsafe {
                self.tail = (*tail.as_ptr()).prev;
                &mut *tail.as_ptr()
            });

            self.check_done();

            node
        }
    }
}

impl Node {
    /// Creates a disconnected [`Node`].
    pub fn new() -> Self {
        Self {
            prev: None,
            next: None,
        }
    }

    /// Checks if this node is an element of some list.
    fn is_element(&self) -> bool {
        // Sanity check.
        assert!(self.prev.is_some() == self.next.is_some());

        self.prev.is_some()
    }

    /// Returns a cursor placed at this node.
    pub fn cursor_mut<'a, T>(&mut self, list: &'a mut LinkedList<T>) -> CursorMut<'a, T> {
        unsafe {
            CursorMut {
                cur: Some(core::ptr::NonNull::new_unchecked(self as *mut Node)),
                list,
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> CursorMut<'a, T> {
    /// Move the cursor to the next element.
    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).next;
            }
        } else {
            self.cur = self.list.head;
        }
    }

    /// Move the cursor to the previous element.
    pub fn move_prev(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).prev;
            }
        } else {
            self.cur = self.list.tail;
        }
    }

    /// Retrieves the node at the cursor.
    pub fn current(&mut self) -> Option<&'static mut Node> {
        unsafe { self.cur.map(|node| &mut *node.as_ptr()) }
    }

    /// Retrieves the node after the cursor.
    pub fn peek_next(&mut self) -> Option<&'static mut Node> {
        unsafe {
            self.cur
                .map_or_else(|| self.list.head, |node| (*node.as_ptr()).next)
                .map(|node| &mut *node.as_ptr())
        }
    }

    /// Retrieves the node before the cursor.
    pub fn peek_prev(&mut self) -> Option<&'static mut Node> {
        unsafe {
            self.cur
                .map_or_else(|| self.list.tail, |node| (*node.as_ptr()).prev)
                .map(|node| &mut *node.as_ptr())
        }
    }

    /// Removes the current element from the list, returning the node.
    /// The cursor will move to the next node.
    pub fn remove_current(&mut self) -> Option<&'static mut Node> {
        if let Some(cur) = self.cur {
            unsafe {
                let cur = cur.as_ptr();
                (*cur).prev.map(|prev| (*prev.as_ptr()).next = (*cur).next);
                (*cur).next.map(|next| (*next.as_ptr()).prev = (*cur).prev);

                if (*cur).prev.is_none() {
                    self.list.head = (*cur).next;
                }
                if (*cur).next.is_none() {
                    self.list.tail = (*cur).prev;
                }

                self.cur = (*cur).next;

                Some(&mut *cur)
            }
        } else {
            None
        }
    }

    /// Inserts `node` before the cursor.
    /// If the cursor was at the sentinel, the node is appended to the tail of
    /// list.
    pub fn insert_before(&mut self, node: &'static mut Node) {
        let mut list = LinkedList::new();
        list.push_back(node);
        self.splice_before(list);
    }

    /// Inserts `node` after the cursor.
    /// If the cursor was at the sentinel, the node is appended to the head of
    /// the list.
    pub fn insert_after(&mut self, node: &'static mut Node) {
        let mut list = LinkedList::new();
        list.push_back(node);
        self.splice_after(list);
    }

    /// Inserts elements from `list` before the cursor, and make `list` empty.
    /// If the cursor was at the sentinel, the nodes are appended to the tail of
    /// the list.
    pub fn splice_before(&mut self, mut list: LinkedList<T>) {
        if list.is_empty() {
            return;
        }

        unsafe {
            if let Some(cur) = self.cur {
                let prev = (*cur.as_ptr()).prev;

                if let Some(prev) = prev {
                    // Insert `list` between `prev` and `cur`.
                    (*prev.as_ptr()).next = list.head;
                    (*list.head.unwrap().as_ptr()).prev = Some(prev); // `list` should be nonempty.
                } else {
                    // `list` becomes the prefix of us.
                    self.list.head = list.head;
                }

                (*cur.as_ptr()).prev = list.tail;
                (*list.tail.unwrap().as_ptr()).next = Some(cur); // `list` should be nonempty.

                list.clear();
            } else if let Some(tail) = self.list.tail {
                // We are at the sentinel, so append the list to the tail.
                (*tail.as_ptr()).next = list.head;
                (*list.head.unwrap().as_ptr()).prev = Some(tail); // `list` should be nonempty.

                self.list.tail = list.tail;
                list.clear();
            } else {
                // We are empty, so we become the given list.
                *self.list = list;
            }
        }
    }

    /// Inserts elements from `list` after the cursor, and make `list` empty.
    /// If the cursor was at the sentinel, the nodes are appended to the head of
    /// the list.
    pub fn splice_after(&mut self, mut list: LinkedList<T>) {
        if list.is_empty() {
            return;
        }

        unsafe {
            if let Some(cur) = self.cur {
                let next = (*cur.as_ptr()).next;

                if let Some(next) = next {
                    // Insert `list` between `cur` and `next`.
                    (*next.as_ptr()).prev = list.tail;
                    (*list.tail.unwrap().as_ptr()).next = Some(next); // `list` should be nonempty.
                } else {
                    // `list` becomes the suffix of us.
                    self.list.tail = list.tail;
                }

                (*cur.as_ptr()).next = list.head;
                (*list.head.unwrap().as_ptr()).prev = Some(cur); // `list` should be nonempty.

                list.clear();
            } else if let Some(head) = self.list.head {
                // We are at the sentinel, so append the list to the head.
                (*head.as_ptr()).prev = list.tail;
                (*list.tail.unwrap().as_ptr()).next = Some(head); // `list` should be nonempty.

                self.list.head = list.head;
                list.clear();
            } else {
                // We are empty, so we become the given list.
                *self.list = list;
            }
        }
    }

    /// Creates a new [`LinkedList`] by splitting the list before the cursor,
    /// returning the newly created list.
    /// The cursor will remain at the current position.
    pub fn split_before(&mut self) -> LinkedList<T> {
        todo!()
    }

    /// Creates a new [`LinkedList`] by splitting the list after the cursor,
    /// returning the newly created list.
    /// The cursor will remain at the current position.
    pub fn split_after(&mut self) -> LinkedList<T> {
        todo!()
    }
}
