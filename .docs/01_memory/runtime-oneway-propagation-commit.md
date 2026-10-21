# Runtime OneWay propagation commit

Scope: commit of prepared OneWay target values and next-generation notifications (`UNIT-CORE-059`).

- Use the established lock order: UI object store, Binding graph, Property change queue. Keep these locks through candidate preparation, reservation of every changed target identity, then value and notification commit.
- Preparation and notification reservation may fail before logical mutation. Once reserved, target object lookup/type are stable under the object-store lock and candidates are already fixed-kind copies; commit each value and append its target identity without allocation. `PropertyChangeQueue` preserves prior dirty order and coalesces target identities already queued.
- Binding-created target notifications remain in the pending queue for the next generation. This unit does not deliver subscribers or apply application validators; it is not full public Binding propagation.
- Focused test (1), workspace tests (138), and `.venv\Scripts\python.exe -X utf8 tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-oneway-propagation-commit.md) and `UT-CORE-RUNTIME-BINDING-COMMIT-001`.
