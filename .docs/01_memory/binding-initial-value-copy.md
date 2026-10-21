# Implementation: Core Binding initial value copy preparation

Scope: typed candidate preparation for initial Binding synchronization (`UNIT-CORE-054`).

- `ValueStorage::prepare_binding_initial_copy` checks source kind and value equivalence before making an owned candidate; it does not mutate the target. Equivalent values return no candidate so retained representation and notification-free behavior can be preserved.
- Scalar and Decimal values copy by value. UTF-8 uses `OwnedUtf8::copy_from_bytes`, which owns an exact, fallible byte copy. Candidate preparation and target commit are distinct operations.
- This is a prerequisite primitive only: target validators and atomic commit with Binding graph, handle, and notification queue remain unimplemented integration work.
- Focused tests (4), workspace tests (128), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/binding-initial-value-copy.md) and `UT-CORE-BINDING-INIT-COPY-001`.
