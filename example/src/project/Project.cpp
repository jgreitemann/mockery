#include "Project.h"

#include <charconv>
#include <sstream>

Project::Project(std::string_view name, std::unique_ptr<ProjectStorage> storage)
        : m_Storage{std::move(storage)} {
  if (m_Storage->Open("/projects/" + std::string{name})) {
    ReadSettings();
  } else {
    throw ProjectNotFound{};
  }
}

Project::~Project() {
  if (m_UnsavedSettings) {
    WriteSettings();
  }
  m_Storage->Close();
}

bool Project::ReadSettings() {
  if (auto list = m_Storage->List(); list.count(std::string{SettingsFilename})) {
    std::istringstream is{m_Storage->ReadFile(SettingsFilename)};
    std::string line;
    while (std::getline(is, line)) {
      std::string_view line_view = line;
      if (auto eq = line_view.find('='); eq != std::string_view::npos) {
        auto key = line_view.substr(0, eq);
        auto val = line_view.substr(eq + 1);
        if (key == "verbose") {
          int verbosity{};
          auto[_, ec] = std::from_chars(val.data(), val.data() + val.size(), verbosity);
          if (ec == std::errc{}) {
            m_Verbosity = verbosity;
            m_UnsavedSettings = false;
          }
        }
      }
    }
    return true;
  } else {
    return false;
  }
}

void Project::WriteSettings() {
  m_Storage->SaveFile(SettingsFilename, "verbose=" + std::to_string(m_Verbosity));
  m_UnsavedSettings = false;
}

int Project::GetVerbosity() const {
  return m_Verbosity;
}

void Project::SetVerbosity(int verbosity) {
  m_Verbosity = verbosity;
  m_UnsavedSettings = true;
}

const char *ProjectNotFound::what() const noexcept {
  return "Project not found.";
}
