# cheetos

Welcome to `cheetos` project. This a a simple operating system framework for the `x86_64` architecture.

`cheetos` uses [`bootloader`](https://crates.io/crates/bootloader) crate for most of the heavy lifting.

> TODO: Add more descriptions

## Getting started

### Requirements

- [Install Rust.](https://www.rust-lang.org/tools/install)
- Enable the nightly toolchain is this project.

```bash
rustup override set nightly
```

- Install the `llvm-tools-preview` component.

```bash
rustup component add llvm-tools-preview
```

- By default, Rust tries to build an executable for the host system. Since Cheetos runs on x86_64 architecture. we need to enable the `x86_64-unknown-none` target.

```bash
rustup target add x86_64-unknown-none
```

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

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
