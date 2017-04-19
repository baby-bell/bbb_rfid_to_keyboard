# Compiling the Rust library
You will need:
- the Rust compiler
- the Rust standard library (cross compiled for armv7)

Follow the instructions [here](https://github.com/japaric/rust-cross) to get the compiler working.
Compile with `cargo build --target=armv7-unknown-linux-gnueabihf --release`.
