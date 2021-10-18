#include <project/Project.h>
#include <project/FilesystemProjectStorage.h>

int main() {
  Project proj{"my_project", std::make_unique<FilesystemProjectStorage>()};
}