cmake_minimum_required(VERSION 3.21)
project(ameya_nana_validation LANGUAGES CXX)
include(CTest)

# The C++ harness validates the client-area snapshot emitted by the Nana view
# builder. It is intentionally independent from the Rust host toolchain.
add_subdirectory(validation)
