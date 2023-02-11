#![no_std]
#![warn(clippy::all)]

pub mod threads;

/// Prints message prefixed with the name of the test.
#[macro_export]
macro_rules! msg {
    ($test_name:expr, $($arg:tt)*) => {
        kernel::print!("({}) ", $test_name);
        kernel::println!($($arg)*);
    }
}

/// Prints a message indicating the current test passed.
#[macro_export]
macro_rules! pass {
    ($test_name:expr) => {
        kernel::println!("({}) PASS", $test_name);
    };
}

/// Prints failure message, then panics the kernel.
#[macro_export]
macro_rules! fail {
    ($test_name:expr, $($arg:tt)*) => {
        kernel::print!("({}) FAIL: ", $test_name);
        kernel::println!($($arg)*);
        panic!("test failed");
    };
}
