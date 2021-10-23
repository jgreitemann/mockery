cmake_minimum_required(VERSION 3.18)
project(mockery-example CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

enable_testing()

add_subdirectory(src)

if(BUILD_TESTING)
    add_subdirectory(test)
endif()