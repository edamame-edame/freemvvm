# Match rust-analyzer to the pinned Rust toolchain

- Status: complete; commit and push recorded in Git history.
- Scope: include rust-analyzer as an optional component of the repository-pinned Rust 1.90.0 toolchain, update local/CI installation commands, and install it for the current Windows toolchain. No Rust language version or product source changes.
- Evidence: VS Code warned that its bundled rust-analyzer was newer than the repository-pinned toolchain and recommended adding `rust-analyzer` to `[toolchain].components`. The component is available for the installed 1.90.0 Windows toolchain.
- References: [DEVELOPMENT.md](../../DEVELOPMENT.md), [rust-toolchain.toml](../../rust-toolchain.toml), [bootstrap workflow](../../.github/workflows/bootstrap.yml).

## Plan

1. Add `rust-analyzer` to the pinned toolchain components and all exact 1.90.0 installation commands.
2. Install the component for the current 1.90.0 toolchain without changing the default toolchain or PATH.
3. Verify the toolchain reports the component installed and run repository checks.
4. Record the setup detail in `.docs/01_memory/`, then commit and push.

## Acceptance

- The pinned stable Rust toolchain includes its matching rust-analyzer component.
- Clean Windows/Linux setup and CI install commands agree with the pin.
- Rust compiler, formatting, lint, and build behavior remain unchanged.

## Result

- Added the matching `rust-analyzer` component to the Rust 1.90.0 pin and its Windows/Linux setup commands and bootstrap CI installation step.
- Installed the component locally; `rustup run 1.90.0 rust-analyzer --version` reports rust-analyzer 1.90.0 and the toolchain component list includes it.
- `rustup run 1.90.0 cargo check --workspace --all-targets --locked` and `tools/dev.py check` passed. The latter includes Rust format/Clippy/build checks, C/C++ probes, C# build, Python wheel build/install/import, and document links.
- Rust compiler version, default toolchain, and system PATH were not changed.
