#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' '--- Debian ---'
cat /etc/debian_version
printf '%s\n' '--- Rust ---'
rustc --version
cargo --version
cargo test -p freemvvm-core --locked
cargo test -p freemvvm-platform-linux --locked
cargo test -p freemvvm-platform-linux headless_gles_renders_core_scale_background_clip_and_fill_pixels --locked -- --ignored --nocapture
cargo check -p freemvvm-platform-linux --locked
cargo clippy -p freemvvm-platform-linux --all-targets --locked -- -D warnings
cargo +1.85.0 check -p freemvvm-platform-linux --all-targets --locked
