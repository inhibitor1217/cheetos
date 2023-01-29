use core::cell::UnsafeCell;

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
    /// Binary semaphore controlling access.
    semaphore: Semaphore,
}

impl Lock {
    /// Creates a new [`Lock`].
    #[must_use = "initializing a `Lock` does nothing without `.init()`"]
    pub const fn new() -> Self {
        Self {
            semaphore: Semaphore::new(1),
        }
    }

    /// Initializes the [`Lock`].
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that the lock is
    /// in a static location. Also, this function must only be called once.
    pub unsafe fn init(&self) {
        self.semaphore.init();
    }

    /// Acquires the lock, sleeping until it becomes available if necessary.
    ///
    /// This function may sleep, so it must not be called within an interrupt
    /// handler. This function may be called with interrupts disabled, but
    /// interrupts will be turned back on if we need to sleep.
    pub fn acquire(&self) {
        assert!(!interrupt::is_external_handler_context());

        self.semaphore.down();
    }

    /// Tries to acquire the lock and returns `true` if successful or `false`
    /// on failure.
    ///
    /// This function will not sleep, so it may be called within an interrupt
    /// handler.
    pub fn try_acquire(&self) -> bool {
        self.semaphore.try_down()
    }

    /// Releases the lock.
    ///
    /// An interrupt handler cannot acquire a lock, so it does not make sense to
    /// try to release a lock within an interrupt handler.
    pub fn release(&self) {
        self.semaphore.up();
    }
}

/// The data protected by a [`Mutex`], with some metadata.
struct MutexData<T> {
    value: T,
    holder: Option<*mut thread::Thread>,
}

/// A mutual exclusion primitive used for protecting shared data,
/// implemented using a [`Lock`].
#[derive(Debug)]
pub struct Mutex<T> {
    /// The data protected by the mutex.
    data: UnsafeCell<MutexData<T>>,

    /// The underlying lock.
    lock: Lock,
}

impl<T> Mutex<T> {
    /// Creates a new [`Mutex`].
    #[must_use = "initializing a `Mutex` does nothing without `.init()`"]
    pub const fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(MutexData {
                value,
                holder: None,
            }),
            lock: Lock::new(),
        }
    }

    /// Initializes the [`Mutex`].
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that the mutex is
    /// in a static location. Also, this function must only be called once.
    pub unsafe fn init(&self) {
        self.lock.init();
    }

    /// Returns a guard which locks the mutex when accessed a mutable reference,
    /// and unlocks the mutex when the reference is dropped.
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard::new(self)
    }
}

/// [`Mutex`] is [`Sync`] because the underlying mutable data is protected by a
/// [`Lock`].
unsafe impl<T> Sync for Mutex<T> {}

/// An RAII guard of a critical section protected by a [`Lock`].
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> MutexGuard<'a, T> {
    /// Creates a new [`MutexGuard`].
    fn new(mutex: &'a Mutex<T>) -> Self {
        Self { mutex }
    }
}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let Mutex { data, .. } = self.mutex;

        // No need to acquire the lock here.
        let data = unsafe { &*data.get() };
        &data.value
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Mutex { lock, data } = self.mutex;

        // Start of a critical section.
        lock.acquire();

        let data = unsafe { &mut *data.get() };
        data.holder = Some(thread::current_thread());
        &mut data.value
    }
}

impl<'a, T> core::ops::Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        let Mutex { lock, data } = self.mutex;

        let data = unsafe { &mut *data.get() };
        let holder = data.holder;
        data.holder = None;

        // End of a critical section.
        if holder == Some(thread::current_thread()) {
            lock.release();
        }
    }
}
