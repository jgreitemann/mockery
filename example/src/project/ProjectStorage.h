#pragma once

#include <map>
#include <string>
#include <string_view>

enum class Type {
    Unknown,
    File,
    Directory,
    SymLink,
};

struct ProjectStorage {
    virtual ~ProjectStorage() = default;

    virtual bool Open(std::string_view path) = 0;

    virtual bool Close() = 0;

    [[nodiscard]] virtual bool IsOpen() const noexcept = 0;

    [[nodiscard]] virtual auto List() const -> std::map<std::string, Type> = 0;

    [[nodiscard]] virtual std::string ReadFile(std::string_view name) const = 0;

    virtual void SaveFile(std::string_view name, std::string const &contents) = 0;
};
