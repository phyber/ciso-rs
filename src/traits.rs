// ciso: Compress and decompress PSP ISOs
use std::fs::File;
use std::io::{
    prelude::*,
    BufReader,
    Error,
    SeekFrom,
};

pub trait ReadSizeAt {
    fn read_size_at(&mut self, size: u64, offset: u64) -> Result<Vec<u8>, Error>;
}

impl ReadSizeAt for File {
    fn read_size_at(&mut self, size: u64, offset: u64) -> Result<Vec<u8>, Error> {
        // Seek to the correct location in the file
        self.seek(SeekFrom::Start(offset))?;

        // Set up our output and readers
        let mut data = Vec::new();
        let reader = BufReader::new(self);
        let mut chunk = reader.take(size);

        // Attempt to read
        let n = chunk.read_to_end(&mut data)?;
        assert_eq!(size as usize, n);

        Ok(data)
    }
}
