# Rust Core Runtime: release of a closed shell from a worker

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-020`.

- Runtime shutdown drains tracked UI state before publishing Closed. A remaining public handle then refers only to the thread-safe control shell.
- Releasing the final public token after Closed does not reserve or enqueue UI cleanup; it only removes the registry entry and releases the shell. This operation is safe from a worker thread.
- UT-HANDLE-SHUTDOWN-001's worker-path check confirms the control block is dropped, the queue remains empty, and duplicate release fails as InvalidToken.
- Runtime UI-operation rejection and immutable Error reads after shutdown remain unimplemented and are not covered by this slice.
