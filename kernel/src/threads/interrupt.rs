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

/// An RAII guard of a critical section using interrupts.
///
/// When the mutable reference is accessed, interrupts are disabled.
/// This ensures that no other thread can access the data while the
/// mutable reference is alive at the scope. When the mutable reference
/// is dropped, interrupts are restored to the previous state.
#[must_use = "if unused InterruptGuard will immediately restore interrupts"]
pub struct InterruptGuard<T> {
    data: core::cell::UnsafeCell<T>,
    prev_enabled: Option<bool>,
}

impl<T> InterruptGuard<T> {
    /// Creates a new [`InterruptGuard`].
    pub const fn new(data: T) -> Self {
        Self {
            data: core::cell::UnsafeCell::new(data),
            prev_enabled: None,
        }
    }
}

/// [`InterruptGuard`] is `Sync` because protects the inner data with disabling
/// interrupts.
unsafe impl<T> core::marker::Sync for InterruptGuard<T> {}

impl<T> core::ops::Deref for InterruptGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // No need to disable interrupts here.
        unsafe { &*self.data.get() }
    }
}

impl<T> core::ops::DerefMut for InterruptGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.prev_enabled = Some(are_enabled());
        disable();
        unsafe { &mut *self.data.get() }
    }
}

impl<T> core::ops::Drop for InterruptGuard<T> {
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
