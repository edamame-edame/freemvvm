# Scope: C++ native ABI smoke

- A C++17 consumer that includes the public C ABI header and links/runs against the Rust cdylib verifies the C-linkage and native loader path without prematurely defining the public C++ wrapper surface.
- Keep C++ public type/namespace naming and exception/RAII API choices in the approved C++ wrapper design process; raw ABI link smoke is not wrapper completion.
- For the tested Windows host, direct C++17 linking and CTest loading succeeded with the MSVC import library; Linux is included in the build graph but still needs CI evidence.
