// ciso: Compress and decompress PSP ISOs
use anyhow::{
    Result,
};
use flate2::{
    Decompress,
    FlushDecompress,
};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::header::CisoHeader;
use crate::traits::ReadSizeAt;

const CISO_BLOCK_SIZE: u32 = 0x800; // 2048 bytes
const CISO_WINDOW_SIZE: u8 = 15; // Window size

type BlockBuffer = [u8; CISO_BLOCK_SIZE as usize];

fn get_block_index(file: &mut File, total_blocks: usize) -> Result<Vec<u32>> {
    let mut block_index = Vec::new();
    let mut buffer: [u8; 4] = [0; 4];

    for _i in 0..total_blocks + 1 {
        file.read_exact(&mut buffer).unwrap();

        let index = u32::from_le_bytes(buffer.try_into()?);

        block_index.push(index);
    }

    Ok(block_index)
}

pub fn decompress<P>(infile: P, outfile: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut infile = File::options()
        .read(true)
        .open(infile)?;

    let header = CisoHeader::try_from(&mut infile).unwrap();
    println!("{}", header);

    let total_blocks = header.total_blocks();
    let block_index = get_block_index(&mut infile, total_blocks)?;

    let mut outfile = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(outfile)?;

    for block in 0..total_blocks {
        let index = block_index[block];

        // Masks off the top most bit to see if the block is compressed
        let plain = index & 0x80000000 != 0;

        // Get the actual position of the block
        let index = index & 0x7fffffff;

        // Get the read position of the block in the compressed file
        let read_pos = (index << header.align()) as u64;

        let read_size = if plain {
            // If it's a plain block, use the full block size as the read size.
            header.block_size() as u64
        }
        else {
            // If it's a compressed block, we also get the next block and read
            // some more.
            let next_block = (block + 1) as usize;
            let index2 = block_index[next_block] & 0x7fffffff;
            let read_size = (index2 - index) << header.align();

            read_size as u64
        };

        // Should error if we can't read the size of buffer
        // Rename this to data later
        let data = infile.read_size_at(read_size, read_pos)?;

        let decompressed_data = if plain {
            data
        }
        else {
            // No header on our data, and a custom window size.
            let mut d = Decompress::new_with_window_bits(
                false,
                CISO_WINDOW_SIZE,
            );
            let mut buffer: BlockBuffer = [0; CISO_BLOCK_SIZE as usize];

            d.decompress(&data, &mut buffer, FlushDecompress::None)?;

            buffer.to_vec()
        };

        outfile.write_all(&decompressed_data)?;
    }

    Ok(())
}
