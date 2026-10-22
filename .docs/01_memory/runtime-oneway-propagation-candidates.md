# Runtime OneWay propagation candidate preparation

Scope: Runtime preparation of owned OneWay target values (`UNIT-CORE-058`).

- Lock in the established object-store then Binding-graph order. Resolve an ordered graph snapshot, read a source from the most recent prepared upstream target candidate when present, otherwise from the current Property, and prepare each target through its fixed-kind `ValueStorage`.
- Keep prepared updates local until all edges succeed. Candidate preparation does not mutate Property values, graph topology, or the dirty notification generation; a later chain edge consumes its earlier target candidate.
- This Runtime foundation currently connects OneWay edges only. Runtime TwoWay value winner selection, application validators, value/notification commit, Binding errors, and subscriber delivery remain separate units.
- Focused test (1), workspace tests (137), and `.venv\Scripts\python.exe -X utf8 tools/dev.py check` passed on 2026-10-21. See [implementation plan](../20_todo/runtime-oneway-propagation-candidates.md) and `UT-CORE-RUNTIME-BINDING-PROP-001`.
