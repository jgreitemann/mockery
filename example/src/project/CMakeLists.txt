add_library(project STATIC)

target_sources(project
    PRIVATE
        Project.h
        Project.cpp
        ProjectStorage.h
        FilesystemProjectStorage.h
        FilesystemProjectStorage.cpp
)

target_include_directories(project PUBLIC ${PROJECT_SOURCE_DIR}/src)
