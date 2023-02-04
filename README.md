# cheetos

Welcome to `cheetos` project. This a a simple operating system framework for the `x86_64` architecture.

`cheetos` uses [`bootloader`](https://crates.io/crates/bootloader) crate for most of the heavy lifting.

> TODO: Add more descriptions

## Getting started

### Requirements

- [Install Rust.](https://www.rust-lang.org/tools/install)
- Install [`qemu`](https://www.qemu.org/index.html) for running the operating system in a virtual machine.

### Run `cheetos`

Build the project and run it in a qemu virtual machine.

```bash
cargo run
```

Under the hood, it works like this:

- `cargo run` will first attempt to build the project, if not already built.
- Since `Cargo.toml` marks `kernel` as the [`build-dependencies`](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#build-dependencies), it will build the `kernel` crate, which is our operating system.
- In `build.rs`, we depend on the `bootloader` crate, which will:
  - Build the kernel into an ELF file.
  - Compile the bootloader as a standalone executable.
  - Links the bytes of kernel ELF file into the bootloader executable.
- The `bootloader` crate will then generate a `bootimage` file, which is a bootable disk image.
- In `src/main.rs`, we use the built disk image to run the operating system in a qemu virtual machine.
- Only if the sources in `kernel` crate is changed, `cargo` will rebuild the project.

# Acknowledgements

This project is highly inspired by the following projects:

- [pintos](https://www.scs.stanford.edu/22wi-cs212/pintos/pintos.html)
- [Phil Opp's Blog](https://os.phil-opp.com/)
- [`rust-osdev` organization](https://github.com/rust-osdev)

Simply said, `cheetos` is an attempt to port Pintos into Rust.

Also, the following crates were referenced in this project.
`cheetos` do not use these crates as direct dependency, but rather uses sources derived from them.

- [conquer-once](https://crates.io/crates/conquer-once)
- [pic8259](https://crates.io/crates/pic8259)
- [uart_16550](https://crates.io/crates/uart_16550)

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
