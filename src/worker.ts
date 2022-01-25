import Sqlite from "@rkusa/wasm-sqlite";

interface Env {
  DATABASE: DurableObjectNamespace;
}

export class Database {
  private readonly state: DurableObjectState;

  constructor(state: DurableObjectState, _env: Env) {
    this.state = state;
  }

  pages: Array<Uint8Array> = [];

  async fetch(_req: Request) {
    // const query: { sql: string; params: Array<Param> } = await req.json();
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

    // const json = await sqlite.queryRaw(query.sql, query.params);
    // return new Response(json, {
    //   headers: {
    //     "content-type": "application/json; charset=utf-8",
    //   },
    // });

    await sqlite.execute(
      "CREATE TABLE IF NOT EXISTS vals (id INT PRIMARY KEY, val VARCHAR NOT NULL)",
      []
    );
    await sqlite.execute("INSERT INTO vals (val) VALUES (?1)", ["val"]);
    const result: Array<{ count: number }> = await sqlite.query(
      "SELECT COUNT(*) AS count FROM vals",
      []
    );
    const count = result[0].count;

    return new Response(`Row Count: ${count}`);
  }
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
