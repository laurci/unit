# unit

Learn more about this repository [here](https://laurci.notion.site/XDay-Hackathon-Log-087f6778a60245f58a4e0d9e2bea3b95?pvs=4).

## Getting started

Prerequisites:

- Rust
- `cargo-wasix`
- `protoc` protobuf compiler

Setup:

- `cargo install --git https://github.com/laurci/unit.git unit-cli`
- `unit-cli init <name of your app>`
- `cd <name of your app>`
- Edit the values in `.env` to point to your `unit` API

Deployment:

- Build the app: `cargo wasix build --release`
- Deploy to your configured unit instance: `unit-cli deploy ./target/wasm32-wasmer-wasi/release/<name of your app>.wasm`

## Building from source

After building the CLI, it's convenient to symlink it to `/usr/local/bin/unit` to make it accessible to the system: `sudo ln -s $PWD/target/release/unit-cli /usr/local/bin/unit`.
