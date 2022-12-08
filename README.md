# OMO

Bytecode emulator with per-step state proof.
It can be used to generate challenge proof of optimistic rollup,
and other scenarios in blockchain which need state proof.

See more introductions here: [en](docs/overview.md) / [zh](docs/ch/overview.md).

## Platforms

Could emulate program built with:

- **Arch**: MIPS32
- **OS**: Linux
- **Executable File Format**: ELF 32-bit MSB

May support more in the future.

## Getting Started

The project contains two Rust crates:

- `./omo` : main entrypoint of the OMO emulator.
- `./rust-mips-example`: example crate. It is configured to build into a linux mips binary, which can be run by `omo`.

### Prerequisites

- [rust](https://rustup.rs/)

#### Using Rust Cross

- [cross](https://github.com/cross-rs/cross)
- Docker: cross needs it.
- [cmake](https://cmake.org/download/) >= 3.12

#### Using MUSL tools

- Add mips-unknown-linux-musl supports:
```shell
rustup target add mips-unknown-linux-musl
```
- Download musl toolchain from [musl.cc](https://musl.cc): mips-linux-musl-cross

### Run Example

**Compile `rust-mips-example`:**

```shell
cd ./rust-mips-example
cross build --target mips-unknown-linux-musl --release -v
# the compiled mips binary will be ./target/mips-unknown-linux-musl/release/rust-mips-example
file target/mips-unknown-linux-musl/release/rust-mips-example
```

**If using MUSL tools:**

- Set `linker = "mips-linux-musl-gcc"` in [cargo config](rust-mips-example/.cargo/config.toml)
```shell
cargo build --target mips-unknown-linux-musl --release --no-default-features
```

**Compile `OMO`:**
```shell
cargo build --release
```

**Run:**
```shell
cd ./omo
cargo run -- --config config.toml.example --env E1=a --env E2=b ../rust-mips-example/target/mips-unknown-linux-musl/release/rust-mips-example E1 E2
```

**Output:**
```
Run ../rust-mips-example/target/mips-unknown-linux-musl/release/rust-mips-example
E1=a
E2=b
```

## License

Distributed under the Apache License 2.0. See [LICENSE](LICENSE) for more information.

## Acknowledgments

- [Cannon](https://github.com/ethereum-optimism/cannon)
- [Qiling](https://github.com/qilingframework/qiling)
