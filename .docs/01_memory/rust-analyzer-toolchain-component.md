# Rust development environment / pinned rust-analyzer component

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, Rust toolchain and VS Code rust-analyzer setup.

- The repository pins Rust 1.90.0 in `rust-toolchain.toml`; VS Code may ship a newer rust-analyzer binary that warns against this compiler version.
- Pin `rust-analyzer` as a rustup component beside rustfmt and clippy, and include it in clean setup commands so rust-analyzer can use the version paired with the compiler.
- Adding a component does not change `rustup default`, system PATH, Rust compiler version, or product build semantics.
