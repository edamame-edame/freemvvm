[Rust wrapper / Property]
# Rust safe Property wrapper implementation notes

- The `freemvvm-sys` crate declares versioned ABI functions; keep safe Property API code in `freemvvm/src/property.rs`.
- If `native_api.rs` is explicitly supplied as read-only, preserve it and avoid adding Property operations there. `UiContext` may expose crate-private Runtime identity, token, and shared API accessors for sibling modules.
- A typed owner should reuse `Owned<T>` so explicit close, Drop cleanup tickets, Runtime anchors, and thread affinity remain consistent.
- The approved Rust DD 1.1 specified `Property<T>::get/set` but did not name the public constructor. Put a proposed constructor and ownership mapping into a new DD revision draft and obtain whole-document review before committing the plan or implementation.
- For an initial BOOL/I64 slice, a real separate Rust consumer can validate symbol widths and value mapping through the built shared C ABI. Keep UTF-8/F64/DECIMAL, subscriptions, and Binding as separate units.
