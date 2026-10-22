# Scope: C++ Runtime wrapper implementation and unit verification

- The approved C++ wrapper v1.2 starts with a move-only `Runtime`, non-throw `Result<T>`, explicit shutdown/request/close, and a per-Runtime cleanup state retained by the process-wide supervisor after destructor release failure.
- On Windows, initialize the pinned compiler environment through `python tools/dev.py cpp-runtime-test`; invoking the generated CMake build directly from an ordinary PowerShell may lack MSVC include/library variables.
- `UT-WRAP-CPP-RT-001` covers successful create/UI-thread query, pending shutdown/request query, explicit close, unknown raw status plus owned error snapshot, close retry, and supervisor retry after destructor failure. It does not establish Linux or CI success; package install/export remains a separate implementation unit.
- When constructing a private `ErrorSnapshot` inside `std::optional`, construct the temporary in its friend factory and move-assign it; `optional::emplace` checks constructor access from the library template context and cannot use the factory's friendship.
- Keep the native ABI imported target and its link dependency outside `if(BUILD_TESTING)`; otherwise the product wrapper silently loses its native link dependency in test-disabled builds. The focused command also configures and builds `freemvvm_cpp` with `BUILD_TESTING=OFF` to catch this regression.
