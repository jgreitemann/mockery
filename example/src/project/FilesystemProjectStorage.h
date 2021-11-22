#pragma once

#include <project/ProjectStorage.h>

#include <filesystem>
#include <optional>

class FilesystemProjectStorage : public ProjectStorage {
public:
    bool Open(std::string_view path) override;
    bool Close() override;
    [[nodiscard]] bool IsOpen() const noexcept override;

    [[nodiscard]] auto List() const -> std::map<std::string, Type> override;

    [[nodiscard]] std::string ReadFile(std::string_view name) const override;
    void SaveFile(std::string_view name, std::string const &contents) override;

private:
    std::optional<std::filesystem::path> m_ProjectDir{};
};
