const esbuild = require("esbuild");
const fsp = require("fs").promises;
const path = require("path");

const wasmPlugin = {
  name: "wasm",
  setup(build) {
    build.onResolve({ filter: /\.wasm$/ }, async (args) => {
      if (args.namespace === "wasm-file") {
        return { external: true };
      }

      if (args.resolveDir === "") {
        return;
      }

      return {
        // path: "test",
        // external: true,
        external: true,
        path: path.isAbsolute(args.path)
          ? args.path
          : path.join(args.resolveDir, args.path),
        namespace: "wasm-file",
      };
    });

    build.onLoad({ filter: /.*/, namespace: "wasm-file" }, async (args) => ({
      contents: await fsp.readFile(args.path),
      loader: "file",
    }));
  },
};

esbuild
  .build({
    entryPoints: ["./src/worker.ts"],
    format: "esm",
    platform: "neutral",
    bundle: true,
    outdir: "./dist",
    plugins: [wasmPlugin],
  })
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });

// esbuild --format=esm --platform=neutral --loader:.wasm=file --loader:.js=file
// --outdir=./dist --bundle src/worker.ts
