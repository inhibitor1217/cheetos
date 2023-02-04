use super::control::{are_enabled, disable, enable};

/// A mutual exclusion primitive used for protecting shared data.
///
/// This mutex will disable interrupts when the mutable reference is accessed.
/// This ensures that no other thread can access the data while the mutex
/// is locked.
#[derive(Debug)]
pub struct Mutex<T> {
    data: core::cell::UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new [`Mutex`].
    pub const fn new(data: T) -> Self {
        Self {
            data: core::cell::UnsafeCell::new(data),
        }
    }

    /// Returns a read-only reference without locking.
    pub fn peek(&self) -> &T {
        unsafe { &(*self.data.get()) }
    }

    /// Locks the mutex and returns a mutable reference to the inner data.
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard::new(self)
    }
}

/// [`Mutex`] is `Sync` because protects the inner data with disabling
/// interrupts.
unsafe impl<T> core::marker::Sync for Mutex<T> {}

/// An RAII guard of a critical section using interrupts.
///
/// When the mutable reference is accessed, interrupts are disabled.
/// This ensures that no other thread can access the data while the
/// mutable reference is alive at the scope. When the mutable reference
/// is dropped, interrupts are restored to the previous state.
#[derive(Debug)]
pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
    prev_enabled: Option<bool>,
}

impl<'a, T> MutexGuard<'a, T> {
    /// Creates a new [`MutexGuard`].
    pub const fn new(lock: &'a Mutex<T>) -> Self {
        Self {
            lock,
            prev_enabled: None,
        }
    }
}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // No need to disable interrupts here.
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.prev_enabled = Some(are_enabled());
        disable();
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> core::ops::Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(true) = self.prev_enabled {
            enable();
        }
    }
}
