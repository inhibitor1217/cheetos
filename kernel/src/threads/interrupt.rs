/// Returns `true` if interrupts are enabled, and `false` otherwise.
pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

/// Returns `true` if interrupts are disabled, and `false` otherwise.
pub fn are_disabled() -> bool {
    !are_enabled()
}

/// Returns `true` during processing of an external interrupt, and `false` at
/// all other times.
///
/// TODO: implement this properly after we implement interrupts.
pub fn is_external_handler_context() -> bool {
    false
}
