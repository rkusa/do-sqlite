use do_sqlite::PagesVfs;
use rusqlite::{Connection, OpenFlags};
use sqlite_vfs::register;

fn main() {
    // let ptr = unsafe { get_page(0) };
    // unsafe { debug(*ptr as i32) };

    eprintln!("1");
    register("cfdo", PagesVfs::<4096>).unwrap();
    eprintln!("2");

    // let conn = Connection::open_with_flags_and_vfs(
    //     "main.db3",
    //     OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE, // | OpenFlags::SQLITE_OPEN_NO_MUTEX
    //     "cfdo",
    // )
    // .unwrap();
    eprintln!("3");

    let conn = Connection::open_in_memory_with_flags(
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap();

    conn.execute("PRAGMA page_size = 4096;", []).unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode=MEMORY", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode, "memory");

    let n: i64 = conn.query_row("SELECT 42", [], |row| row.get(0)).unwrap();
    assert_eq!(n, 42);
}
