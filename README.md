# cheetos

Welcome to `cheetos` project. This a a simple operating system framework for the `x86_64` architecture.

`cheetos` relies on [`bootloader`](https://crates.io/crates/bootloader) crate for most of the heavy lifting. `bootloader` creates a bootable disk image, create page tables, does the memory mapping, etc, so that we can focus on developing the kernel itself.

The structure of the project is highly inspired by the [pintos](https://www.scs.stanford.edu/22wi-cs212/pintos/pintos.html) project. However, since the original project was written in C, some parts of the code are not idiomatic in Rust. `cheetos` tries to address this issue by rewriting some of the code to match the interface from the standard library of Rust. (e.g. [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) for synchonization, [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) for memory allocation, etc.)

> `cheetos` is still under development!

> TODO: Add more description

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

### Debugging

You can use `gdb` to debug `pintos`. The following command will start `cheetos` in a qemu virtual machine, and wait for `gdb` to connect to it.

```bash
cargo run -- -g
```

### Testing

`pintos` contains a rich test suite (it is an educational operating system project, after all). `cheetos` ported the test suite to Rust, and you can run it with the following command.

```bash
cargo test
```

# Acknowledgements

This project is highly inspired by the following projects:

- [pintos](https://www.scs.stanford.edu/22wi-cs212/pintos/pintos.html)
- [Phil Opp's Blog](https://os.phil-opp.com/)
- [`rust-osdev`](https://github.com/rust-osdev)

Simply said, `cheetos` is an attempt to port Pintos into Rust.

Also, the following crates were referenced in this project.
`cheetos` do not use these crates as direct dependency, but rather uses sources derived from them.

- [conquer-once](https://crates.io/crates/conquer-once)
- [pic8259](https://crates.io/crates/pic8259)
- [uart_16550](https://crates.io/crates/uart_16550)

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
