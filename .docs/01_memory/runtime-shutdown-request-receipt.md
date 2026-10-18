# Rust Core Runtime: owned shutdown request receipt

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-019`.

- `HostRegistration::request_shutdown_receipt` returns an owned `ShutdownRequest` only after admission succeeds. Its `query` and `wait` delegate to the thread-independent completion record; dropping a receipt releases only that observer.
- HostRegistration's strong completion list owns accepted work independently of both the caller's `ShutdownCompletion` and any receipt. A shutdown boundary drains cleanup, takes that list, and marks every record Completed, so boundary commit does not need a caller-held fallback observer.
- UT-RUNTIME-005 verifies cross-thread Pending query, release of both caller handles before cleanup, one cleanup pass, and release of the record after commit. Worker Sync mode uses the receipt's wait path.
- The receipt remains an internal Rust runtime type. Public ABI token creation and Runtime Closed-shell ownership remain separate implementation units.
