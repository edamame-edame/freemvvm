<!-- scope: C# async Command wrapper, native ownership, worker post, and verification -->
# C# asynchronous Command wrapper memory

- `UiCommand` deliberately separates `TryClose()` (stop future executions and request cancellation) from `Dispose()` (release the public handle). Handle release alone must not close the Core-owned registration.
- The C# start callback must retain the borrowed Command and cancellation token, plus an independent Runtime token, before transferring a `UiCommandWorker` to app code. Each terminal post owns a separate retained Command token so the worker may dispose immediately after enqueue.
- `runtime_post` transfers callback-root ownership only when accepted. On post rejection, free the local GCHandle and retained Command reference and permit a retry; after acceptance, only Core's destroy callback frees the root.
- `UiErrorSnapshot` copies status/domain/code/message for managed reads and can optionally retain the native snapshot token until `Dispose()`. Plain snapshots from ordinary wrapper errors have no native owner.
- The C# RuntimeTests project is a custom executable harness. Use `tools/dev.py csharp-command-test` and `csharp-command-consumer-test`; the latter builds the host C ABI and validates UI-thread dispatch against the real library.
- The first complete C# Command unit passed `tools/dev.py check`, `minimum`, and Windows/Linux CI run 40074886918 on commit `fa49bac`.
- Keep C# wrapper work on a language branch descended from the relevant Core APIs branch; do not derive it from another wrapper-language branch.
