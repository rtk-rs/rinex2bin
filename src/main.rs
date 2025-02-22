mod cli;
use cli::Cli;

#[macro_use]
extern crate log;

use log::{debug, info};
use env_logger::{Builder, Target};

use rinex::prelude::{binex::RNX2BIN, FormattingError, ParsingError, Rinex};

use std::{
    fs::File,
    io::{Write, BufWriter},
    path::{Path, PathBuf},
};

use flate2::{write::GzEncoder, Compression};
use thiserror::Error;

/// Supported output types
pub enum Output {
    // Simple file
    File(File),
    // Gzip compressed file
    GzipFile(GzEncoder<File>),
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(fd) => fd.flush(),
            Self::GzipFile(fd) => fd.flush(),
        }
    }
}

impl Output {
    pub fn new(
        rinex: &Rinex,
        gzip_in: bool,
        workspace: &Path,
        gzip_out: bool,
        short_name: bool,
        custom_name: Option<&String>,
    ) -> Self {
        if let Some(custom) = custom_name {
            let path = workspace.join(custom);

            let fd = File::create(&path)
                .unwrap_or_else(|e| panic!("Failed to create file within workspace: {}", e));

            if gzip_in || gzip_out {
                info!("Generating custom gzip file: {}", path.display());
                let fd = GzEncoder::new(fd, Compression::new(5));
                Output::GzipFile(fd)
            } else {
                info!("Generating custom file: {}", path.display());
                Output::File(fd)
            }
        } else {
            // auto generated name
            let mut suffix = ".bin".to_string();
            if gzip_out {
                suffix.push_str(".gz");
            }

            let auto = rinex.standard_filename(short_name, Some(&suffix), None);

            let path = workspace.join(auto);

            let fd = File::create(&path)
                .unwrap_or_else(|e| panic!("Failed to create file within workspace: {}", e));

            if gzip_in || gzip_out {
                info!("Generating gzip file: {}", path.display());
                let fd = GzEncoder::new(fd, Compression::new(5));
                Output::GzipFile(fd)
            } else {
                info!("Generating file: {}", path.display());
                Output::File(fd)
            }
        }
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("formatting error")]
    FormattingError(#[from] FormattingError),
}

fn binex_streaming<W: Write>(streamer: &mut RNX2BIN, w: &mut BufWriter<W>) {
    const BUF_SIZE: usize = 4096;
    let mut buf = [0; BUF_SIZE];
    debug!("Streaming started!");
    loop {
        match streamer.next() {
            Some(msg) => {
                debug!("Streaming: {:?}", msg);
                msg.encode(&mut buf, BUF_SIZE)
                    .unwrap_or_else(|e| panic!("BINEX encoding error: {:?}", e));

                w
                    .write(&buf)
                    .unwrap_or_else(|e| panic!("I/O error: {}", e));

                buf = [0; BUF_SIZE];
            },
            None => {},
        }
    }
}

fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();

    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    let cli = Cli::new();

    let meta = cli.binex_meta();

    let input_path = cli.input_path();
    let input_path_str = input_path.to_string_lossy().to_string();
    let gzip_input = input_path_str.ends_with(".gz");

    let rinex = if gzip_input {
        Rinex::from_gzip_file(input_path)
    } else {
        Rinex::from_file(input_path)
    };

    let rinex = rinex
        .unwrap_or_else(|e| {
            panic!("RINEX parsing error: {}", e)
    });


    let mut rnx2bin = rinex
        .rnx2bin(meta)
        .unwrap_or_else(|| panic!("Failed to deploy BINEX streamer"));


    let fd = File::create("test.bin")
        .unwrap_or_else(|e| panic!("Failed to create test.bin (output) file: {}", e));

    let mut writer = BufWriter::new(fd);
    // let mut output = Output::new(
    //     &rinex,
    //     gzip_input,
    //     &workspace,
    //     gzip_out,
    //     short_name,
    //     output_name,
    // );

    binex_streaming(&mut rnx2bin, &mut writer);
    Ok(())
}
