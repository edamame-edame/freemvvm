# Scope: freemvvm Linux Wayland Window API and build-profile review

## 2026-11-14 status

- The user approved the API/build direction for Linux Wayland Window initial client size: retain existing create APIs, add a size-aware C ABI entry point guarded by feature 10, expose options in Rust/C++/C#/Python, and build Wayland and DRM/KMS artifacts as separate profiles.
- The proposal was applied to seven canonical DDs: common ABI 1.8, Platform internal layers 3.0, build/toolchain packages 1.3, and Rust/C++/C#/Python wrappers 1.9/1.6/1.7/2.3. The user approved all seven documents as a whole on 2026-11-14; implementation and test status remain separate.
- Before unit/test catalogs, dependencies, source, or implementation change, obtain approval of the revised implementation plan 0.2. Once approved, keep source and corresponding tests on the same revision and follow the documented Core-before-wrapper branch order.
- Rust wrapper linking is direct to native ABI symbols; the Rust wrapper and native artifact must have a compatible minimum version. C++, C#, and Python resolve the new entry point optionally so legacy constructors can still use existing symbols when feature 10 is absent.
- Wayland profile candidates are Debian 13, Debian 12, Ubuntu 24.04, and Ubuntu 26.04 x64. Profile-specific build evidence does not itself establish runtime support; each VM/session requires visible presentation, configure resize/redraw, and close/reopen evidence.

References: [API/build proposal](../20_todo/linux-wayland-window-api-proposal.md), [implementation plan](../20_todo/linux-wayland-window-implementation.md), [Platform feasibility findings](platform-linux-wayland-feasibility.md), [Git rules](../02_git/rule.md).
