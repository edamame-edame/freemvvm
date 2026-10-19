# Rust C ABI: Property create and kind query

Scope: `UNIT-ABI-009`, versioned C ABI and Runtime-owned Property handles.

- Keep public Property tokens in a dedicated `HandleRegistry<PropertyHandleKind, RuntimeIdentity>`; maintain a token-to-`Arc<RuntimeCore>` owner anchor because `property_kind` has no Runtime argument.
- Reserve owner-map capacity before registering the Property, then publish the anchor before returning the token. Generic release consumes the registry token first and removes the matching Runtime anchor after success.
- Query clones the Runtime anchor before validating/pinning the token. This lets concurrent release invalidate the query safely while keeping the Runtime alive for the in-flight call.
- Map Core `ValueKind` to stable ABI numbers in the adapter: BOOL=1, I64=2, UTF8=3, F64=5, DECIMAL=6. OBJECT=4 stays reserved.
- In C ABI tests, run `cargo test -p freemvvm-c-abi property_` to include all property creation/query/release oracles in this unit.
