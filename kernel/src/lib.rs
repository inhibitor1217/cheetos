#![no_std]
#![warn(clippy::all, clippy::pedantic)]

pub mod console;
pub mod devices;
pub mod init;
pub mod threads;

pub use init::init;
