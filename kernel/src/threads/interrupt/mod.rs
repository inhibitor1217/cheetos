mod constants;
mod control;
mod handler;
mod mutex;
mod pic;

pub use self::control::are_disabled;
pub use self::control::are_enabled;
pub use self::control::disable;
pub use self::control::enable;

pub use self::handler::is_external_handler_context;
pub use self::handler::REGISTRY;

pub use self::mutex::Mutex;
pub use self::mutex::MutexGuard;

static PICS: Mutex<pic::Pics> =
    Mutex::new(pic::Pics::new(self::constants::EXTERNAL_INTERRUPT_OFFSET));

/// Initializes the interrupt system.
pub fn init() {
    // Initialize interrupt controller.
    unsafe {
        PICS.lock().init();
    }

    // Initialize the interrupt handler registry.
    REGISTRY.lock().init();
    REGISTRY.peek().load();
}
