use crate::threads::{interrupt, thread};

use super::semaphore::Semaphore;

/// A lock can be held at most a single thread at any given time. Our locks are
/// not "recursive", that is, it is an error for the thread currently holding
/// a lock to try to acquire that lock.
///
/// A lock is a specialization of a sempahore with an initial value of 1. The
/// difference between a lock and such a semaphore is twofold. First, a
/// semaphore can have a value greater than 1, but a lock can only be owned by
/// a single thread at a time. Second, a semaphore does not have an owner,
/// meaning that one thread can "down" a semaphore and then another one "up" it,
/// but with a lock the same thread must both acquire and release it. When these
/// restrictions prove onernous, it's a good sign that a semaphore should be
/// used, instead of a lock.
#[derive(Debug)]
pub struct Lock {
    /// Thread holding the lock (for debugging).
    holder: Option<*mut thread::Thread>,

    /// Binary semaphore controlling access.
    semaphore: Semaphore,
}

impl Lock {
    /// Creates a new [`Lock`].
    pub fn new() -> Self {
        Self {
            holder: None,
            semaphore: Semaphore::new(1),
        }
    }

    /// Initializes the [`Lock`].
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that the lock is
    /// in a static location. Also, this function must only be called once.
    pub unsafe fn init(&mut self) {
        self.semaphore.init();
    }

    /// Acquires the lock, sleeping until it becomes available if necessary.
    /// The lock must not be held by the current thread.
    ///
    /// This function may sleep, so it must not be called within an interrupt
    /// handler. This function may be called with interrupts disabled, but
    /// interrupts will be turned back on if we need to sleep.
    pub fn acquire(&mut self) {
        assert!(!interrupt::is_external_handler_context());
        assert!(!self.is_acquired_by_current_thread());

        self.semaphore.down();
        self.holder = Some(thread::current_thread());
    }

    /// Tries to acquire the lock and returns `true` if successful or `false`
    /// on failure. The lock must not be held by the current thread.
    ///
    /// This function will not sleep, so it may be called within an interrupt
    /// handler.
    pub fn try_acquire(&mut self) -> bool {
        assert!(!self.is_acquired_by_current_thread());

        if self.semaphore.try_down() {
            self.holder = Some(thread::current_thread());
            true
        } else {
            false
        }
    }

    /// Releases the lock, which must be held by the current thread.
    ///
    /// An interrupt handler cannot acquire a lock, so it does not make sense to
    /// try to release a lock within an interrupt handler.
    pub fn release(&mut self) {
        assert!(self.is_acquired_by_current_thread());

        self.holder = None;
        self.semaphore.up();
    }

    /// Returns `true` if the current thread holds the lock, `false` otherwise.
    fn is_acquired_by_current_thread(&self) -> bool {
        return self.holder == Some(thread::current_thread());
    }
}
