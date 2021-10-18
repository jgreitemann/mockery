add_executable(example-app)

target_sources(example-app PRIVATE main.cpp)
target_link_libraries(example-app PRIVATE project)
