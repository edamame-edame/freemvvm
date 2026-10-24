# Scope: C# BOOL/I64 Property wrapper

- `UiProperty<T>` currently supports `bool` and `long` through `TryCreate`, `TryGet`, `TrySet`, and explicit close/Dispose. Other managed types must be rejected before any native call until a matching typed mapping is implemented.
- C ABI `ui_bool` is `byte` and valid only as 0/1; `int64_t` maps directly to C# `long`. Never route either value through object-sized pointers or narrower integers.
- Every Property create/get/set call borrows the Runtime or Property token through `TokenOwner.AcquireLease`; explicit release failure leaves the owner retryable, and Dispose uses the cleanup ticket after failure.
- The real host C ABI without a UI loop bridge still supports creating and reading/writing properties on the Runtime's creating thread; only shutdown admission is unsupported in that host configuration.
- Extend the no-third-party C# console test harness with the Property fake dispatch, and keep the separate consumer process testing the actual built C ABI. `tools/dev.py check` runs Runtime and Property focused tests and consumers.
- `UNIT-WRAP-CS-PROP-BOOL-I64-001` passed local focused/consumer tests, Windows `tools/dev.py check`, Rust minimum, and GitHub Actions run 40049830264 on both Windows and Linux for implementation commit `0330ed2`.
