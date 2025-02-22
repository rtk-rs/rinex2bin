use binex::prelude::Meta;
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use std::path::{Path, PathBuf};

pub struct Cli {
    /// arguments passed by user
    pub matches: ArgMatches,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("rinex2bin")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX to BINEX")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .next_help_heading("Input")
                    .arg(
                        Arg::new("filepath")
                            .help("Input RINEX file")
                            .long_help("Input RINEX file. All supported formats may apply, that includes CRINEX.")
                            .value_name("filepath")
                            .required(true),
                    )
                    .next_help_heading("BINEX (forging)")
                    .arg(
                        Arg::new("little")
                            .short('l')
                            .long("little")
                            .action(ArgAction::SetTrue)
                            .help("Encoded stream uses Little endianness. Big endiannes is the default")
                    )
                    .arg(
                        Arg::new("crc")
                            .short('c')
                            .long("crc")
                            .action(ArgAction::SetTrue)
                            .help("Encode stream uses enhanced CRC technique (for very robust messaging).")
                        )
                    .arg(
                        Arg::new("reversed")
                            .short('r')
                            .long("rev")
                            .action(ArgAction::SetTrue)
                            .help("Forge a Reversed BINEX Stream.")
                    )
                    .next_help_heading("Output File")
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .value_name("filepath")
                            .action(ArgAction::Set)
                            .conflicts_with("stream")
                            .required(false)
                            .help("Define output BIN file name. Otherwise, BIN file name is guessed fron RINEX content.")
                    )
                    .arg(
                        Arg::new("short")
                            .short('s')
                            .long("short")
                            .action(ArgAction::SetTrue)
                            .help("Prefer V2 (short) file name when auto guessing the BIN file name.")
                    )
                    .next_help_heading("Streaming")
                    .arg(
                        Arg::new("streaming")
                            .long("stream")
                            .action(ArgAction::Set)
                            .conflicts_with("output")
                            .value_name("writable interface")
                            .required(false)
                            .help("Stream on custom I/O interface, instead of forging a BIN file.")
                    )
                    .arg(
                        Arg::new("gzip")
                            .long("gzip")
                            .action(ArgAction::SetTrue)
                            .help("Gzip compress the BINEX stream.")
                    )
                    .get_matches()
            },
        }
    }

    pub fn input_path(&self) -> PathBuf {
        Path::new(self.matches.get_one::<String>("filepath").unwrap()).to_path_buf()
    }

    pub fn custom_prefix(&self) -> Option<&String> {
        self.matches.get_one::<String>("prefix")
    }

    pub fn custom_bin_name(&self) -> Option<&String> {
        self.matches.get_one::<String>("output")
    }

    pub fn short_bin_name(&self) -> bool {
        self.matches.get_flag("short")
    }

    pub fn streaming(&self) -> Option<PathBuf> {
        let stream = self.matches.get_one::<String>("streaming")?;
        Some(Path::new(stream).to_path_buf())
    }

    pub fn gzip_output(&self) -> bool {
        self.matches.get_flag("gzip")
    }

    pub fn binex_meta(&self) -> Meta {
        Meta {
            reversed: self.matches.get_flag("reversed"),
            enhanced_crc: self.matches.get_flag("crc"),
            big_endian: !self.matches.get_flag("little"),
        }
    }
}
