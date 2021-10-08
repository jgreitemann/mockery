#include "Fooable.h"

struct Foo : Fooable {
    void foo() override {}
    int const& bar(double x) const override { return 42; }
    auto bla(std::pair<int, bool> const& a, const std::optional<float>* b) const noexcept -> std::map<int, float> override { return {}; }
};

int main() {
  Foo f{};
}
