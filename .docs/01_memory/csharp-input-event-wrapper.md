<!-- scope: C# Input event wrapper, callback-only borrowed views, C ABI roots, and wrapper tests -->
# C# Input event wrapper memory

- The C ABI callback uses `void *userdata` first, then current widget, original target, and borrowed event handles. The C# unmanaged trampoline must preserve this order even when the public delegate exposes only the event view.
- Treat `InputEventView` as callback-scoped managed state: invalidate its scope in `finally`; after return, properties throw and `Try*` accessors return `UI_INVALID_HANDLE`. Copy UTF-8 text into a managed string while the callback is active.
- The input callback GC root owns an independent Runtime anchor until C ABI destroy. On registration failure C# retains ownership and frees the root itself; after success the destroy callback releases root and anchor exactly once.
- `UiSubscription.TryCancel()` and `TryClose()` have separate meanings. Cancellation prevents later callback entry; releasing the subscription handle is not a substitute for cancellation.
- Delegate and failure-handler exceptions must remain inside the unmanaged callback boundary. Report delegate exceptions through the Runtime handler, and mark the current event handled even when the handler is absent or throws.
- The input UTF-8 getter uses a capacity query followed by a copy. C ABI returns `UI_BUFFER_TOO_SMALL` with required length for a non-empty zero-capacity query; an empty payload may return `UI_OK` with length zero.
- C# focused tests live in the existing `RuntimeTests` executable fake-ABI harness. `csharp-input-consumer-test` selects an input-only path in the separate Consumer executable so Window peer availability in the broader Controls consumer does not determine Input wrapper results.
