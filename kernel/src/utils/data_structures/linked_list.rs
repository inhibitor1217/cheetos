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

/// A type alias for a pointer to the [`Node`].
type Link = Option<core::ptr::NonNull<Node>>;

/// A doubly linked list.
///
/// This implementation of a doubly linked list does not require use of
/// dynamicaly allocated memory. Instead, it is instrusive, that is, a
/// potential list element must embed a [`Node`] member in its struct. All of
/// the list functions operate on [`CursorMut`]s, which can be accessed by the
/// references to the [`Node`]. The [`get_element`] macro allows conversion from
/// the [`Node`] member back to the struct that contains it.
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
///     let foo = get_element!(node, Foo, elem);
///     // ...
/// }
/// ```
#[derive(Debug)]
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
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns the first node of the list, `None` if the list is empty.
    pub fn front(&self) -> Option<&Node> {
        unsafe { self.head.map(|head| &*head.as_ptr()) }
    }

    /// Returns the first node of the list, `None` if the list is empty.
    pub fn front_mut(&mut self) -> Option<&mut Node> {
        unsafe { self.head.map(|head| &mut *head.as_ptr()) }
    }

    /// Returns the last node of the list, `None` if the list is empty.
    pub fn back(&self) -> Option<&Node> {
        unsafe { self.tail.map(|tail| &*tail.as_ptr()) }
    }

    /// Returns the last node of the list, `None` if the list is empty.
    pub fn back_mut(&mut self) -> Option<&mut Node> {
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
}

impl Node {
    /// Creates a disconnected [`Node`].
    pub fn new() -> Self {
        Self {
            prev: None,
            next: None,
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
