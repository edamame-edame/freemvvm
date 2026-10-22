cmake_minimum_required(VERSION 3.28)
project(freemvvm_cpp_package_consumer LANGUAGES CXX)

include(CTest)
find_package(FreeMvvm CONFIG REQUIRED)

add_executable(freemvvm_cpp_package_consumer main.cpp)
target_link_libraries(freemvvm_cpp_package_consumer PRIVATE FreeMvvm::cpp)

if(WIN32)
  add_custom_command(TARGET freemvvm_cpp_package_consumer POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy_if_different
      $<TARGET_RUNTIME_DLLS:freemvvm_cpp_package_consumer>
      "$<TARGET_FILE_DIR:freemvvm_cpp_package_consumer>"
    COMMAND_EXPAND_LISTS
    VERBATIM)
endif()

add_test(NAME package.consumer.runtime COMMAND freemvvm_cpp_package_consumer)
