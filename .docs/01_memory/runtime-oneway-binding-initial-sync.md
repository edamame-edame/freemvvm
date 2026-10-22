# Implementation: Runtime OneWay Binding initial synchronization

Scope: internal Runtime operation that initially synchronizes and connects two Property states (`UNIT-CORE-056`).

- `RuntimeCore::create_one_way_binding_initial` validates both Property tokens against the same Runtime and UI thread, snapshots source with fallible owned copy, then prepares topology and target candidate.
- The operation holds the UI object-store and Binding graph locks across prepare/commit. For changed values it reserves notification queue capacity before using `BindingGraph::commit_one_way_with` to commit target value, notification identity, and graph edge. Equivalent values commit only the edge and create no notification.
- Kind/topology/runtime/thread rejection leaves target value, published graph, and notification queue unchanged. The graph proposal may reserve internal vector capacity without publishing topology.
- This is not full Binding support: application validators, Binding owner/handle/cancel, ongoing propagation, TwoWay/OneTime, and public C ABI remain future units.
- Focused tests (3), workspace tests (134), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-oneway-binding-initial-sync.md) and `UT-CORE-RUNTIME-BINDING-INIT-001`.
