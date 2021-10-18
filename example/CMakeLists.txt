cmake_minimum_required(VERSION 3.18)
project(mockery-example CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(${CMAKE_BINARY_DIR}/conan_paths.cmake)

add_subdirectory(src)
add_subdirectory(test)
