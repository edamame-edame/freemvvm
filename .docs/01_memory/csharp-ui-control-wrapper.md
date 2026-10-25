<!-- scope: C# UI Control wrapper, C ABI interop, token ownership, and consumer verification -->
# C# UI Control wrapper memory

- C ABI `ui_handle` and child indexes are `ulong`, enum parameters are `uint`, and any `ui_bool` is `byte`; keep these widths identical in `INativeApi` and `LibraryImport` declarations.
- UTF-8 control constructors pin encoded `byte[]` only for the duration of the native call and pass explicit byte length. Use strict UTF-8 encoding so invalid managed surrogate sequences are rejected before entering the ABI.
- For any C ABI function that returns an owned out-handle, allocate its `TokenOwner` before the call and adopt a nonzero result immediately, including when status is failure. Let `Dispose` move a failed release into its retry ticket; do not raw-release the token in catch/error handling.
- A child retrieved from `window_content` or `container_child_at` is returned as the generic `UiWidget` unless the ABI provides a stable control-kind query. Do not infer concrete managed type from creation history or token value.
- Existing RuntimeTests is a custom executable fake-ABI harness. `tools/dev.py csharp-controls-test` runs it; `csharp-controls-consumer-test` builds the host C ABI and uses a separate C# process for tree/layout/property-binding/command ownership integration without claiming native rendering.
- C# UI Control facade implementation is on `impl-csharp/ui-controls-wrapper`, based on the Core APIs child branch. Local focused and real C ABI consumer checks passed on the first Controls implementation revision; record CI and final commit in the plan after push.
