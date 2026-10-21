# Runtime Binding active ownership and cancellation

Scope: Core Runtime Binding groundwork (`UNIT-CORE-063`).

- Give each committed OneWay/TwoWay graph connection a stable identity that survives later topology changes. OneWay uses its original endpoint pair; TwoWay cancellation must retain the original endpoint pair, not only the merged representatives.
- Removing a TwoWay connection requires rebuilding synchronization groups from the remaining TwoWay edges; a union-only group map cannot support cancellation correctly.
- Store original OneWay endpoints in graph edges. If a OneWay edge was created between aliases of a TwoWay group, canceling the TwoWay link must restore the original OneWay source/target identities.
- Active Runtime ownership holds validation pins for both endpoints. Releasing all external Property tokens can reserve final cleanup, but the Runtime pin delays it until cancel or shutdown drops the active owner.
- Keep pending Property notifications after cancel, but remove the connection from the graph before the next notification generation is prepared. This lets independent Property notification proceed while suppressing only the canceled edge.
- On shutdown, drop active Binding owners and clear the graph before clearing UI object state so endpoint pins release while the Runtime cleanup path is still available.
- The Core unit does not expose a public Binding token or C ABI. Those entrypoints must route explicit cancel to the internal connection identity and preserve the same release-versus-cancel distinction.
- Focused graph/runtime lifecycle tests (4), workspace tests (148), `tools/dev.py check`, format, and documentation-link checks passed on 2026-10-21. See the [plan](../20_todo/runtime-binding-lifecycle.md) and [test design](../32_unit_test/core-runtime.md#ut-core-runtime-binding-lifecycle-001).
