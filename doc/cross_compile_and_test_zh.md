
# 使用 Cargo 进行交叉编译指南

本文档介绍了如何使用`cargo` 将`movefmt`交叉编译为适用于其他架构（如 ARM64 Linux）的可执行文件。

## 1. 环境准备

确保已安装以下工具：

- Rust（建议通过 [rustup](https://rustup.rs/) 安装）
- `cargo`
- `gcc` 或 `clang` 交叉编译工具链（例如 `aarch64-linux-gnu-gcc`或者`aarch64-unknown-linux-musl`）
- 目标平台支持（通过 `rustup` 安装）

## 1.1 步骤一：添加目标

```bash
rustup target add aarch64-unknown-linux-gnu     // 动态链接 
// or
rustup target add aarch64-unknown-linux-musl    // 静态链接
```

## 1.2 步骤二：安装交叉编译工具链

以 Ubuntu 为例：

```bash
sudo apt update
sudo apt install gcc-aarch64-linux-gnu

sudo wget https://musl.cc/aarch64-linux-musl-cross.tgz
sudo tar -xvzf aarch64-linux-musl-cross.tgz
some command for export `aarch64-linux-musl-cross/bin` to $PATH
```

安装后将获得类似 `aarch64-linux-gnu-gcc` 的编译器。

## 1.3 步骤三：创建 `.cargo/config.toml`

在项目根目录下新建或编辑 `.cargo/config.toml` 文件，添加以下内容：

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
ar = "aarch64-unknown-linux-gnu-gcc"

[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
```

该配置告诉 `cargo` 在交叉编译时使用该交叉链接器。

## 1.4 步骤四：编译项目

```bash
cargo build --target aarch64-unknown-linux-gnu --release
// or
cargo build --target aarch64-unknown-linux-musl --release
```

构建产物会出现在：

```
target/aarch64-unknown-linux-gnu/release/movefmt
target/aarch64-unknown-linux-musl/release/movefmt
```

## 2. 测试

### 2.1 安装qemu
从 https://download.qemu.org/ 下载qemu源码 或者 从 https://github.com/qemu/qemu 克隆

```bash
cd qemu
mkdir build
cd build
../configure
make
```

### 2.2 在本机使用QEMU环境测试（静态链接）

可以使用 `qemu-aarch64` 在 x86 主机上运行 AArch64 编译结果：

```bash
qemu-aarch64 target/aarch64-unknown-linux-musl/release/movefmt your_move_file.move 
```

### 2.3 在aarch64环境中测试

安装aarch64虚拟机
```bash
qemu-system-aarch64 -m 2048 -cpu cortex-a57 -smp 2 -M virt -bios QEMU_EFI.fd -nographic -drive if=none,file=ubuntu-22.04.05-server-arm64.iso,id=cdrom,media=cdrom -device virtio-scsi-device -device scsi-cd,drive=cdrom -drive if=none,file=ubuntu16.04-arm64.img,id=hd0 -device virtio-blk-device,drive=hd0
```

运行单元测试
```bash
cargo test
```

## 参考资料

- [Rust 官方交叉编译文档](https://doc.rust-lang.org/stable/rustc/targets/index.html)
- [cross 项目 - 更简洁的交叉编译工具](https://github.com/cross-rs/cross)
