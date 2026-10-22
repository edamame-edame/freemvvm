# Initial C++ wrapper implementation. Install/export and relocatable package
# configuration are tracked as the following implementation unit.
add_library(freemvvm_cpp STATIC src/runtime.cpp)
add_library(freemvvm::cpp ALIAS freemvvm_cpp)
target_compile_features(freemvvm_cpp INTERFACE cxx_std_17)
target_compile_features(freemvvm_cpp PUBLIC cxx_std_17)
target_include_directories(freemvvm_cpp
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/crates/c-abi/include)
