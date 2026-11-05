<!-- scope: Python Input wrapper readiness, public Runtime/UI prerequisites, and branch planning -->
# Python Input wrapper readiness

- The Python wrapper baseline exists on `impl-python-wrapper/window-background-config` at `3e018a0`: Runtime, Property, Binding, Command, Controls/Tree/Focus, Window, native API, examples, and tests. Its CI run `41178355548` succeeded.
- That branch does not yet contain `subscribe_input`, `InputEventView`, or the Input callback trampoline/root lifecycle. Keep the scope to this approved Input DD feature; do not recreate Runtime/Window/Widget APIs.
- For Python Input work, branch from Core Input `impl-rust/core-input-focus`, then integrate the existing same-language Python wrapper branch. Do not inherit a different language wrapper.
- The shared plan `.docs/20_todo/input-focus-event-path-implementation.md` authorizes the four-language Input wrapper phase. The Python-specific plan `.docs/20_todo/python-input-event-wrapper.md` was approved on 2026-11-05 and its implementation/tests are complete locally; commit, push, and CI confirmation remain pending.
- Python callback roots use the shared userdata registry and destroy callback; destroy removes the root from the registry and defers its final drop to `_CleanupManager.retry_pending()` safe points. A Python 3.13 system interpreter with pytest may be available even when the pinned repository `.venv` is absent; package installation can be blocked by DNS, so report that limitation separately from focused test results.
