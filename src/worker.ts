import { Sqlite } from "@rkusa/wasm-sqlite";

interface Env {
  DATABASE: DurableObjectNamespace;
}

export class Database {
  private pageCount: number;
  private sqlite: Sqlite;

  constructor(state: DurableObjectState, _env: Env) {
    this.pageCount = 0;

    const storage = state.storage;
    const self = this;

    // Block concurrency until DO is completely initialized
    state.blockConcurrencyWhile(async () => {
      // Read the page count from page 0 (if page 0 already exists)
      const page: Array<number> | undefined = await storage.get<Array<number>>(
        String(0)
      );
      if (page) {
        const view = new DataView(new Uint8Array(page).buffer);
        this.pageCount = view.getUint32(28, false);
      }

      this.sqlite = await Sqlite.instantiate({
        pageCount(): number {
          return self.pageCount;
        },

        async getPage(ix: number): Promise<Uint8Array> {
          const page: Array<number> =
            (await storage.get<Array<number>>(String(ix))) ?? new Array(4096);
          return new Uint8Array(page);
        },

        async putPage(ix: number, page: Uint8Array): Promise<void> {
          await storage.put(String(ix), Array.from(page), {});
          self.pageCount = Math.max(self.pageCount, ix + 1);
        },

        async delPage(ix: number): Promise<void> {
          await storage.delete(String(ix));
          if (ix + 1 >= self.pageCount) {
            self.pageCount = ix;
          }
        },
      });
    });
  }

  async fetch(req: Request) {
    if (req.method !== "GET") {
      return new Response("Only GET requests are allowed", {
        status: 405 /* method not allowed */,
      });
    }

    const url = new URL(req.url);
    if (url.pathname !== "/websocket") {
      return new Response("Not found, try GET /websocket", {
        status: 405 /* method not allowed */,
      });
    }

    if (req.headers.get("Upgrade") != "websocket") {
      return new Response("Expected websocket connection", {
        status: 426 /* upgrade required */,
      });
    }

    const conn = await this.sqlite.connect();
    const { 0: tx, 1: rx } = new WebSocketPair();
    rx.accept();

    console.log("initial page count:", this.pageCount);

    rx.addEventListener("message", (msg) => {
      let data: unknown;
      try {
        data = JSON.parse(msg.data);
      } catch (err) {
        rx.send(JSON.stringify({ error: `Expected JSON: ${err}` }));
        return;
      }
      if (!data || typeof data !== "object") {
        rx.send(JSON.stringify({ error: "Expected JSON object" }));
        return;
      }

      // validate data
      if (!isQuery(data)) {
        rx.send(JSON.stringify({ error: "Expected `sql` property" }));
        return;
      }
      if (!isParams(data.params)) {
        rx.send(
          JSON.stringify({
            error:
              "Expected `body.params` to be an array of `string | number | boolean | null`",
          })
        );
        return;
      }

      conn
        .queryRaw(data.sql, data.params)
        .then((json) => rx.send(json))
        .catch((err: unknown) =>
          rx.send(
            JSON.stringify({ error: String(err), stack: (err as Error).stack })
          )
        );
    });
    rx.addEventListener("close", () => {
      conn.drop();
    });
    rx.addEventListener("error", () => {
      conn.drop();
    });

    // Now we return the other end of the pair to the client.
    return new Response(null, { status: 101, webSocket: tx });
  }
}

interface Query {
  sql: string;
  params?: unknown;
}

function isQuery(data: unknown): data is Query {
  return Boolean(
    data &&
      typeof data === "object" &&
      "sql" in data &&
      typeof (data as { sql: unknown }).sql === "string"
  );
}

function isParams(
  params: unknown
): params is undefined | Array<null | string | number | boolean> {
  return (
    !params ||
    (Array.isArray(params) &&
      !params.find(
        (p) =>
          !(
            p === null ||
            typeof p === "string" ||
            typeof p === "number" ||
            typeof p === "boolean"
          )
      ))
  );
}

export default {
  async fetch(req: Request, env: Env) {
    if (req.method !== "GET") {
      return new Response(null, { status: 405 /* method not allowed */ });
    }

    // Expected pattern: /:database
    const url = new URL(req.url);
    const segments = url.pathname.slice(1).split("/");
    if (segments.length !== 1) {
      return new Response("not found", {
        status: 404 /* not found */,
      });
    }
    const [name] = segments;

    const id = env.DATABASE.idFromName(name);
    const stub = env.DATABASE.get(id);
    return stub.fetch(`http://sqlite/websocket`, req);
  },
};
