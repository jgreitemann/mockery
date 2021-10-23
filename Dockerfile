# syntax=docker/dockerfile:1
FROM alpine:edge

RUN apk add --no-cache \
	bash \
	cmake \
	make \
	clang-dev \
	build-base \
	g++ \
	python3 \
	py3-pip \
	cargo

ENV CC="gcc"
ENV CXX="g++"
	
RUN pip install conan
RUN conan profile new default --detect
RUN conan profile update settings.compiler.cppstd=17 default
RUN conan profile update settings.compiler.libcxx=libstdc++11 default

WORKDIR /mockery
COPY . .

RUN cargo build --release
ENV PATH="/mockery/target/release:${PATH}"

WORKDIR /mockery/example/build
RUN conan install .. --build=missing

WORKDIR /mockery
ENTRYPOINT ["./integration_test.sh"]