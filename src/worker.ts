import Sqlite, { Param } from "@rkusa/wasm-sqlite";

interface Env {
  DATABASE: DurableObjectNamespace;
}

export class Database {
  private readonly state: DurableObjectState;

  constructor(state: DurableObjectState, _env: Env) {
    this.state = state;
  }

  async fetch(req: Request) {
    if (req.method !== "POST") {
      return new Response("only POST requests are allowed", {
        status: 405 /* method not allowed */,
      });
    }

    const url = new URL(req.url);
    let isQuery = false;
    if (url.pathname === "/query") {
      isQuery = true;
    } else if (url.pathname !== "/execute") {
      return new Response("not found", {
        status: 404 /* not found */,
      });
    }

    // parse body
    const query: { sql: string; params: Array<Param> } = await req.json();

    if (!query || typeof query !== "object") {
      return new Response("expected body to be an object", {
        status: 400 /* bad request */,
      });
    }

    // validate sql property
    if (typeof query?.sql !== "string") {
      return new Response("expected `body.sql` to be a string", {
        status: 400 /* bad request */,
      });
    }

    // validate params property
    if (
      query?.params &&
      (!Array.isArray(query?.params) ||
        query.params.find(
          (p) =>
            !(
              p === null ||
              typeof p === "string" ||
              typeof p === "number" ||
              typeof p === "boolean"
            )
        ))
    ) {
      return new Response(
        "expected `body.params` to be an array of `string | number | boolean | null`",
        {
          status: 400 /* bad request */,
        }
      );
    }

    // instantiate SQLite and plug it into the DO's storage
    const storage = this.state.storage;
    const sqlite = await Sqlite.instantiate(
      async (ix: number) => {
        const page: Array<number> =
          (await storage.get<Array<number>>(String(ix))) ?? new Array(4096);
        return new Uint8Array(page);
      },
      async (ix: number, page: Uint8Array) => {
        await storage.put(String(ix), Array.from(page), {});
      }
    );

    if (isQuery) {
      const json = await sqlite.queryRaw(query.sql, query.params);
      return new Response(json, {
        headers: {
          "content-type": "application/json; charset=utf-8",
        },
      });
    } else {
      await sqlite.execute(query.sql, query.params);
      return new Response(null, {
        status: 204 /* no content */,
      });
    }
  }
}

export default {
  async fetch(req: Request, env: Env) {
    if (req.method !== "POST") {
      return new Response(null, { status: 405 }); // method not allowed
    }

    // Expected pattern: /:database/{query,execute}
    const url = new URL(req.url);
    const segments = url.pathname.slice(1).split("/");
    if (segments.length !== 2) {
      return new Response("not found", {
        status: 404 /* not found */,
      });
    }
    const [name, path] = segments;

    const id = env.DATABASE.idFromName(name);
    const stub = env.DATABASE.get(id);
    return stub.fetch(`http://sqlite/${path}`, {
      method: "POST",
      body: req.body,
    });
  },
};
