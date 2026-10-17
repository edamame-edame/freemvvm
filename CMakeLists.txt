cmake_minimum_required(VERSION 3.28)
project(freemvvm_bootstrap LANGUAGES C CXX)

include(CTest)
add_subdirectory(wrappers/cpp)

if(BUILD_TESTING)
  add_executable(freemvvm_c_environment_probe tools/probes/c_environment.c)
  set_target_properties(freemvvm_c_environment_probe PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
  add_executable(freemvvm_cpp_environment_probe tools/probes/cpp_environment.cpp)
  target_link_libraries(freemvvm_cpp_environment_probe PRIVATE freemvvm_cpp)
  if(MSVC)
    target_compile_options(freemvvm_cpp_environment_probe PRIVATE /Zc:__cplusplus)
  endif()
  add_test(NAME environment.c COMMAND freemvvm_c_environment_probe)
  add_test(NAME environment.cpp COMMAND freemvvm_cpp_environment_probe)
  set_tests_properties(environment.c environment.cpp PROPERTIES LABELS "bootstrap")
endif()
