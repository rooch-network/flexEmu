## OMO

### Run Example:

```
# build mips binary
cd ../rust-mips-example
cross build --target mips-unknown-linux-musl --release -v

cd ../omo
cargo run -- --config config.toml.example --env OMO=abc ../rust-mips-example/target/mips-unknown-linux-musl/release/rust-mips-example OMO
```

