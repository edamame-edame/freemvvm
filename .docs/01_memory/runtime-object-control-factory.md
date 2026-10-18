# Runtime-owned object control factory

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-025`.

- HandleRegistry is a process-token table and should not be embedded once per RuntimeCore. Runtime object entries carry their owning Runtime identity, while RuntimeCore owns the identity and cleanup queue.
- Construct Runtime-bound `CoreObjectControl` through RuntimeCore so identity and queue cannot be paired inconsistently by downstream constructors.
- A control's identity metadata is thread-safe; this factory does not expose or create UI object state. UI-state access still requires the separate Runtime thread/store path.
