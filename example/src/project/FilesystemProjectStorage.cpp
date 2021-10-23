#include <project/FilesystemProjectStorage.h>

#include <algorithm>
#include <fstream>

static Type TypeOfFileAtPath(std::filesystem::path const &path) {
  if (std::filesystem::is_regular_file(path))
    return Type::File;
  else if (std::filesystem::is_directory(path))
    return Type::Directory;
  else if (std::filesystem::is_symlink(path))
    return Type::SymLink;
  else
    return Type::Unknown;
}

bool FilesystemProjectStorage::Open(std::string_view path) {
  auto fs_path = std::filesystem::u8path(path);
  if (std::filesystem::is_directory(fs_path)) {
    m_ProjectDir = fs_path;
    return true;
  } else {
    return false;
  }
}

bool FilesystemProjectStorage::Close() {
  if (m_ProjectDir) {
    m_ProjectDir.reset();
    return true;
  } else {
    return false;
  }
}

bool FilesystemProjectStorage::IsOpen() const noexcept {
  return m_ProjectDir.has_value();
}

auto FilesystemProjectStorage::List() const -> std::map<std::string, Type> {
  if (!IsOpen())
    throw std::runtime_error{"Project is not open."};

  std::map<std::string, Type> files_and_dates;
  std::filesystem::recursive_directory_iterator dir{*m_ProjectDir};

  std::transform(begin(dir), end(dir),
                 std::inserter(files_and_dates, files_and_dates.end()),
                 [](std::filesystem::path const &path) {
                     return std::pair{path.u8string(),
                                      TypeOfFileAtPath(path)};
                 });
  return files_and_dates;
}

std::string FilesystemProjectStorage::ReadFile(std::string_view name) const {
  if (!IsOpen())
    throw std::runtime_error{"Project is not open."};

  std::ifstream is{*m_ProjectDir / name};
  return std::string{std::istreambuf_iterator<char>{is}, std::istreambuf_iterator<char>{}};
}

void FilesystemProjectStorage::SaveFile(std::string_view name, std::string const &contents) {
  if (!IsOpen())
    throw std::runtime_error{"Project is not open."};

  std::ofstream os{*m_ProjectDir / name};
  os << contents;
}
