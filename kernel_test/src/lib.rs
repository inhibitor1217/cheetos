#![no_std]
#![warn(clippy::all)]

/// Prints failure message, then panics the kernel.
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {
        kernel::print!("({}) FAIL: ", crate::TEST_NAME);
        kernel::println!($($arg)*);
        panic!("test failed");
    };
}
