<!-- scope: C# UI Control wrapper, C ABI interop, token ownership, and consumer verification -->
# C# UI Control wrapper memory

- C ABI `ui_handle` and child indexes are `ulong`, enum parameters are `uint`, and any `ui_bool` is `byte`; keep these widths identical in `INativeApi` and `LibraryImport` declarations.
- UTF-8 control constructors pin encoded `byte[]` only for the duration of the native call and pass explicit byte length. Use strict UTF-8 encoding so invalid managed surrogate sequences are rejected before entering the ABI.
- For any C ABI function that returns an owned out-handle, allocate its `TokenOwner` before the call and adopt a nonzero result immediately, including when status is failure. Let `Dispose` move a failed release into its retry ticket; do not raw-release the token in catch/error handling.
- A child retrieved from `window_content` or `container_child_at` is returned as the generic `UiWidget` unless the ABI provides a stable control-kind query. Do not infer concrete managed type from creation history or token value.
- A close-request callback receives a borrowed token that is valid only during the callback. Invalidate its managed scope in `finally`; `window_close_request_defer` must create a separate `TokenOwner` before the ABI call and an independent retained Runtime anchor.
- The close callback GC root owns a Runtime anchor until the C ABI destroy callback runs. Callback exceptions go to `UiCallbackFailureKind.WindowCloseRequest` and must never cross the unmanaged boundary.
- With no native peer, `window_open` reports `UI_UNSUPPORTED`; `window_close` on a Window that is already closed is idempotent success. Preserve the Core/C ABI's actual state-dependent behavior instead of expecting both operations to reject.
- Existing RuntimeTests is a custom executable fake-ABI harness. `tools/dev.py csharp-controls-test` runs it; `csharp-controls-consumer-test` builds the host C ABI and uses a separate C# process for tree/layout/property-binding/command ownership integration without claiming native rendering.
- C# UI Control facade implementation is on `impl-csharp/ui-controls-wrapper`, based on the Core APIs child branch. Controls focused and real C ABI consumer checks passed locally, and commit `8d8ca6c` passed Windows/Linux CI run `40168746235`.
