// ciso: Compress and decompress PSP ISOs
use anyhow::{
    Result,
};
use flate2::{
    Compress,
    Compression,
    Decompress,
    FlushCompress,
    FlushDecompress,
};
use std::fs::File;
use std::io::{
    prelude::*,
    SeekFrom,
};
use std::path::Path;

use crate::consts::{
    CISO_BLOCK_SIZE,
    CISO_HEADER_SIZE,
    CISO_WINDOW_SIZE,
};
use crate::header::CisoHeader;
use crate::traits::ReadSizeAt;

type BlockBuffer = [u8; CISO_BLOCK_SIZE as usize];

fn get_block_index(file: &mut File, total_blocks: usize) -> Result<Vec<u32>> {
    let mut block_index = Vec::new();
    let mut buffer: [u8; 4] = [0; 4];

    for _i in 0..total_blocks + 1 {
        file.read_exact(&mut buffer)?;

        let index = u32::from_le_bytes(buffer);

        block_index.push(index);
    }

    Ok(block_index)
}

fn write_block_index(file: &mut File, block_index: &[u32]) -> Result<()> {
    // Seek to after the header, which is where the block index lives.
    file.seek(SeekFrom::Start(CISO_HEADER_SIZE as u64))?;

    for block in block_index {
        let bytes = block.to_le_bytes();

        file.write_all(&bytes)?;
    }

    Ok(())
}

pub fn compress<P>(infile: P, outfile: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // ISO file to compress
    let mut infile = File::options()
        .read(true)
        .open(infile)?;

    // Compressed CSO output
    let mut outfile = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(outfile)?;

    // Get the input file size, needed for the header.
    let file_size = infile.metadata()?.len();

    let header = CisoHeader::new_with_total_bytes(file_size);
    println!("HEADER: {:#?}", header);

    // Write the header to the output file
    header.to_file(&mut outfile)?;

    // Our actual block index storage while we're compressing things
    let block_capacity = header.total_blocks() + 1;
    let mut block_index = vec![0u32; block_capacity];

    // Write out the blank block index
    write_block_index(&mut outfile, &block_index)?;

    let alignment_buffer: [u8; 64] = [0; 64];
    let mut write_pos = outfile.stream_position()?;
    let align_b = 1 << header.align();
    let align_m = align_b - 1;

    // Reuse the same compressor through all operations.
    // Must remember to reset it for each loop.
    let mut compressor = Compress::new_with_window_bits(
        Compression::new(9),
        false,
        CISO_WINDOW_SIZE,
    );

    // Start processing blocks
    for index in block_index.iter_mut().take(header.total_blocks()) {
        // Write alignment
        let mut align = write_pos & align_m;

        if align != 0 {
            println!("Aligning with alignment of: {}", align);
            align = align_b - align_m;
            println!("Fixed up alignment: {}", align);
            outfile.write_all(&alignment_buffer[0..align as usize])?;
            write_pos += align;
        }

        // Mark offset index
        let block_offset = (write_pos >> header.align()) as u32;
        *index = block_offset;

        // Read a block of data from input file
        let data = infile.read_size(header.block_size() as u64)?;
        let data_size = data.len();

        // Buffer to throw compressed data into
        let mut buffer: [u8; (CISO_BLOCK_SIZE * 2) as usize] = [0; (CISO_BLOCK_SIZE * 2) as usize];

        compressor.compress(&data, &mut buffer, FlushCompress::Finish)?;

        let compressed_size = compressor.total_out() as usize;
        compressor.reset();

        // Figure out which data we're going to write
        let writable_data = if compressed_size >= data_size {
            // Set the plain block marker
            *index |= 0x80000000;
            write_pos += data_size as u64;
            data
        }
        else {
            write_pos += compressed_size as u64;
            buffer[0..compressed_size].to_vec()
        };

        outfile.write_all(&writable_data)?;
    }

    // Set the final block to the total size
    block_index[header.total_blocks()] = write_pos as u32 >> header.align();

    // Write out the block index
    write_block_index(&mut outfile, &block_index)?;

    Ok(())
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
