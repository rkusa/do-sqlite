#!/usr/bin/env sh
set -e
cd wasm
# `--branch wasi-sdk-14` doesn't build, so use `main` for now
git clone --branch wasi-sdk-14 https://github.com/WebAssembly/wasi-sdk.git
cd wasi-sdk
git submodule update --init --recursive
NINJA_FLAGS=-v make package
tar -xzf dist/wasi-sdk-*.tar.gz -C dist
