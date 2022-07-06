// ciso: Compress and decompress PSP ISOs
use anyhow::Result;
use std::fs::File;
use std::io::{
    prelude::*,
    SeekFrom,
};

use crate::consts::CISO_HEADER_SIZE;

pub struct BlockIndex(Vec<u32>);

impl BlockIndex {
    pub fn new(num_blocks: usize) -> Self {
        Self(vec![0u32; num_blocks])
    }

    pub fn get(&self, offset: usize) -> u32 {
        self.0[offset]
    }

    pub fn iter_mut(&mut self) -> ::std::slice::IterMut<u32> {
        self.0.iter_mut()
    }

    pub fn read_from(&mut self, file: &mut File) -> Result<&mut Self> {
        let mut buffer: [u8; 4] = [0; 4];
        let total_blocks = self.0.len();

        for block in self.0.iter_mut().take(total_blocks) {
            file.read_exact(&mut buffer)?;

            *block = u32::from_le_bytes(buffer);
        }

        Ok(self)
    }

    pub fn set(&mut self, offset: usize, content: u32) {
        self.0[offset] = content;
    }

    pub fn write_to(&mut self, file: &mut File) -> Result<()> {
        // Seek to after the header, which is where the block index lives.
        file.seek(SeekFrom::Start(CISO_HEADER_SIZE as u64))?;

        for block in &self.0 {
            let bytes = block.to_le_bytes();

            file.write_all(&bytes)?;
        }

        Ok(())
    }
}
