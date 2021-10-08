#pragma once

#include <utility>
#include <optional>
#include <map>

struct Fooable {
    virtual void foo() = 0;
    virtual int const& bar(double x) const = 0;
    virtual auto bla(std::pair<int, bool> const& a, const std::optional<float>*) const noexcept -> std::map<int, float> = 0;
    virtual void lvalue() & = 0;
    virtual void const_lvalue() const& = 0;
    virtual void rvalue() && = 0;
    float baz() { return 3.141; }

    int i;
};