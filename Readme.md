Mockery: generate Google Mock class from C++ interfaces
=======================================================

Building the project
--------------------

1. Make sure libclang is installed in the library search path:
- On Ubuntu, install it using `sudo apt install libclang-dev`
- On macOS, install Xcode development tools and set `DYLD_LIBRARY_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib` in your environment.

2. `cargo build`

Creating a mock class for the example C++ project
-------------------------------------------------

The `example` directory contains a simple CMake project which itself deal in
some sort of fictional `Project` class which might represent an IDE project for
instance. Projects are persisted to disk through an interface called
`ProjectStorage`. A implementation `FilesystemProjectStorage` exists and is
injected into the `Project` in production but is unsuitable for testing the
`Project` unit as filesystem operations quickly cause tests to become flakey.
Thanks to the dependency injection, we can test `Project` by mocking
`ProjectStorage`.

To create the mock implementation using Mockery, we first configure the CMake
project using
```sh
$ cmake -G Ninja -S example -B example/cmake-build-debug
```

The example is already set up to create a compile commands database. To do in
your own projects, pass `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` as an additional
argument.

Mockery will try to locate the compile commands database in relation to the
project sources automatically. You only need to specify the name of the
interface that should be mocked, in this case `ProjectStorage`. The definition
of the mock class will be written to standard out for now:
```sh
$ cargo run -- create example/src/project/Project.cpp -i ProjectStorage
```
```cpp
struct ProjectStorageMock : ProjectStorage {
    MOCK_METHOD(bool, Open, (std::string_view), (override));
    MOCK_METHOD(bool, Close, (), (override));
    MOCK_METHOD(bool, IsOpen, (), (const, noexcept, override));
    MOCK_METHOD((std::map<std::string, Type>), List, (), (const, override));
    MOCK_METHOD(std::string, ReadFile, (std::string_view), (const, override));
    MOCK_METHOD(void, SaveFile, (std::string_view, std::string const&), (override));
};
```
