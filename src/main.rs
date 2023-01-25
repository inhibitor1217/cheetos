fn main() {
    let bios_path = core::env!("BIOS_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive");
    cmd.arg(format!("format=raw,file={bios_path}"));

    // Connect serial port to stdio of the host.
    cmd.arg("-serial");
    cmd.arg("stdio");

    // This enables `isa-debug-exit` device.
    cmd.arg("-device");
    cmd.arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
