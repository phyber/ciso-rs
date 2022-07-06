// ciso: Compress and decompress PSP ISOs
use anyhow::Result;

mod consts;
mod header;
mod traits;
mod zlib;

fn main() -> Result<()>{
    let args: Vec<String> = ::std::env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: ciso mode infile outfile");
        ::std::process::exit(1);
    }

    let mode = &args[1];
    let infile = &args[2];
    let outfile = &args[3];

    match mode.as_str() {
        "compress"   => zlib::compress(infile, outfile)?,
        "decompress" => zlib::decompress(infile, outfile)?,
        _ => {
            eprintln!("Unknown mode: {}", mode);
            ::std::process::exit(1);
        },
    }

    Ok(())
}
