# Runtime UI object store memory

Scope: freemvvm Core Runtime UI-thread-confined `UiObjectStore` and `ObjectIdentity`.

- The store is accessed only from its captured `UiThreadContext`; check thread affinity before reading or mutating slots.
- `ObjectIdentity` combines slot and nonzero generation. Destroy makes the identity unresolved; reuse increments generation before publishing the new value.
- Retire a slot at `u64::MAX` generation rather than wrapping. Keep free slots in an intrusive list so destroy does not need a new allocation.
- Convert slot indices with checked integer conversions, use `Vec::try_reserve` before adding a slot, and reject invalid/stale identities without mutation.
- This arena is infrastructure only. Public handles, Runtime registration, Property behavior, and shutdown cleanup integration remain separate units.
