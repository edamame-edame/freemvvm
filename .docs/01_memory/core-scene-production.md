# Scope: Core UI scene production

- The current Core has Logical Tree, Control/Layout state, `SceneSnapshot`, and frame submission/completion contracts, but the production path from a Window/Visual tree to a `SceneSnapshot` is not implemented; current scenes are constructed directly in tests.
- Platform DD explicitly leaves Visual-tree traversal and scene construction to a later unit. Keep layout/hit-test bounds and renderer geometry on one revision, and preserve Core ownership of immutable snapshots.
- The Linux GLES backend currently renders Clip/FillRect and rejects GlyphRun. A Text UI path needs an explicit TextService/glyph data/render dependency; do not silently drop text or mark unsupported operations presented.
- Keep hardware DRM/KMS confirmation separate from Core scene construction; Docker llvmpipe tests establish only software rendering for the exercised pixels.
