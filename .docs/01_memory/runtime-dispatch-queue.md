# Rust Core Runtime: dispatcher queue implementation

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-006`.

- `runtime.md` v1.1 and `core-runtime.md` v0.4 approve FIFO posting, one callback per UI turn, non-reentrant callback execution, and wake coalescing/lost-wake tests. The external host bridge connection is left to ABI design.
- `dispatcher.rs` stores posted `FnOnce + Send` callbacks in a mutex-protected `VecDeque`; each accepted callback is inserted before asynchronous wake scheduling. A turn clears the pending wake before invoking one callback, then a guard rechecks remaining work after the callback, including on unwind.
- For deterministic drain-boundary coverage, the test-only turn hook synchronizes a worker post after callback return and before the final queue recheck. The test uses a barrier and channel acknowledgement rather than elapsed-time sleeps.
- `UNIT-CORE-006` covers UT-RUNTIME-001 and UT-RUNTIME-002 only. Shutdown priority, cancel/destroy ownership, host registration, and ABI connection require separate units.
