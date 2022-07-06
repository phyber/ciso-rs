// ciso: Compress and decompress PSP ISOs
use anyhow::{
    Result,
};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::consts::{
    CISO_BLOCK_SIZE,
    CISO_HEADER_SIZE,
    CISO_MAGIC,
};

#[derive(Debug)]
pub enum CisoError {
    HeaderError,
    MagicError,
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct CisoHeader {
    magic:       [u8; 4],
    header_size: u32,
    total_bytes: u64,
    block_size:  u32,
    version:     u8,
    align:       u8,
    _reserved:   [u8; 2],
}

impl CisoHeader {
    pub fn new_with_total_bytes(total_bytes: u64) -> Self {
        Self {
            total_bytes,
            magic:       CISO_MAGIC.to_le_bytes(),
            header_size: CISO_HEADER_SIZE,
            block_size:  CISO_BLOCK_SIZE,
            version:     1,
            ..Self::default()
        }
    }

    pub fn align(&self) -> u8 {
        self.align
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn total_blocks(&self) -> usize {
        (self.total_bytes / self.block_size as u64) as usize
    }

    // There might be a safe way of doing this, if so, replace this.
    pub unsafe fn as_bytes(&self) -> &[u8] {
        ::std::slice::from_raw_parts(
            (self as *const Self) as *const u8,
            ::std::mem::size_of::<Self>(),
        )
    }

    pub fn to_file(&self, file: &mut File) -> Result<(), ::std::io::Error> {
        let data = unsafe { self.as_bytes() };
        file.write_all(data)
    }
}

impl TryFrom<&mut File> for CisoHeader {
    type Error = CisoError;

    fn try_from(file: &mut File) -> Result<Self, CisoError> {
        let mut buffer: [u8; CISO_HEADER_SIZE as usize] = [0; CISO_HEADER_SIZE as usize];

        file.read_exact(&mut buffer)
            .map_err(|_| CisoError::HeaderError)?;

        // Quick header check
        let magic: [u8; 4] = [buffer[0], buffer[1], buffer[2], buffer[3]];

        if u32::from_le_bytes(magic) != CISO_MAGIC {
            eprintln!("invalid file magic");
            return Err(CisoError::MagicError);
        }

        let header = CisoHeader {
            magic,
            header_size: u32::from_le_bytes(buffer[4..8].try_into().unwrap()),
            total_bytes: u64::from_le_bytes(buffer[8..16].try_into().unwrap()),
            block_size:  u32::from_le_bytes(buffer[16..20].try_into().unwrap()),
            version:     buffer[20],
            align:       buffer[21],
            _reserved:   [buffer[22], buffer[23]],
        };

        if header.block_size == 0 || header.total_bytes == 0 {
            eprintln!(
                "invalid block_size ({}) or total_bytes ({})",
                header.block_size,
                header.total_bytes,
            );
            return Err(CisoError::HeaderError);
        }

        if header.header_size != CISO_HEADER_SIZE {
            eprintln!("Incorrect header size found, ignoring.");
        }

        println!("HEADER: {:#?}", header);

        Ok(header)
    }
}

impl fmt::Display for CisoHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            concat!(
                "Magic: {magic:?}\n",
                "Header Size: {header_size}\n",
                "Total Bytes: {total_bytes}\n",
                "Block Size: {block_size}\n",
                "Version: {version}\n",
                "Align: {align}\n",
            ),
            magic = self.magic,
            header_size = self.header_size,
            total_bytes = self.total_bytes,
            block_size = self.block_size,
            version = self.version,
            align = self.align,
        )
    }
}
