#pragma once

struct Fooable {
    virtual void foo() = 0;
    virtual int bar(double x) const = 0;
    float baz() { return 3.141; }
    
    int i;
};