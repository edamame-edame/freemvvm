# Runtime Binding delivery order

Scope: OneWay propagation before Runtime Property subscriber delivery (`UNIT-CORE-060`).

- Prepare subscriber snapshots for the current dirty generation first, then propagate while holding UI object store, Binding graph, and pending-change queue locks in that order. No subscriber callback starts until all reachable OneWay targets have committed.
- Pass the current-generation identity list into propagation. A target already represented by a current-generation item gets its final propagated value before that existing callback and is not queued again; other changed targets are appended once to the next generation.
- Before propagation commit, a candidate or notification reservation error leaves logical queue state unchanged, so the taken current generation can be restored. Once commit starts, targets remain stable under the object-store lock and prepared fixed-kind copies make the writes infallible.
- Focused delivery test (1), generation snapshot tests (2), workspace tests (139), and `.venv\Scripts\python.exe -X utf8 tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-binding-delivery-order.md) and `UT-CORE-RUNTIME-BINDING-DELIVERY-001`.
