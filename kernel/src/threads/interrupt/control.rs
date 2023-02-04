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

/// Returns `true` during processing of an external interrupt, and `false` at
/// all other times.
///
/// TODO: implement this properly after we implement interrupts.
pub fn is_external_handler_context() -> bool {
    false
}
