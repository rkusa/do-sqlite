use rusqlite::{Connection, OpenFlags};
use sqlite_vfs::register;

pub use crate::vfs::DurableObjectVfs;

mod utils;
mod vfs;

extern "C" {
    pub fn get_page(ix: u32) -> *mut u8;
    pub fn put_page(ix: u32, ptr: *const u8);
}

#[no_mangle]
extern "C" fn sqlite3_os_init() -> i32 {
    if register("cfdo", DurableObjectVfs::<4096>).is_ok() {
        0
    } else {
        1
    }
}

#[no_mangle]
extern "C" fn run(ptr: *const u8, len: usize) -> i32 {
    utils::set_panic_hook();

    let query = unsafe { std::slice::from_raw_parts::<'_, u8>(ptr, len) };
    let query = std::str::from_utf8(query).unwrap();
    println!("Query: {}", query);

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

    conn.execute("PRAGMA page_size = 4096;", []).unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode=MEMORY", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode, "memory");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS vals (id INT PRIMARY KEY, val VARCHAR NOT NULL)",
        [],
    )
    .unwrap();

    conn.execute("INSERT INTO vals (val) VALUES ('v')", [])
        .unwrap();

    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM vals", [], |row| row.get(0))
        .unwrap();

    n as i32
}

#[no_mangle]
unsafe fn alloc(size: usize) -> *mut u8 {
    use std::alloc::{alloc, Layout};

    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    alloc(layout)
}

#[no_mangle]
unsafe fn dealloc(ptr: *mut u8, size: usize) {
    use std::alloc::{dealloc, Layout};
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    dealloc(ptr, layout);
}
