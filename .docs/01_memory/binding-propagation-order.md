# Core Binding propagation ordering

Scope: topology snapshot for reachable OneWay propagation edges (`UNIT-CORE-057`).

- Normalize stored OneWay edges through TwoWay group roots before reachability and ordering.
- Start from the deduplicated dirty group identities, walk outgoing edges, and snapshot only reachable edges. Order selected edges with a stable topological pass so edge insertion order cannot place a downstream edge before its upstream edge.
- The operation is read-only with respect to accepted graph topology and Property values. Fallible snapshot allocation reports `BindingGraphError::OutOfMemory`; this slice does not commit propagated values or integrate Runtime delivery.
- Focused tests (2), workspace tests (136), and `.venv\Scripts\python.exe -X utf8 tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-oneway-propagation-order.md) and `UT-CORE-BINDING-PROPAGATION-ORDER-001`.
