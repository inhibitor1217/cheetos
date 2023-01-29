/// Returns `true` if interrupts are enabled, and `false` otherwise.
pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

/// Returns `true` if interrupts are disabled, and `false` otherwise.
pub fn are_disabled() -> bool {
    !are_enabled()
}

/// Enables interrupts.
pub fn enable() {
    assert!(!is_external_handler_context());

    x86_64::instructions::interrupts::enable();
}

/// Disables interrupts.
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

/// Evaluates the given expression with interrupts disabled.
#[macro_export]
macro_rules! without_interrupts {
    ($body:expr) => {{
        use $crate::threads::interrupt;

        let prev_enabled = interrupt::are_enabled();
        interrupt::disable();

        let result = $body;

        if prev_enabled {
            interrupt::enable();
        }

        result
    }};
}

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
        match self.prev_enabled {
            Some(true) => enable(),
            _ => {}
        }
    }
}

/// Returns `true` during processing of an external interrupt, and `false` at
/// all other times.
///
/// TODO: implement this properly after we implement interrupts.
pub fn is_external_handler_context() -> bool {
    false
}
