#![no_std]
#![warn(clippy::all)]

pub mod threads;

/// Prints message prefixed with the name of the test.
#[macro_export]
macro_rules! msg {
    ($($arg:tt)*) => {
        kernel::print!("({}) ", crate::TEST_NAME);
        kernel::println!($($arg)*);
    }
}

/// Prints a message indicating the current test passed.
#[macro_export]
macro_rules! pass {
    () => {
        kernel::println!("({}) PASS", crate::TEST_NAME);
    };
}

/// Prints failure message, then panics the kernel.
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {
        kernel::print!("({}) FAIL: ", crate::TEST_NAME);
        kernel::println!($($arg)*);
        panic!("test failed");
    };
}
