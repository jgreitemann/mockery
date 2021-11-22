#pragma once

#include <project/ProjectStorage.h>

#include <memory>

struct ProjectNotFound : std::exception {
    [[nodiscard]] const char *what() const noexcept override;
};

class Project {
public:
    static constexpr int DefaultVerbosity = 1;
    static constexpr std::string_view SettingsFilename = "settings.toml";

    Project(std::string_view name, std::unique_ptr<ProjectStorage> storage);
    ~Project();

    [[nodiscard]] int GetVerbosity() const;
    void SetVerbosity(int verbosity);

private:
    bool ReadSettings();
    void WriteSettings();

private:
    std::unique_ptr<ProjectStorage> m_Storage;
    int m_Verbosity = DefaultVerbosity;
    bool m_UnsavedSettings = true;
};
