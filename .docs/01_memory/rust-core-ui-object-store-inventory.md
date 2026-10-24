<!-- scope: Rust Core UI object-store implementation and unit-inventory reconciliation -->
# Rust Core UI object-store inventory memory

- Before selecting a `ready` unit for new work, compare its implementation sources, focused test, unit-test design, and matching `.docs/20_todo` status. The Core UI object-store units 034-036 were already implemented and their TODO records said `complete`, while `tools/units.yaml` still said `ready`.
- `cargo test -p freemvvm-core` verifies the current Core test baseline; on 2026-10-24 it passed all 128 tests, including the seven `object_store::tests` and Runtime shutdown cleanup tests.
- Reconcile inventory status only after confirming all source/test evidence exists and rerunning the mapped command. Keep the existing requirements and design references unchanged when the mismatch is only bookkeeping.
