# Core Binding OneWay graph

Scope: Core OneWay graph topology model (`UNIT-CORE-052`).

- Each Property identity is a singleton synchronization-group vertex in the first graph slice; TwoWay group merging is a later responsibility.
- `prepare_one_way` checks topology and reserves edge storage without changing accepted edges. `commit_one_way` accepts only a proposal for the current graph generation, so a stale proposal cannot publish topology validated against older state.
- Rejection order distinguishes self-connection, exact duplicate edge, second input to an existing target, and cycle. Duplicate edge takes precedence over the general second-input condition, matching the approved ABI error mapping.
- Focused graph tests (2), workspace tests (121), and `tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/binding-oneway-graph-invariants.md) and `UT-CORE-BINDING-GRAPH-001`.
