include(${CMAKE_BINARY_DIR}/conan_paths.cmake)
find_package(GTest)
include(GoogleTest)

add_executable(example-tests)
gtest_discover_tests(example-tests)

target_sources(example-tests
    PRIVATE
        ProjectTest.cpp
)

target_link_libraries(example-tests
    PRIVATE
        project
        GTest::GTest
)
