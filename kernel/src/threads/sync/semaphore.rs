use crate::{
    get_list_element,
    threads::{
        interrupt,
        scheduler::SCHEDULER,
        thread::{current_thread, Thread},
    },
    utils::data_structures::linked_list::LinkedList,
};

/// A counting semaphore.
///
/// A semaphore is a nonnegative integer along with two atomic operations for
/// manipulating it:
///
/// - down or "P": wait for the value to become positive, then decrement it.
/// - up or "V": increment the value (and wake up a waiting thread, if any).
#[derive(Debug)]
pub struct Semaphore {
    inner: interrupt::Mutex<Inner>,
}

impl Semaphore {
    /// Creates a new, uninitialized [`Semaphore`] with `value`.
    pub const fn new(value: usize) -> Self {
        Self {
            inner: interrupt::Mutex::new(Inner::new(value)),
        }
    }

    /// Down or "P" operation on a [`Semaphore`]. Waits for `self`'s value to
    /// become positive and then atomically decrements it.
    ///
    /// This function may sleep, so it must not be called within an interrupt
    /// handler. This function may be called with interrupts disabled, but if
    /// it sleeps then the next scheduled thread will probably turn interrupts
    /// back on.
    pub fn down(&self) {
        assert!(!interrupt::is_external_handler_context());
        self.inner.lock().down();
    }

    /// Down or "P" operation on a [`Semaphore`], but only if the value is not
    /// already 0. Returns `true` if the value was decremented, `false`
    /// otherwise.
    ///
    /// This function may be called from an interrupt handler.
    pub fn try_down(&self) -> bool {
        self.inner.lock().try_down()
    }

    /// Up or "V" operation on a semaphore. Increments the value and wakes up
    /// one thread of those waiting for `self`.
    ///
    /// This function may be called from an interrupt handler.
    pub fn up(&self) {
        self.inner.lock().up();
    }
}

/// We can share [`Semaphore`]s between multiple threads, since it is protected
/// by a interrupt mutex.
unsafe impl Send for Semaphore {}

/// Internal structure of a [`Semaphore`].
///
/// Should be guarded with a mutex to ensure atomicity.
#[derive(Debug)]
struct Inner {
    value: usize,
    waiters: LinkedList<Thread>,
}

impl Inner {
    const fn new(value: usize) -> Self {
        Self {
            value,
            waiters: LinkedList::new(),
        }
    }

    fn down(&mut self) {
        while self.value == 0 {
            self.waiters.push_back(&mut current_thread().sync_node);
            SCHEDULER.lock().block_current_thread();
        }

        self.value -= 1;
    }

    fn try_down(&mut self) -> bool {
        if self.value > 0 {
            self.value -= 1;
            true
        } else {
            false
        }
    }

    fn up(&mut self) {
        if let Some(node) = self.waiters.pop_front() {
            SCHEDULER
                .lock()
                .unblock(get_list_element!(node, Thread, sync_node));
        }
        self.value += 1;
    }
}
