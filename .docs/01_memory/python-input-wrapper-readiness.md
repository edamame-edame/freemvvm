<!-- scope: Python Input wrapper readiness, public Runtime/UI prerequisites, and branch planning -->
# Python Input wrapper readiness

- The Python wrapper baseline exists on `impl-python-wrapper/window-background-config` at `3e018a0`: Runtime, Property, Binding, Command, Controls/Tree/Focus, Window, native API, examples, and tests. Its CI run `41178355548` succeeded.
- That branch does not yet contain `subscribe_input`, `InputEventView`, or the Input callback trampoline/root lifecycle. Keep the scope to this approved Input DD feature; do not recreate Runtime/Window/Widget APIs.
- For Python Input work, branch from Core Input `impl-rust/core-input-focus`, then integrate the existing same-language Python wrapper branch. Do not inherit a different language wrapper.
- The shared plan `.docs/20_todo/input-focus-event-path-implementation.md` authorizes the four-language Input wrapper phase. The Python-specific plan `.docs/20_todo/python-input-event-wrapper.md` was approved on 2026-11-05; implementation, tests, commits `a8a54dd`/`22a879f`, push, and CI run `42054917806` are complete.
- Python callback roots use the shared userdata registry and destroy callback; destroy removes the root from the registry and defers its final drop to `_CleanupManager.retry_pending()` safe points. A Python 3.13 system interpreter with pytest may be available even when the pinned repository `.venv` is absent; package installation can be blocked by DNS, so report that limitation separately from focused test results.
- Linux bootstrap CI installs `fonts-noto-cjk` (61.2 MB) from `azure.archive.ubuntu.com`; a slow package transfer once hit the 20-minute job timeout. Rerunning the job succeeded, so check whether Linux environment preparation advanced before treating this as a code failure.
