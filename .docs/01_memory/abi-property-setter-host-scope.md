# C ABI Property setter host scope

Scope: Rust Core `UNIT-ABI-012`, Core-side C ABI Property setters and notification safe points.

- `abi_support::apply_property_value` routes resolved handles through `RuntimeCore::apply_property_candidate_in_host_event`.
- The Runtime helper opens a host event scope, applies the typed candidate, and closes the scope whether the update succeeds or fails. If both update and close report errors, the update error remains primary.
- Nested setter scopes defer notifications to the enclosing outer boundary. At a standalone outermost setter, the existing `end_host_event` delivery slice runs before the helper returns.
- Existing wrong-kind setter errors remain `PropertyApiError::WrongType`; wrong thread, closed Runtime, and allocation failures from host begin retain their corresponding API mappings. Other boundary failures map to Internal.
- Focused oracle: `UT-ABI-012`; focused and workspace tests plus `tools/dev.py check` passed on 2026-10-20. This does not implement a public subscription ABI or Binding propagation.
