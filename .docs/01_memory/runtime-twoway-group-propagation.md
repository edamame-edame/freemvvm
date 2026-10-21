# Runtime TwoWay group propagation

Scope: `UNIT-CORE-062` in `crates/core-runtime`.

- Keep direct-setter recency separate from notification queue order: notification order is first-change order, while a group's winner is the last successful direct change in the current generation.
- Record recency only after a successful value change. Failed setters and equivalent-value writes do not move the winner position.
- Build propagation candidates from the Binding graph's normalized edge order. A TwoWay group is treated as one source/target unit for OneWay propagation; an incoming OneWay source takes precedence over direct changes to its target group in the same generation.
- Prepare candidates and reserve notifications before committing values. If generation preparation fails, restore both the pending generation and direct-change recency.
- Commit group synchronization before subscriber callbacks. Members already in the current generation keep their notification slot; other changed members are queued once for the next generation, preventing recursive delivery.
- Keep group member ordering deterministic by using the representative followed by identities ordered by slot and generation.
- Focused propagation tests, workspace tests (144), `tools/dev.py check`, format, and documentation-link checks passed on 2026-10-21. See [plan](../20_todo/runtime-twoway-group-propagation.md) and [test oracle](../32_unit_test/core-runtime.md#ut-core-runtime-twoway-propagation-001).
