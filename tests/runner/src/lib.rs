use std::{io::Write, path::PathBuf};

// QEMU binary name.
const QEMU_BIN: &str = "qemu-system-x86_64";

// QEMU supports `isa-debug-exit` device, which can be used to exit QEMU.
// `isa-debug-exit` device uses port-mapped I/O to communicate with the
// host system. By sending a specific value to the device, QEMU can be
// instructed to exit with a specific exit code.
//
// On success, the kernel writes `0x31` to the I/O port, then QEMU will
// exit with exit code `(0x31 << 1) | 1`, hence `0x63`.
const QEMU_EXIT_CODE_SUCCESS: i32 = 0x63;

const QEMU_ARGS: &[&str] = &[
    // This enables `isa-debug-exit` device.
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    // Exit instead of rebooting.
    "--no-reboot",
];

/// Runs a kernel on QEMU with test setup.
pub fn run_test_kernel(kernel_binary_path: &str) {
    let kernel_binary = PathBuf::from(kernel_binary_path);
    let kernel_bios = kernel_binary.with_extension("mbr");
    bootloader::BiosBoot::new(&kernel_binary)
        .create_disk_image(&kernel_bios)
        .unwrap();

    let mut cmd = std::process::Command::new(QEMU_BIN);
    cmd.arg("-drive");
    cmd.arg(format!("format=raw,file={}", kernel_bios.display()));
    cmd.args(QEMU_ARGS);

    let child_output = cmd.output().unwrap();
    std::io::stdout().write_all(&child_output.stdout).unwrap();
    std::io::stderr().write_all(&child_output.stderr).unwrap();

    match child_output.status.code() {
        Some(QEMU_EXIT_CODE_SUCCESS) => (),
        Some(code) => panic!("Test failed with exit code {}", code),
        None => panic!("QEMU was killed by a signal"),
    }
}
