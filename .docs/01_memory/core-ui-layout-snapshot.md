# Scope: Core UI Control tree to immutable LayoutSnapshot integration

- Keep layout input capture UI-thread-confined. Copy the Window content tree and Control orientation/child Align settings into an immutable `LayoutNodeSnapshot` before calling Measure/Arrange.
- `RuntimeTreeStore` has a monotonic tree revision. A layout pass revalidates tree and Control configuration after measurement; if input changed, preserve the previous complete snapshot and dirty state.
- Store the complete `LayoutSnapshot` on `WindowLifecycle`; publish and clear participating dirty flags only after the current open generation, client area, and captured inputs still match.
- Until TextService provides real intrinsic metrics, production must not fabricate leaf sizes. Tests may inject a fake internal `IntrinsicMeasureProvider`; Text/TextBox/Button measurement without a real provider must fail the pass.
- Layout integration does not create a public C ABI, Scene producer, or claim Text rendering/hardware presentation. Those remain separate units.
