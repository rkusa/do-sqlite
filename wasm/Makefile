.PHONY: build
build:
	PATH="$(shell pwd)/wasi-sdk/dist/wasi-sdk-14.0/bin:${PATH}" \
	CFLAGS="--sysroot=$(shell pwd)/wasi-sdk/dist/wasi-sdk-14.0/share/wasi-sysroot" \
		cargo build --target wasm32-wasi

build_release:
	PATH="$(shell pwd)/wasi-sdk/dist/wasi-sdk-14.0/bin:${PATH}" \
	CFLAGS="--sysroot=$(shell pwd)/wasi-sdk/dist/wasi-sdk-14.0/share/wasi-sysroot" \
		cargo build --target wasm32-wasi --release
