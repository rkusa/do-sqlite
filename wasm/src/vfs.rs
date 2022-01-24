use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::slice;

use rusqlite::OpenFlags;
use sqlite_vfs::Vfs;

pub struct PagesVfs<const PAGE_SIZE: usize>;

struct Page<const PAGE_SIZE: usize> {
    data: [u8; PAGE_SIZE],
    dirty: bool,
}

pub struct Pages<const PAGE_SIZE: usize> {
    count: usize,
    offset: usize,
    blocks: HashMap<u32, Page<PAGE_SIZE>>,
}

impl<const PAGE_SIZE: usize> Vfs for PagesVfs<PAGE_SIZE> {
    type File = Pages<PAGE_SIZE>;

    fn open(
        &self,
        _path: &std::path::Path,
        _flags: OpenFlags,
    ) -> Result<Self::File, std::io::Error> {
        // TODO: open file based on path

        let mut blocks = Pages {
            count: 0,
            offset: 0,
            blocks: Default::default(),
        };

        if let Some(page) = Self::File::get_page(0) {
            // TODO: unwrap?
            blocks.count = u32::from_be_bytes(page[28..32].try_into().unwrap()) as usize;
        }

        Ok(blocks)
    }

    fn delete(&self, _path: &std::path::Path) -> Result<(), std::io::Error> {
        // Only used to delete journal or wal files, which both are not implemented yet, thus simply
        // ignored for now.
        Ok(())
    }

    fn exists(&self, _path: &Path) -> Result<bool, std::io::Error> {
        // Only used to check existance of journal or wal files, which both are not implemented yet,
        // thus simply always return `false` for now.
        Ok(false)
    }
}

impl<const PAGE_SIZE: usize> sqlite_vfs::File for Pages<PAGE_SIZE> {
    fn file_size(&self) -> Result<u64, std::io::Error> {
        Ok(dbg!((self.count * PAGE_SIZE) as u64))
    }
}

impl<const PAGE_SIZE: usize> Seek for Pages<PAGE_SIZE> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let offset = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::End(_) => unimplemented!(),
            SeekFrom::Current(_) => unimplemented!(),
        };

        self.offset = offset as usize;

        Ok(self.offset as u64)
    }
}

impl<const PAGE_SIZE: usize> Read for Pages<PAGE_SIZE> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let offset = self.offset % PAGE_SIZE;
        let block = self.current()?;
        let n = (&block.data[offset..]).read(buf)?;
        self.offset += n;
        Ok(n)
    }
}

impl<const PAGE_SIZE: usize> Write for Pages<PAGE_SIZE> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let offset = self.offset % PAGE_SIZE;
        let block = self.current()?;
        let n = (&mut block.data[offset..]).write(buf)?;
        block.dirty = true;
        self.offset += n;

        let count = (self.offset / PAGE_SIZE) + 1;
        if count > self.count {
            self.count = count;
        }

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for (index, block) in &mut self.blocks {
            if block.dirty {
                Self::put_page(*index, &block.data);
                block.dirty = false;
            }
        }
        Ok(())
    }
}

impl<const PAGE_SIZE: usize> Pages<PAGE_SIZE> {
    fn current(&mut self) -> Result<&mut Page<PAGE_SIZE>, std::io::Error> {
        let index = self.offset / PAGE_SIZE;

        if let Entry::Vacant(entry) = self.blocks.entry(index as u32) {
            let data = Self::get_page(index as u32);
            entry.insert(Page {
                data: data.unwrap_or_else(|| [0; PAGE_SIZE]),
                dirty: false,
            });
        }

        Ok(self.blocks.get_mut(&(index as u32)).unwrap())
    }

    pub fn get_page(ix: u32) -> Option<[u8; PAGE_SIZE]> {
        unsafe {
            let ptr = crate::get_page(ix);
            if ptr.is_null() {
                None
            } else {
                let slice = slice::from_raw_parts_mut(ptr, PAGE_SIZE);
                slice[..].try_into().ok()
            }
        }
    }

    fn put_page(ix: u32, data: &[u8; PAGE_SIZE]) {
        eprintln!("BEFORE");
        unsafe {
            crate::put_page(ix, data.as_ptr());
        }
        eprintln!("AFTER");
    }
}
