pub mod addr;
pub mod interrupt;
mod palloc;
mod scheduler;
mod sync;
mod thread;

pub use self::scheduler::SCHEDULER;

pub use self::interrupt::init as interrupt_init;
pub use self::palloc::init as palloc_init;
pub use self::thread::init as thread_init;

pub use self::sync::lock::Mutex;
