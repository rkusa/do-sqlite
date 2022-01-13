import * as Asyncify from "asyncify-wasm/dist/asyncify.mjs";

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
    const pages = this.pages;
    const storage = this.state.storage;

    const instance = await Asyncify.instantiate(module, {
      wasi_snapshot_preview1: {
        // "wasi_snapshot_preview1"."random_get": [I32, I32] -> [I32]
        random_get(offset: number, length: number) {
          const buffer = new Uint8Array(
            instance.exports.memory.buffer,
            offset,
            length
          );
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
          const memoryView = new DataView(instance.exports.memory.buffer);
          let nwritten = 0;
          for (let i = 0; i < iovsLength; i++) {
            const dataOffset = memoryView.getUint32(iovsOffset, true);
            iovsOffset += 4;

            const dataLength = memoryView.getUint32(iovsOffset, true);
            iovsOffset += 4;

            const data = new Uint8Array(
              instance.exports.memory.buffer,
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
          const memoryView = new DataView(instance.exports.memory.buffer);
          memoryView.setUint32(environcOffset, 0, true);
          return ERRNO_SUCCESS;
        },

        // "wasi_snapshot_preview1"."proc_exit": [I32] -> []
        proc_exit(rval: number) {
          throw new Error(`WASM program exited with code: ${rval}`);
        },
      },

      env: {
        __extenddftf2() {
          console.log("__extenddftf2");
          throw new Error("__extenddftf2 not implemented");
        },
        __multf3() {
          console.log("__multf3");
          throw new Error("__multf3 not implemented");
        },
        __addtf3() {
          console.log("__addtf3");
          throw new Error("__addtf3 not implemented");
        },
        __trunctfdf2() {
          console.log("__trunctfdf2");
          throw new Error("__trunctfdf2 not implemented");
        },
        __gttf2() {
          console.log("__gttf2");
          throw new Error("__gttf2 not implemented");
        },
        __getf2() {
          console.log("__getf2");
          throw new Error("__getf2 not implemented");
        },
        __divtf3() {
          console.log("__divtf3");
          throw new Error("__divtf3 not implemented");
        },
        __lttf2() {
          console.log("__lttf2");
          throw new Error("__lttf2 not implemented");
        },
        __fixtfsi() {
          console.log("__fixtfsi");
          throw new Error("__fixtfsi not implemented");
        },
        __floatsitf() {
          console.log("__floatsitf");
          throw new Error("__floatsitf not implemented");
        },
        __subtf3() {
          console.log("__subtf3");
          throw new Error("__subtf3 not implemented");
        },
        __floatditf() {
          console.log("__floatditf");
          throw new Error("__floatditf not implemented");
        },

        async get_page(ix: number): Promise<number> {
          const page: Array<number> =
            (await storage.get<Array<number>>(String(ix))) ?? new Array(4096);

          const offset: number = await (instance.exports.alloc as Function)(
            4096
          );
          const dst = new Uint8Array(
            (instance.exports.memory as WebAssembly.Memory).buffer,
            offset,
            4096
          );
          dst.set(Array.from(new Uint8Array(page)));

          // TODO: dealloc

          return offset;
        },

        async put_page(ix: number, ptr: number) {
          const page = new Uint8Array(
            (instance.exports.memory as WebAssembly.Memory).buffer,
            ptr,
            4096
          );
          await storage.put(String(ix), Array.from(page), {});
        },
      },
    });

    const encoder = new TextEncoder();
    const offset: number = await (instance.exports.alloc as Function)(
      query.length
    );
    encoder.encodeInto(
      query,
      new Uint8Array(instance.exports.memory.buffer, offset, query.length)
    );
    const result = await (instance.exports.run as Function)(
      offset,
      query.length
    );
    await (instance.exports.dealloc as Function)(offset, query.length);

    return new Response(`Ok: ${result}`);
  }
}

export default {
  async fetch(request: Request, env: Env) {
    // if !matches!(req.method(), Method::Get) {
    //     return Response::error("Method Not Allowed", 405);
    // }

    // let namespace = env.durable_object("DATABASE")?;
    // let stub = namespace.id_from_name("main")?.get_stub()?;
    // stub.fetch_with_str("http://sqlite/").await

    const id = env.DATABASE.idFromName("sqlite");
    const stub = env.DATABASE.get(id);
    return stub.fetch("http://sqlite/");
  },
};
