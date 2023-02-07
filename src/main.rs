fn main() {
    let bios_path = core::env!("BIOS_PATH");
    let args: Vec<String> = std::env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("g", "gdb", "run with gdb");
    opts.optflag("h", "help", "print this help menu");

    let program = args[0].clone();
    let run_options = opts.parse(&args[1..]).unwrap();

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive");
    cmd.arg(format!("format=raw,file={bios_path}"));

    // Connect serial port to stdio of the host.
    cmd.arg("-serial");
    cmd.arg("stdio");

    // This enables `isa-debug-exit` device.
    cmd.arg("-device");
    cmd.arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    if run_options.opt_present("h") {
        let brief = format!("Usage: {program} [options]");
        print!("{}", opts.usage(&brief));
        return;
    }

    if run_options.opt_present("gdb") {
        /// Virtual address where kernel ELF is loaded by bootloader.
        const KERNEL_VADDR_OFFSET: u64 = 0x8000000000;

        cmd.arg("-s");
        cmd.arg("-S");

        println!("Run gdb in separate shell with following command:");
        println!(
            "gdb -ex \"target remote :1234\" -ex \"exec-file {kernel}\" -ex \"add-symbol-file {kernel} -o {offset:#0x}\"",
            kernel = env!("KERNEL_PATH"),
            offset = KERNEL_VADDR_OFFSET
        );
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
