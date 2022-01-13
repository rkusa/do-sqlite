use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::Future;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use rusqlite::OpenFlags;
use sqlite_vfs::Vfs;
use worker::{console_log, State, Storage};

pub struct DurableObjectVfs<const BLOCK_SIZE: usize> {
    state: State,
}

impl<const BLOCK_SIZE: usize> DurableObjectVfs<BLOCK_SIZE> {
    pub fn new(state: State) -> Self {
        Self { state }
    }
}

impl<const BLOCK_SIZE: usize> Vfs for DurableObjectVfs<BLOCK_SIZE> {
    type File = Blocks<BLOCK_SIZE>;

    fn open(
        &self,
        path: &std::path::Path,
        _flags: OpenFlags,
    ) -> Result<Self::File, std::io::Error> {
        console_log!("open");

        let name = path.file_name().unwrap().to_string_lossy().to_string();
        console_log!("0");
        self.state
            .storage()
            .get::<Option<Vec<u8>>>(&format!("{}.0.block", name));
        console_log!("1");
        let count = block_on(
            self.state
                .storage()
                .get::<Option<Vec<u8>>>(&format!("{}.0.block", name)),
        )
        .map_err(worker_to_io_error)
        .and_then(|page| {
            console_log!("2");
            if let Some(page) = page {
                let mut bytes = [0u8; 4];
                (&page[28..]).read_exact(&mut bytes[..])?;

                Ok(u32::from_be_bytes(bytes) as usize)
            } else {
                Ok(0)
            }
        })
        .unwrap_or(0);
        console_log!("COUNT: {}", count);
        Ok(Blocks {
            name,
            count,
            offset: 0,
            blocks: Default::default(),
            storage: self.state.storage(),
        })
    }

    fn delete(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        console_log!("Ignore delete {}", path.to_string_lossy());
        // std::fs::remove_file(path)
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool, std::io::Error> {
        console_log!("exists");
        todo!("exists");

        let path = if let Some(ext) = path.extension() {
            path.with_extension(format!("{}.0.block", ext.to_string_lossy()))
        } else {
            path.with_extension("0.block")
        };
        Ok(dbg!(path.is_file()))
    }
}

struct Block {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Blocks<const BLOCK_SIZE: usize> {
    name: String,
    count: usize,
    offset: usize,
    blocks: HashMap<usize, Block>,
    storage: Storage,
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

        console_log!("seek to {}", offset);

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
                block_on(
                    self.storage
                        .put(&format!("{}.{}.block", self.name, index), &block.data),
                )
                .map_err(worker_to_io_error)?;
                block.dirty = false;
            }
        }
        Ok(())
    }
}

impl<const BLOCK_SIZE: usize> Blocks<BLOCK_SIZE> {
    fn current(&mut self) -> Result<&mut Block, std::io::Error> {
        let index = self.offset / BLOCK_SIZE;

        if let Entry::Vacant(entry) = self.blocks.entry(index) {
            let data: Option<Vec<u8>> =
                block_on(self.storage.get(&format!("{}.{}.block", self.name, index)))
                    .map_err(worker_to_io_error)?;
            entry.insert(Block {
                data: data.unwrap_or_else(|| vec![0; BLOCK_SIZE]),
                dirty: false,
            });
        }

        console_log!("Block: {}", index);

        Ok(self.blocks.get_mut(&index).unwrap())
    }
}

fn worker_to_io_error(err: worker::Error) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, err.to_string())
}

pub fn block_on<F, R>(fut: F) -> R
where
    F: Future<Output = R>,
{
    use std::task::{Context, Poll};

    let mut fut = Box::pin(fut);
    let waker = futures_task::noop_waker();
    let mut context = Context::from_waker(&waker);
    console_log!("start polling");
    loop {
        console_log!("loop");
        if let Poll::Ready(val) = fut.as_mut().poll(&mut context) {
            console_log!("ready");
            return val;
        }
    }
}
