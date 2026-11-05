# Scope: Window background RGB Core and C ABI implementation

- Per-Window persistent background override belongs to `WindowLifecycle`; `None` resolves to opaque white `[1.0, 1.0, 1.0]`.
- Validate all RGB channels for finiteness and `[0, 1]` before mutation. Invalid set must preserve the prior override and frame state.
- Background changes are paint invalidation, not Layout invalidation. `force_full_damage` ensures the next submitted frame repaints; `begin_frame` applies the Window's effective RGB to `SceneSnapshot` before submission.
- C ABI get/set/reset exports are in `crates/c-abi/src/lib.rs`, with declarations in `crates/c-abi/include/freemvvm/abi.h`. Focused test names are `window_background_color_is_core_owned_validated_and_forwarded_to_scene` and `window_background_color_abi_supports_get_set_and_reset_atomically`.
- Do not consider wrapper-level background color APIs or physical Platform Adapter rendering validated by the Core/C ABI unit.
