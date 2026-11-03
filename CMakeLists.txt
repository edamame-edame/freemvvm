cmake_minimum_required(VERSION 3.28)
project(freemvvm_bootstrap LANGUAGES C CXX)

include(CTest)
include(GNUInstallDirs)
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
add_library(FreeMvvm::runtime SHARED IMPORTED GLOBAL)
set_target_properties(FreeMvvm::runtime PROPERTIES
  IMPORTED_LOCATION "${FREEMVVM_ABI_RUNTIME}")
if(WIN32)
  set_target_properties(FreeMvvm::runtime PROPERTIES
    IMPORTED_IMPLIB "${FREEMVVM_ABI_IMPORT_LIBRARY}")
endif()
target_link_libraries(freemvvm_cpp PUBLIC FreeMvvm::runtime Threads::Threads)
add_dependencies(freemvvm_cpp freemvvm_abi_native_build)

if(WIN32)
  install(FILES "${FREEMVVM_ABI_RUNTIME}" DESTINATION "${CMAKE_INSTALL_BINDIR}")
  install(FILES "${FREEMVVM_ABI_IMPORT_LIBRARY}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")
else()
  install(FILES "${FREEMVVM_ABI_RUNTIME}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")
endif()

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
  target_link_libraries(freemvvm_abi_runtime_smoke_exe PRIVATE FreeMvvm::runtime)
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
  add_executable(freemvvm_abi_command_smoke_exe
    tools/probes/abi_command_smoke.c)
  target_include_directories(freemvvm_abi_command_smoke_exe PRIVATE
    crates/c-abi/include)
  target_link_libraries(freemvvm_abi_command_smoke_exe PRIVATE FreeMvvm::runtime)
  add_dependencies(freemvvm_abi_command_smoke_exe freemvvm_abi_native_build)
  set_target_properties(freemvvm_abi_command_smoke_exe PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
  if(WIN32)
    add_custom_command(TARGET freemvvm_abi_command_smoke_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_abi_command_smoke_exe>"
      VERBATIM)
  endif()
  add_test(NAME abi.command-smoke COMMAND freemvvm_abi_command_smoke_exe)
  set_tests_properties(abi.command-smoke PROPERTIES LABELS "abi;command")
  add_custom_target(freemvvm_abi_command_smoke
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^abi.command-smoke$" --output-on-failure
    DEPENDS freemvvm_abi_command_smoke_exe
    VERBATIM)
  add_executable(freemvvm_abi_cpp_runtime_smoke_exe
    tools/probes/abi_cpp_runtime_smoke.cpp)
  target_include_directories(freemvvm_abi_cpp_runtime_smoke_exe PRIVATE
    crates/c-abi/include)
  target_link_libraries(freemvvm_abi_cpp_runtime_smoke_exe PRIVATE
    freemvvm_cpp FreeMvvm::runtime)
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
  add_executable(freemvvm_cpp_property_tests_exe
    wrappers/cpp/tests/property_tests.cpp)
  target_include_directories(freemvvm_cpp_property_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_property_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_property_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_property_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_property_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property COMMAND freemvvm_cpp_property_tests_exe)
  set_tests_properties(wrapper.cpp.property PROPERTIES LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_property_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property$" --output-on-failure
    DEPENDS freemvvm_cpp_property_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_utf8_property_tests_exe
    wrappers/cpp/tests/utf8_property_tests.cpp)
  target_include_directories(freemvvm_cpp_utf8_property_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_utf8_property_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_utf8_property_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_utf8_property_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_utf8_property_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property.utf8
    COMMAND freemvvm_cpp_utf8_property_tests_exe)
  set_tests_properties(wrapper.cpp.property.utf8 PROPERTIES LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_utf8_property_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property\\.utf8$" --output-on-failure
    DEPENDS freemvvm_cpp_utf8_property_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_f64_property_tests_exe
    wrappers/cpp/tests/f64_property_tests.cpp)
  target_include_directories(freemvvm_cpp_f64_property_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_f64_property_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_f64_property_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_f64_property_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_f64_property_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property.f64
    COMMAND freemvvm_cpp_f64_property_tests_exe)
  set_tests_properties(wrapper.cpp.property.f64 PROPERTIES LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_f64_property_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property\\.f64$" --output-on-failure
    DEPENDS freemvvm_cpp_f64_property_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_decimal64_property_tests_exe
    wrappers/cpp/tests/decimal64_property_tests.cpp)
  target_include_directories(freemvvm_cpp_decimal64_property_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_decimal64_property_tests_exe PRIVATE
    freemvvm_cpp)
  add_dependencies(freemvvm_cpp_decimal64_property_tests_exe
    freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_decimal64_property_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_decimal64_property_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property.decimal
    COMMAND freemvvm_cpp_decimal64_property_tests_exe)
  set_tests_properties(wrapper.cpp.property.decimal PROPERTIES
    LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_decimal64_property_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property\\.decimal$" --output-on-failure
    DEPENDS freemvvm_cpp_decimal64_property_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_property_subscription_tests_exe
    wrappers/cpp/tests/property_subscription_tests.cpp)
  target_include_directories(freemvvm_cpp_property_subscription_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_property_subscription_tests_exe PRIVATE
    freemvvm_cpp)
  add_dependencies(freemvvm_cpp_property_subscription_tests_exe
    freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_property_subscription_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_property_subscription_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property.subscription
    COMMAND freemvvm_cpp_property_subscription_tests_exe)
  set_tests_properties(wrapper.cpp.property.subscription PROPERTIES
    LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_property_subscription_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property\\.subscription$" --output-on-failure
    DEPENDS freemvvm_cpp_property_subscription_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_property_binding_tests_exe
    wrappers/cpp/tests/property_binding_tests.cpp)
  target_include_directories(freemvvm_cpp_property_binding_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_property_binding_tests_exe PRIVATE
    freemvvm_cpp)
  add_dependencies(freemvvm_cpp_property_binding_tests_exe
    freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_property_binding_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_property_binding_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.property.binding
    COMMAND freemvvm_cpp_property_binding_tests_exe)
  set_tests_properties(wrapper.cpp.property.binding PROPERTIES
    LABELS "wrapper;unit")
  add_custom_target(freemvvm_cpp_property_binding_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.property\\.binding$" --output-on-failure
    DEPENDS freemvvm_cpp_property_binding_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_command_tests_exe
    wrappers/cpp/tests/command_tests.cpp)
  target_include_directories(freemvvm_cpp_command_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_command_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_command_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_command_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_command_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.command COMMAND freemvvm_cpp_command_tests_exe)
  set_tests_properties(wrapper.cpp.command PROPERTIES
    LABELS "wrapper;unit;command")
  add_custom_target(freemvvm_cpp_command_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.command$" --output-on-failure
    DEPENDS freemvvm_cpp_command_tests_exe
    VERBATIM)
  add_executable(freemvvm_cpp_ui_control_tests_exe
    wrappers/cpp/tests/ui_control_tests.cpp)
  target_include_directories(freemvvm_cpp_ui_control_tests_exe PRIVATE
    wrappers/cpp/src crates/c-abi/include)
  target_link_libraries(freemvvm_cpp_ui_control_tests_exe PRIVATE freemvvm_cpp)
  add_dependencies(freemvvm_cpp_ui_control_tests_exe freemvvm_abi_native_build)
  if(WIN32)
    add_custom_command(TARGET freemvvm_cpp_ui_control_tests_exe POST_BUILD
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${FREEMVVM_ABI_RUNTIME}"
        "$<TARGET_FILE_DIR:freemvvm_cpp_ui_control_tests_exe>"
      VERBATIM)
  endif()
  add_test(NAME wrapper.cpp.ui-controls
    COMMAND freemvvm_cpp_ui_control_tests_exe)
  set_tests_properties(wrapper.cpp.ui-controls PROPERTIES
    LABELS "wrapper;unit;ui")
  add_custom_target(freemvvm_cpp_ui_control_tests
    COMMAND "${CMAKE_CTEST_COMMAND}" --test-dir "${CMAKE_BINARY_DIR}"
      -R "^wrapper\\.cpp\\.ui-controls$" --output-on-failure
    DEPENDS freemvvm_cpp_ui_control_tests_exe
    VERBATIM)
  if(MSVC)
    target_compile_options(freemvvm_cpp_environment_probe PRIVATE /Zc:__cplusplus)
  endif()
  add_test(NAME environment.c COMMAND freemvvm_c_environment_probe)
  add_test(NAME environment.cpp COMMAND freemvvm_cpp_environment_probe)
  set_tests_properties(environment.c environment.cpp PROPERTIES LABELS "bootstrap")
endif()
