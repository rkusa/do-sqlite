#!/bin/sh
set -e

rm -rf ./dist/*
mkdir -p dist

./node_modules/.bin/esbuild --format=esm --platform=neutral --external:"*.wasm" --outdir=./dist --bundle --main-fields=module src/worker.ts
cp ./node_modules/@rkusa/wasm-sqlite/dist/wasm_sqlite.wasm dist/