use std::{io::Write, path::PathBuf};

// QEMU binary name.
const QEMU_BIN: &str = "qemu-system-x86_64";

// QEMU supports `isa-debug-exit` device, which can be used to exit QEMU.
// `isa-debug-exit` device uses port-mapped I/O to communicate with the
// host system. By sending a specific value to the device, QEMU can be
// instructed to exit with a specific exit code.
//
// On shutdown, the kernel writes `0x31` to the I/O port, then QEMU will
// exit with exit code `(0x31 << 1) | 1`, hence `0x63`.
const QEMU_EXIT_CODE_SUCCESS: i32 = 0x63;

// In constrast, the kernel writes `0x42` to the I/O port on failure.
const QEMU_EXIT_CODE_FAILURE: i32 = 0x85;

const QEMU_ARGS: &[&str] = &[
    // This enables `isa-debug-exit` device.
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    // Connect serial port to stdio of the host.
    "-serial",
    "stdio",
    // Disable GUI.
    "-display",
    "none",
    // Exit instead of rebooting.
    "--no-reboot",
];

pub struct TestOptions {
    pub gdb: bool,
}

impl TestOptions {
    pub fn default() -> TestOptions {
        TestOptions { gdb: false }
    }
}

/// Runs a kernel on QEMU with test setup.
pub fn run_test_kernel(kernel_binary_path: &str, options: TestOptions) {
    let kernel_binary = PathBuf::from(kernel_binary_path);
    let kernel_bios = kernel_binary.with_extension("mbr");
    bootloader::BiosBoot::new(&kernel_binary)
        .create_disk_image(&kernel_bios)
        .unwrap();

    let mut cmd = std::process::Command::new(QEMU_BIN);
    cmd.arg("-drive");
    cmd.arg(format!("format=raw,file={}", kernel_bios.display()));
    cmd.args(QEMU_ARGS);

    if options.gdb {
        /// Virtual address where kernel ELF is loaded by bootloader.
        const KERNEL_VADDR_OFFSET: u64 = 0x8000000000;

        cmd.arg("-s");
        cmd.arg("-S");

        println!("Run gdb in separate shell with following command:");
        println!(
            "gdb -ex \"target remote :1234\" -ex \"exec-file {kernel}\" -ex \"add-symbol-file {kernel} -o {offset:#0x}\"",
            kernel = kernel_binary_path,
            offset = KERNEL_VADDR_OFFSET
        );
    }

    let child_output = cmd.output().unwrap();
    std::io::stdout().write_all(&child_output.stdout).unwrap();
    std::io::stderr().write_all(&child_output.stderr).unwrap();

    match child_output.status.code() {
        Some(QEMU_EXIT_CODE_SUCCESS) => (),
        Some(QEMU_EXIT_CODE_FAILURE) => panic!("Test failed"),
        Some(code) => panic!("QEMU exited with code {}", code),
        None => panic!("QEMU was killed by a signal"),
    }
}
