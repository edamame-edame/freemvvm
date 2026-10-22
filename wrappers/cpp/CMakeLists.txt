# Build-only skeleton. Install/export and ABI linkage follow the implemented API.
add_library(freemvvm_cpp INTERFACE)
add_library(freemvvm::cpp ALIAS freemvvm_cpp)
target_compile_features(freemvvm_cpp INTERFACE cxx_std_17)
