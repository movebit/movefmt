# Cross-Compilation Guide Using Cargo

This document explains how to use `cargo` to cross-compile `movefmt` into executables for other architectures, such as ARM64 Linux.

## 1. Environment Setup

Make sure the following tools are installed:

- Rust (recommended to install via [rustup](https://rustup.rs/))
- `cargo`
- Cross-compilation toolchain such as `gcc` or `clang` (e.g. `aarch64-linux-gnu-gcc` or `aarch64-unknown-linux-musl`)
- Target platform support (install via `rustup`)

## 1.1 Step 1: Add Target

```bash
rustup target add aarch64-unknown-linux-gnu     # Dynamic linking
# or
rustup target add aarch64-unknown-linux-musl    # Static linking
```

## 1.2 Step 2: Install Cross Compilation Toolchain

On Ubuntu:

```bash
sudo apt update
sudo apt install gcc-aarch64-linux-gnu

sudo wget https://musl.cc/aarch64-linux-musl-cross.tgz
sudo tar -xvzf aarch64-linux-musl-cross.tgz
# some command to export `aarch64-linux-musl-cross/bin` to $PATH
```

After installation, you’ll have access to compilers like `aarch64-linux-gnu-gcc`.

On MacOS:
```bash
rustup target add aarch64-apple-darwin

cargo build --target aarch64-apple-darwin --release
# debug version
cargo build --target aarch64-apple-darwin
```

## 1.3 Step 3: Create `.cargo/config.toml`

Create or edit the `.cargo/config.toml` file in the root of your project and add the following:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
ar = "aarch64-unknown-linux-gnu-gcc"

[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
```

This configuration tells `cargo` which linker to use when cross-compiling.

## 1.4 Step 4: Build the Project

```bash
cargo build --target aarch64-unknown-linux-gnu --release
# or
cargo build --target aarch64-unknown-linux-musl --release
```

The build output will appear at:

```
target/aarch64-unknown-linux-gnu/release/movefmt
target/aarch64-unknown-linux-musl/release/movefmt
```

## 2. Testing

### 2.1 Install QEMU
Download QEMU source code from https://download.qemu.org/ or clone from https://github.com/qemu/qemu:

```bash
cd qemu
mkdir build
cd build
../configure
make
```

### 2.2 Run in QEMU Environment on Host Machine (Static Linking)

You can use `qemu-aarch64` to run the AArch64 binary on an x86 host:

```bash
qemu-aarch64 target/aarch64-unknown-linux-musl/release/movefmt your_move_file.move 
```

### 2.3 Run in a Native AArch64 Environment

Set up an AArch64 virtual machine:

```bash
qemu-system-aarch64 -m 2048 -cpu cortex-a57 -smp 2 -M virt -bios QEMU_EFI.fd -nographic -drive if=none,file=ubuntu-22.04.05-server-arm64.iso,id=cdrom,media=cdrom -device virtio-scsi-device -device scsi-cd,drive=cdrom -drive if=none,file=ubuntu16.04-arm64.img,id=hd0 -device virtio-blk-device,drive=hd0
```

Run unit tests:

```bash
cargo test
```

## References

- [Rust official cross-compilation documentation](https://doc.rust-lang.org/stable/rustc/targets/index.html)
- [Cross project – a simpler tool for cross-compilation](https://github.com/cross-rs/cross)
