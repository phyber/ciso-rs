// ciso: Compress and decompress PSP ISOs
use anyhow::Result;
use std::path::PathBuf;

mod block_index;
mod cli;
mod consts;
mod header;
mod traits;
mod zlib;

fn main() -> Result<()>{
    let args = cli::parse_args();

    match args.subcommand() {
        Some(("compress", matches)) => {
            let level = *matches.get_one::<u32>("COMPRESSION_LEVEL").unwrap();
            let infile = matches.get_one::<PathBuf>("INFILE").unwrap();
            let outfile = matches.get_one::<PathBuf>("OUTFILE").unwrap();

            zlib::compress(level, infile, outfile)?;
        },
        Some(("decompress", matches)) => {
            let infile = matches.get_one::<PathBuf>("INFILE").unwrap();
            let outfile = matches.get_one::<PathBuf>("OUTFILE").unwrap();

            zlib::decompress(infile, outfile)?;
        },
        _ => unreachable!(),
    }

    Ok(())
}
