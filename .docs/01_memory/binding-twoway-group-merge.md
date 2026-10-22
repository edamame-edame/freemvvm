# Core Binding TwoWay group merge

Scope: Core Binding graph synchronization-group merge (`UNIT-CORE-053`).

- A successful TwoWay merge aliases one group root to another. OneWay edges are normalized through group roots when validating later connections, so prior edges follow merged membership without rewriting them.
- Before accepting a merge, normalize all OneWay edges under the proposed union and reject self-edges, duplicate edges, targets with multiple incoming groups, or cycles. Duplicate normalized edge is checked before multiple incoming source, matching approved ABI error precedence.
- Prepared OneWay and TwoWay changes carry the graph generation. Any intervening commit makes the proposal stale; stale changes cannot mutate accepted topology.
- Focused graph tests (5), workspace tests (124), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/binding-twoway-group-merge.md) and `UT-CORE-BINDING-GRAPH-002`.
