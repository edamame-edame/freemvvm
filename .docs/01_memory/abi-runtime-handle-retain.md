# Rust C ABI: Runtime handle retain

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-ABI-006`.

- Retain must validate and register the new Runtime token under one RuntimeHandleRegistry lock. The source token remains registered, and both tokens map to the same Runtime owner.
- Reserve table capacity before issuing a token. Map invalid source tokens to `UI_INVALID_HANDLE`, fallible table reservation to `UI_OUT_OF_MEMORY`, and token-issuer/synchronization/panic failures to `UI_INTERNAL_ERROR`; initialize `out_owned` to zero before validation.
- The process token issuer is shared with Runtime identities and other public handles, so retained tokens do not collide or reuse identifiers.
- Runtime `handle_release` remains separate: final owner destruction must be ordered after shutdown has removed UI-thread-affine state. Do not infer shutdown from releasing a token.
- `UNIT-ABI-006` pairs a registry test for same-owner independent tokens with C ABI tests for successful retain, source preservation, invalid input, output initialization, OOM mapping, and panic containment.
