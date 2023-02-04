mod control;
mod mutex;
mod pic;

pub use self::control::are_disabled;
pub use self::control::are_enabled;
pub use self::control::disable;
pub use self::control::enable;
pub use self::control::is_external_handler_context;

pub use self::mutex::Mutex;
pub use self::mutex::MutexGuard;

const EXTERNAL_INTERRUPT_OFFSET: u8 = 0x20;

static PICS: Mutex<pic::Pics> = Mutex::new(pic::Pics::new(EXTERNAL_INTERRUPT_OFFSET));

/// Initializes the interrupt system.
pub fn init() {
    unsafe {
        PICS.lock().init();
    }
}
