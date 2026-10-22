# Scope: Native C ABI smoke linking

- Build a host-native C consumer against Cargo's generated `freemvvm_abi_1` cdylib; keep Windows import-library and Linux shared-object paths distinct.
- Ensure the Cargo build is a dependency of the CMake smoke executable, and copy the Windows DLL next to the executable so CTest exercises the loader as well as the linker.
- A version/create/shutdown/release smoke proves only the tested host-native ABI path; it does not establish release support or an installable CMake package.
- Runtime shutdown requires the adopted host-loop lifecycle. Register a UI-loop bridge, enqueue an asynchronous shutdown request inside a host event, then end the outer event and query the completion receipt; calling synchronous shutdown directly from the creator/UI thread returns `UI_PENDING` until the host services it.
