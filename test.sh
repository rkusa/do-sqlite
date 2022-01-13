#!/bin/sh
set -e

cd wasm
make build
cd ..

wasm-opt -O0 --asyncify --pass-arg asyncify-imports@env.put_page,env.get_page \
  wasm/target/wasm32-wasi/debug/do_sqlite.wasm \
  -o dist/do_sqlite.wasm
# cp wasm/target/wasm32-wasi/debug/do_sqlite.wasm dist/do_sqlite.wasm

# node --experimental-wasi-unstable-preview1 test.mjs
node test.mjs
