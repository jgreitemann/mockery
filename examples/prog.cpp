#include "Fooable.h"

struct Foo : Fooable {
    void foo() override {}
    int bar(double x) const override { return 42; }
};

int main() {
  Foo f{};
}
