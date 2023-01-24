#![no_std]
#![warn(clippy::all, clippy::pedantic)]

pub mod devices;
pub mod init;

pub use init::init;
