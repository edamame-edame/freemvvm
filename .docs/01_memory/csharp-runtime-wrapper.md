# Scope: C# Runtime wrapper implementation

- The first C# Runtime unit targets .NET 10 / C# 14 and uses `LibraryImport` with fixed-width ABI types (`ulong` handle, `byte` bool, raw `uint` status).
- `UiRuntime.Dispose` releases the Runtime ownership only; shutdown must be requested explicitly. A shutdown request receipt is a separate owned handle and must be closed separately.
- `TokenOwner` gates native calls with short-lived leases. Explicit close failure keeps the token retryable; disposal/release failure transfers it to a process-wide cleanup queue for retry.
- The C ABI host without a registered UI loop rejects `runtime_request_shutdown` with `UI_UNSUPPORTED`; pending receipt states must be exercised with a controlled fake ABI rather than assumed from the no-loop integration host.
- C# fake ABI tests are a standalone no-third-party .NET console harness. The consumer test starts a separate process and takes an absolute path to the built host ABI library. `tools/dev.py check` runs both focused tests.
- The required `.docs/00_rule/coding_rule_CSharp.md` did not exist at implementation time; use the project’s C# 14/.NET 10 settings, nullable analysis, warnings-as-errors, approved wrapper DD, and explicit ABI tests until a repository C# rule is added.
- `UNIT-WRAP-CS-001` passed the focused fake-ABI tests, real C ABI consumer, Windows `tools/dev.py check`, and `tools/dev.py minimum`. GitHub Actions run 39994623749 passed on both Windows and Linux for implementation revision `0fa8cc73cc18cf8eba2cf9bcffc07b809fcda20a`.
