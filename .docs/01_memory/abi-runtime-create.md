# Rust C ABI: Runtime creation

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-004`.

- `runtime_create` constructs `RuntimeCore`, stores an owning `Arc` in the process Runtime handle registry, and returns its nonzero monotonic opaque token. Handle release remains a later unit because dropping the last Runtime owner must be coordinated with shutdown.
- The C ABI boundary initializes optional `out_error` and required `out_runtime` before validating the required pointer. Null required output maps to `UI_INVALID_ARGUMENT`; registry reservation failure maps to `UI_OUT_OF_MEMORY`; token, identity, synchronization, and caught panic failures map to `UI_INTERNAL_ERROR`.
- The Core crate exposes only a hidden, narrow `abi_support::runtime_create_handle` bridge to the C ABI crate; wrapper crates must continue to use the versioned C ABI.
- `UT-ABI-004` covers distinct nonzero handles, optional error initialization, null required output, and panic-to-status conversion. Runtime handle table ownership/resolution and poisoned-lock behavior are covered by `UNIT-CORE-026`.
