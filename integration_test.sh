#!/bin/bash

psec () {
	echo -e "\v\033[0;1;34m~~~ $1 ~~~\033[0m\v"
}

psec "Build example app, generating the compilation database"
cmake -S example -B example/build -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=ON
cmake --build example/build --target example-app

psec "Generate Mock class for example project"
mockery create example/src/project/Project.cpp -i ProjectStorage | tee example/test/ProjectStorageMock.h

psec "Build tests using the generated mock"
cmake --build example/build --target example-tests

psec "Run tests for example project"
./example/build/test/example-tests --gtest_color=yes
