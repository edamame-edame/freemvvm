include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

add_library(freemvvm_cpp STATIC src/runtime.cpp src/command.cpp src/ui_controls.cpp)
add_library(FreeMvvm::cpp ALIAS freemvvm_cpp)
set_target_properties(freemvvm_cpp PROPERTIES EXPORT_NAME cpp)
target_compile_features(freemvvm_cpp PUBLIC cxx_std_17)
target_include_directories(freemvvm_cpp
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
  PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/crates/c-abi/include)

set(FREEMVVM_CPP_PACKAGE_VERSION "0.0.0")
configure_package_config_file(
  cmake/FreeMvvmConfig.cmake.in
  "${CMAKE_CURRENT_BINARY_DIR}/FreeMvvmConfig.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/FreeMvvm")
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/FreeMvvmConfigVersion.cmake"
  VERSION "${FREEMVVM_CPP_PACKAGE_VERSION}"
  COMPATIBILITY SameMajorVersion)

install(TARGETS freemvvm_cpp EXPORT FreeMvvmTargets
  ARCHIVE DESTINATION "${CMAKE_INSTALL_LIBDIR}"
  INCLUDES DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(DIRECTORY include/ DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(EXPORT FreeMvvmTargets
  FILE FreeMvvmTargets.cmake
  NAMESPACE FreeMvvm::
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/FreeMvvm")
install(FILES
  "${CMAKE_CURRENT_BINARY_DIR}/FreeMvvmConfig.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/FreeMvvmConfigVersion.cmake"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/FreeMvvm")
