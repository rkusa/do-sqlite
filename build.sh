#!/bin/sh
set -e

rm -rf ./dist/*
mkdir -p dist


cd wasm
make build_release
cd ..

./node_modules/.bin/esbuild --format=esm --platform=neutral --external:"*.wasm" --outdir=./dist --bundle src/worker.ts

wasm-opt -O --asyncify --pass-arg asyncify-imports@env.put_page,env.get_page \
  wasm/target/wasm32-wasi/release/do_sqlite.wasm \
  -o dist/do_sqlite.wasm
