# Implementation: Core Binding initial graph/value proposal preparation

Scope: combine OneWay graph validation and typed initial target-copy preparation (`UNIT-CORE-055`).

- `BindingGraph::prepare_one_way_with_initial_copy` validates topology first, then prepares an owned typed candidate through `ValueStorage`. It returns both as a proposal; it does not commit either.
- Topology and kind errors leave accepted edge count/generation and target value unchanged. Existing graph preparation may reserve vector capacity, but this does not publish an edge or alter logical topology.
- This slice does not run application validators or coordinate candidate commit with Runtime ownership, notification queue, or Binding handles. Those remain later integration work.
- Focused tests (3), workspace tests (131), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/binding-initial-graph-prepare.md) and `UT-CORE-BINDING-INIT-PREPARE-001`.
