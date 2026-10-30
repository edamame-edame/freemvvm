# [platform/linux-backend] DRM/KMS renderer lifecycle and verification boundaries

Scope: implementation notes for the first Linux DRM/KMS + GBM/EGL/OpenGL ES platform adapter in FreeMvvm.

- `SceneSnapshot` geometry is DIP-based; the Core-owned `DeviceScale` is passed unchanged to the renderer and applied per axis when mapping rectangle edges to surface pixels.
- `Clip` restricts visibility only. It does not alter child layout size or PAINT/event semantics. If the scene producer emits nested clips as replacements, it must provide the effective ancestor intersection.
- GLES/EGL objects are created, used, and dropped on the owning platform worker thread while its context is current. Keep GBM buffers and DRM framebuffer handles alive until scanout has moved to another buffer or the previous CRTC has been restored.
- DRM page-flip submission is asynchronous: only its matching completion may complete the Core frame ticket. Do not close/destroy scanout resources while a flip is pending.
- Fake adapter tests and Linux-target cross-checks do not prove real DRM/GPU presentation. Debian/Raspberry Pi hardware verification remains a separate profile test.
- The Core supplies an opaque normalized RGB background in each `SceneSnapshot`; GLES clears the complete surface to this value before applying draw operations. Do not restore a renderer-local hard-coded clear color.

Related records: [Platform implementation plan](../20_todo/platform-first-backend.md), [Platform internal layers](../13_detail_design/platform/internal-layers.md), and [UI unit-test design](../32_unit_test/ui.md).
