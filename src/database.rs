use rusqlite::{Connection, OpenFlags};
use sqlite_vfs::register;
use worker::{
    async_trait, js_sys, wasm_bindgen, wasm_bindgen_futures, worker_sys, Env, Request, Response,
    Result,
};

use crate::vfs::DurableObjectVfs;

#[worker::durable_object]
pub struct Database {
    conn: Connection,
}

#[worker::durable_object]
impl DurableObject for Database {
    fn new(state: State, _env: Env) -> Self {
        register("cfdo", DurableObjectVfs::<4096>::new(state)).unwrap();
        let conn = Connection::open_with_flags_and_vfs(
            "main.db3",
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            "cfdo",
        )
        .unwrap();
        // let conn = Connection::open_in_memory_with_flags(
        //     OpenFlags::SQLITE_OPEN_READ_WRITE
        //         | OpenFlags::SQLITE_OPEN_CREATE
        //         | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        // )
        // .unwrap();

        Self { conn }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // the following will block?

        self.conn.execute("PRAGMA page_size = 4096;", []).unwrap();
        let journal_mode: String = self
            .conn
            .query_row("PRAGMA journal_mode=MEMORY", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "memory");

        let n: i64 = self
            .conn
            .query_row("SELECT 42", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 42);

        Response::ok("OK")
    }
}
