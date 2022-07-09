// ciso: Compress and decompress PSP ISOs
use std::fs::File;
use std::io::{
    prelude::*,
    Error,
    SeekFrom,
};

pub trait ReadSizeAt {
    fn read_size(&mut self, size: u64) -> Result<Vec<u8>, Error>;
    fn read_size_at(
        &mut self,
        size: u64,
        offset: u64,
    ) -> Result<Vec<u8>, Error>;
}

impl ReadSizeAt for File {
    // Read size bytes from the file at its current stream position
    fn read_size(&mut self, size: u64) -> Result<Vec<u8>, Error> {
        // Attempt to read
        let mut data = vec![0u8; size as usize];
        self.read_exact(&mut data)?;

        Ok(data)
    }

    // Read size bytes from the file at the given offset
    fn read_size_at(
        &mut self,
        size: u64,
        offset: u64,
    ) -> Result<Vec<u8>, Error> {
        // Seek to the correct location in the file
        self.seek(SeekFrom::Start(offset))?;

        // Attempt to read
        let data = self.read_size(size)?;

        Ok(data)
    }
}
