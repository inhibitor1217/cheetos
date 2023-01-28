#![no_std]
#![warn(clippy::all)]

pub mod console;
pub mod devices;
pub mod init;
pub mod threads;
pub mod utils;

pub use init::init;
