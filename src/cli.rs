// cli
use clap::{
    crate_description,
    crate_name,
    crate_version,
    value_parser,
    Arg,
    ArgMatches,
    Command,
};
use std::path::PathBuf;

fn create_app<'a>() -> Command<'a> {
    let app = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .term_width(80);

    let compress = Command::new("compress")
        .about("Compress an ISO")
        .arg(
            Arg::new("COMPRESSION_LEVEL")
                .default_value("9")
                .long("level")
                .help("Compression level, 1-9")
                .required(false)
                .short('l')
                .takes_value(true)
                .value_name("LEVEL")
                .value_parser(value_parser!(u32).range(1..=9))
        )
        .arg(
            Arg::new("INFILE")
                .help("Input ISO filename")
                .index(1)
                .required(true)
                .takes_value(true)
                .value_name("INFILE")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("OUTFILE")
                .help("Output CSO filename")
                .index(2)
                .required(true)
                .takes_value(true)
                .value_name("OUTFILE")
                .value_parser(value_parser!(PathBuf))
        );

    let decompress = Command::new("decompress")
        .about("Decompress a CSO")
        .arg(
            Arg::new("INFILE")
                .help("Input CSO filename")
                .index(1)
                .required(true)
                .takes_value(true)
                .value_name("INFILE")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("OUTFILE")
                .help("Output ISO filename")
                .index(2)
                .required(true)
                .takes_value(true)
                .value_name("OUTFILE")
                .value_parser(value_parser!(PathBuf))
        );

    app
        .subcommand(compress)
        .subcommand(decompress)
}

pub fn parse_args() -> ArgMatches {
    create_app().get_matches()
}
