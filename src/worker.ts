import * as Asyncify from "asyncify-wasm";

import module from "./do_sqlite.wasm";

interface Env {
  DATABASE: DurableObjectNamespace;
}

const ERRNO_SUCCESS = 0;
const ERRNO_BADF = 8;

export class Database {
  private readonly state: DurableObjectState;

  constructor(state: DurableObjectState, _env: Env) {
    this.state = state;
  }

  pages: Array<Uint8Array> = [];

  async fetch(request: Request) {
    const query = await request.text();
    const storage = this.state.storage;

    let exports: Exports;
    const instance = await Asyncify.instantiate(module, {
      wasi_snapshot_preview1: {
        // "wasi_snapshot_preview1"."random_get": [I32, I32] -> [I32]
        random_get(offset: number, length: number) {
          const buffer = new Uint8Array(exports.memory.buffer, offset, length);
          crypto.getRandomValues(buffer);

          return ERRNO_SUCCESS;
        },

        // "wasi_snapshot_preview1"."clock_time_get": [I32, I64, I32] -> [I32]
        clock_time_get() {
          throw new Error("clock_time_get not implemented");
        },

        // "wasi_snapshot_preview1"."fd_write": [I32, I32, I32, I32] -> [I32]
        fd_write(
          fd: number,
          iovsOffset: number,
          iovsLength: number,
          nwrittenOffset: number
        ) {
          if (fd !== 1 && fd !== 2) {
            return ERRNO_BADF;
          }

          const decoder = new TextDecoder();
          const memoryView = new DataView(exports.memory.buffer);
          let nwritten = 0;
          for (let i = 0; i < iovsLength; i++) {
            const dataOffset = memoryView.getUint32(iovsOffset, true);
            iovsOffset += 4;

            const dataLength = memoryView.getUint32(iovsOffset, true);
            iovsOffset += 4;

            const data = new Uint8Array(
              exports.memory.buffer,
              dataOffset,
              dataLength
            );
            const s = decoder.decode(data);
            nwritten += data.byteLength;
            switch (fd) {
              case 1: // stdout
                console.log(s);
                break;
              case 2: // stderr
                console.error(s);
                break;
              default:
                return ERRNO_BADF;
            }
          }

          memoryView.setUint32(nwrittenOffset, nwritten, true);

          return ERRNO_SUCCESS;
        },

        // "wasi_snapshot_preview1"."poll_oneoff": [I32, I32, I32, I32] -> [I32]
        poll_oneoff() {
          throw new Error("poll_oneoff not implemented");
        },

        // "wasi_snapshot_preview1"."environ_get": [I32, I32] -> [I32]
        environ_get() {
          throw new Error("environ_get not implemented");
        },

        // "wasi_snapshot_preview1"."environ_sizes_get": [I32, I32] -> [I32]
        environ_sizes_get(
          environcOffset: number,
          _environBufferSizeOffset: number
        ) {
          const memoryView = new DataView(exports.memory.buffer);
          memoryView.setUint32(environcOffset, 0, true);
          return ERRNO_SUCCESS;
        },

        // "wasi_snapshot_preview1"."proc_exit": [I32] -> []
        proc_exit(rval: number) {
          throw new Error(`WASM program exited with code: ${rval}`);
        },
      },

      env: {
        async get_page(ix: number): Promise<number> {
          const page: Array<number> =
            (await storage.get<Array<number>>(String(ix))) ?? new Array(4096);

          const offset = await exports.alloc(page.length);
          const dst = new Uint8Array(
            exports.memory.buffer,
            offset,
            page.length
          );
          dst.set(Array.from(new Uint8Array(page)));

          // TODO: dealloc

          return offset;
        },

        async put_page(ix: number, ptr: number) {
          const page = new Uint8Array(exports.memory.buffer, ptr, 16384);
          await storage.put(String(ix), Array.from(page), {});
        },
      },
    });
    exports = instance.exports as unknown as Exports;

    // increase asyncify stack size
    const STACK_SIZE = 4096;
    const DATA_ADDR = 16;
    const ptr = await exports.alloc(STACK_SIZE);
    new Int32Array(exports.memory.buffer, DATA_ADDR, 2).set([
      ptr,
      ptr + STACK_SIZE,
    ]);

    const encoder = new TextEncoder();
    const offset: number = await exports.alloc(query.length);
    encoder.encodeInto(
      query,
      new Uint8Array(exports.memory.buffer, offset, query.length)
    );
    const result = await exports.run(offset, query.length);
    await exports.dealloc(offset, query.length);

    return new Response(`Ok: ${result}`);
  }
}

interface Exports {
  readonly memory: WebAssembly.Memory;
  alloc(size: number): number;
  dealloc(size: number, len: number): void;
  run(ptr: number, len: number): number;
}

export default {
  async fetch(req: Request, env: Env) {
    if (req.method !== "GET") {
      return new Response(null, { status: 405 }); // method not allowed
    }

    const id = env.DATABASE.idFromName("sqlite");
    const stub = env.DATABASE.get(id);
    return stub.fetch("http://sqlite/");
  },
};
