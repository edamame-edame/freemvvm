# Core Runtime UI object store clear memory

Scope: `crates/core-runtime/src/object_store.rs`, `UNIT-CORE-035`.

- `UiObjectStore::clear` must check the captured `UiThreadContext` before touching slot state; this keeps wrong-thread failure non-mutating.
- Clearing takes and drops each live slot value on the caller thread, which is the owning UI thread after the check. It clears the intrusive free links and head without allocating, leaving old identities unresolved.
- Repeating clear is safe and does not drop values twice. Tests use a drop counter and verify off-thread clear preserves both the value and identity.
- The method is a primitive only. `RuntimeCore` cleanup-queue registration and shutdown wiring remain a separate implementation unit.
