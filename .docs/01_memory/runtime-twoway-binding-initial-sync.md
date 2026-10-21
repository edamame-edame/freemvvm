# Runtime TwoWay binding initial sync

Scope: Runtime TwoWay initial value + synchronization-group commit (`UNIT-CORE-061`).

- Preserve the Runtime lock order: object store, Binding graph, pending Property notifications. Prepare the owned source copy, target typed candidate, and graph merge before mutating state.
- A changed initial target requires a reserved notification slot before the graph merge closure commits the target and queues its notification. An equivalent initial value commits only the graph merge.
- `BindingGraph::prepare_two_way_merge` normalizes OneWay endpoints after a proposed group union, so merging the endpoints of an existing OneWay edge is classified as `SelfConnection`.
- Focused tests (2), workspace tests (141), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-twoway-binding-initial-sync.md) and `UT-CORE-RUNTIME-TWOWAY-INIT-001`.
