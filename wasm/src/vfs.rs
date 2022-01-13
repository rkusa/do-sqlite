use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::slice;

use rusqlite::OpenFlags;
use sqlite_vfs::Vfs;

pub struct DurableObjectVfs<const BLOCK_SIZE: usize>;

struct Block<const BLOCK_SIZE: usize> {
    data: [u8; BLOCK_SIZE],
    dirty: bool,
}

pub struct Blocks<const BLOCK_SIZE: usize> {
    name: String,
    count: usize,
    offset: usize,
    blocks: HashMap<u32, Block<BLOCK_SIZE>>,
}

impl<const BLOCK_SIZE: usize> Vfs for DurableObjectVfs<BLOCK_SIZE> {
    type File = Blocks<BLOCK_SIZE>;

    fn open(
        &self,
        path: &std::path::Path,
        _flags: OpenFlags,
    ) -> Result<Self::File, std::io::Error> {
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        let mut blocks = Blocks {
            name,
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

    fn delete(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        // std::fs::remove_file(path)
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool, std::io::Error> {
        // todo!("exists");
        Ok(false)

        // let path = if let Some(ext) = path.extension() {
        //     path.with_extension(format!("{}.0.block", ext.to_string_lossy()))
        // } else {
        //     path.with_extension("0.block")
        // };
        // Ok(dbg!(path.is_file()))
    }
}

impl<const BLOCK_SIZE: usize> sqlite_vfs::File for Blocks<BLOCK_SIZE> {
    fn file_size(&self) -> Result<u64, std::io::Error> {
        Ok(dbg!((self.count * BLOCK_SIZE) as u64))
    }
}

impl<const BLOCK_SIZE: usize> Seek for Blocks<BLOCK_SIZE> {
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

impl<const BLOCK_SIZE: usize> Read for Blocks<BLOCK_SIZE> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let offset = self.offset % BLOCK_SIZE;
        let block = self.current()?;
        let n = (&block.data[offset..]).read(buf)?;
        self.offset += n;
        Ok(n)
    }
}

impl<const BLOCK_SIZE: usize> Write for Blocks<BLOCK_SIZE> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let offset = self.offset % BLOCK_SIZE;
        let block = self.current()?;
        let n = (&mut block.data[offset..]).write(buf)?;
        block.dirty = true;
        self.offset += n;

        let count = (self.offset / BLOCK_SIZE) + 1;
        if count > self.count {
            self.count = count;
        }

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for (index, block) in &mut self.blocks {
            if block.dirty {
                Self::put_page(*index, block.data.as_ptr());
                block.dirty = false;
            }
        }
        Ok(())
    }
}

impl<const BLOCK_SIZE: usize> Blocks<BLOCK_SIZE> {
    fn current(&mut self) -> Result<&mut Block<BLOCK_SIZE>, std::io::Error> {
        let index = self.offset / BLOCK_SIZE;

        if let Entry::Vacant(entry) = self.blocks.entry(index as u32) {
            let data = Self::get_page(index as u32);
            entry.insert(Block {
                data: data.unwrap_or_else(|| [0; BLOCK_SIZE]),
                dirty: false,
            });
        }

        Ok(self.blocks.get_mut(&(index as u32)).unwrap())
    }

    pub fn get_page(ix: u32) -> Option<[u8; BLOCK_SIZE]> {
        unsafe {
            let ptr = crate::get_page(ix);
            if ptr.is_null() {
                None
            } else {
                let slice = slice::from_raw_parts_mut(ptr, BLOCK_SIZE);
                slice[..].try_into().ok()
            }
        }
    }

    fn put_page(ix: u32, data: *const u8) {
        eprintln!("BEFORE");
        unsafe {
            crate::put_page(ix, data);
        }
        eprintln!("AFTER");
    }
}
