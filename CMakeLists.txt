cmake_minimum_required(VERSION 3.28)
project(freemvvm_bootstrap LANGUAGES C CXX)

include(CTest)
find_package(Threads REQUIRED)
add_subdirectory(wrappers/cpp)

if(WIN32)
  set(FREEMVVM_ABI_RUNTIME "${CMAKE_SOURCE_DIR}/target/debug/freemvvm_abi_1.dll")
  set(FREEMVVM_ABI_IMPORT_LIBRARY "${CMAKE_SOURCE_DIR}/target/debug/freemvvm_abi_1.dll.lib")
  set(FREEMVVM_ABI_BYPRODUCTS
    "${FREEMVVM_ABI_RUNTIME}"
    "${FREEMVVM_ABI_IMPORT_LIBRARY}")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  set(FREEMVVM_ABI_RUNTIME "${CMAKE_SOURCE_DIR}/target/debug/libfreemvvm_abi_1.so")
  set(FREEMVVM_ABI_IMPORT_LIBRARY "${FREEMVVM_ABI_RUNTIME}")
  set(FREEMVVM_ABI_BYPRODUCTS "${FREEMVVM_ABI_RUNTIME}")
else()
  message(FATAL_ERROR "The native C ABI target currently supports Windows and Linux hosts only")
endif()
find_program(FREEMVVM_CARGO_EXECUTABLE cargo REQUIRED)
add_custom_target(freemvvm_abi_native_build
  COMMAND "${FREEMVVM_CARGO_EXECUTABLE}" build --locked -p freemvvm-c-abi
  WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
  BYPRODUCTS ${FREEMVVM_ABI_BYPRODUCTS}
  VERBATIM)
add_library(freemvvm_abi_native SHARED IMPORTED GLOBAL)
set_target_properties(freemvvm_abi_native PROPERTIES
  IMPORTED_LOCATION "${FREEMVVM_ABI_RUNTIME}")
if(WIN32)
  set_target_properties(freemvvm_abi_native PROPERTIES
    IMPORTED_IMPLIB "${FREEMVVM_ABI_IMPORT_LIBRARY}")
endif()
target_link_libraries(freemvvm_cpp PUBLIC freemvvm_abi_native Threads::Threads)
add_dependencies(freemvvm_cpp freemvvm_abi_native_build)

if(BUILD_TESTING)
  add_executable(freemvvm_c_environment_probe tools/probes/c_environment.c)
  set_target_properties(freemvvm_c_environment_probe PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
  add_executable(freemvvm_cpp_environment_probe tools/probes/cpp_environment.cpp)
  target_compile_features(freemvvm_cpp_environment_probe PRIVATE cxx_std_17)
  add_library(freemvvm_abi_header_c_probe OBJECT tools/probes/abi_header_c.c)
  add_library(freemvvm_abi_header_cpp_probe OBJECT tools/probes/abi_header_cpp.cpp)
  target_include_directories(freemvvm_abi_header_c_probe PRIVATE crates/c-abi/include)
  target_include_directories(freemvvm_abi_header_cpp_probe PRIVATE crates/c-abi/include)
  target_compile_features(freemvvm_abi_header_cpp_probe PRIVATE cxx_std_17)
  set_target_properties(freemvvm_abi_header_c_probe PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
  add_custom_target(freemvvm_abi_header_probes
    DEPENDS freemvvm_abi_header_c_probe freemvvm_abi_header_cpp_probe)
  add_executable(freemvvm_abi_runtime_smoke_exe tools/probes/abi_runtime_smoke.c)
  target_include_directories(freemvvm_abi_runtime_smoke_exe PRIVATE crates/c-abi/include)
  target_link_libraries(freemvvm_abi_runtime_smoke_exe PRIVATE freemvvm_abi_native)
  add_dependencies(freemvvm_abi_runtime_smoke_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_abi_runtime_smoke_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}" "$<TARGET_FILE_DIR:freemvvm_abi_runtime_smoke_exe>"
      VERBATIM)
  endif()
  add_test(NAME abi.runtime-smoke COMMAND freemvvm_abi_runtime_smoke_exe)
  set_tests_properties(abi.runtime-smoke PROPERTIES LABELS "abi")
  add_custom_target(freemvvm_abi_runtime_smoke
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^abi.runtime-smoke$" --output-on-failure
    DEPENDS freemvvm_abi_runtime_smoke_exe
    VERBATIM)
  add_executable(freemvvm_abi_cpp_runtime_smoke_exe
    tools/probes/abi_cpp_runtime_smoke.cpp)
  target_include_directories(freemvvm_abi_cpp_runtime_smoke_exe PRIVATE
    crates/c-abi/include)
  target_link_libraries(freemvvm_abi_cpp_runtime_smoke_exe PRIVATE
    freemvvm_cpp freemvvm_abi_native)
  add_dependencies(freemvvm_abi_cpp_runtime_smoke_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_abi_cpp_runtime_smoke_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_abi_cpp_runtime_smoke_exe>"
      VERBATIM)
  endif()
  add_test(NAME abi.runtime-smoke-cpp
    COMMAND freemvvm_abi_cpp_runtime_smoke_exe)
  set_tests_properties(abi.runtime-smoke-cpp PROPERTIES LABELS "abi")
  add_custom_target(freemvvm_abi_cpp_runtime_smoke
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^abi.runtime-smoke-cpp$" --output-on-failure
    DEPENDS freemvvm_abi_cpp_runtime_smoke_exe
    VERBATIM)
  add_executable(freemvvm_cpp_runtime_tests_exe
    wrappers/cpp/tests/runtime_tests.cpp)
  target_include_directories(freemvvm_cpp_runtime_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_runtime_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_runtime_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_runtime_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_runtime_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.runtime COMMAND freemvvm_cpp_runtime_tests_exe)
  set_tests_properties(wrapper.cpp.runtime PROPERTIES LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_runtime_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.runtime$" --output-on-failure
    DEPENDS freemvvm_cpp_runtime_tests_exe
    VERBATIM)
  if(MSVC)
    target_compile_options(freemvvm_cpp_environment_probe PRIVATE /Zc:__cplusplus)
  endif()
  add_test(NAME environment.c COMMAND freemvvm_c_environment_probe)
  add_test(NAME environment.cpp COMMAND freemvvm_cpp_environment_probe)
  set_tests_properties(environment.c environment.cpp PROPERTIES LABELS "bootstrap")
endif()
