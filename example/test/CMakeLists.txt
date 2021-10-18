find_package(GTest)

add_executable(example-tests)

target_sources(example-tests
    PRIVATE
        ProjectTest.cpp
)

target_link_libraries(example-tests
    PRIVATE
        project
        GTest::GTest
)
