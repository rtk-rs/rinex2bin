mod cli;
use cli::Cli;

extern crate log;

use env_logger::{Builder, Target};
use log::debug;

use rinex::prelude::{binex::RNX2BIN, FormattingError, ParsingError, Rinex};

use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use flate2::{write::GzEncoder, Compression};
use thiserror::Error;

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

                w.write(&buf).unwrap_or_else(|e| panic!("I/O error: {}", e));

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

    let gzip_output = cli.gzip();
    let forced_short_v2 = cli.short_bin_name();

    let rinex = if gzip_input {
        Rinex::from_gzip_file(input_path)
    } else {
        Rinex::from_file(input_path)
    };

    let rinex = rinex.unwrap_or_else(|e| panic!("RINEX parsing error: {}", e));

    let version_major = rinex.header.version.major;
    let short_v2 = forced_short_v2 || version_major < 3;

    let mut rnx2bin = rinex
        .rnx2bin(meta)
        .unwrap_or_else(|| panic!("Failed to deploy BINEX streamer"));

    if let Some(constellation) = rinex.header.constellation {
        rnx2bin.custom_announce = Some(format!(
            "rtk-rs/rinex2bin v{} from V{} {} {}",
            env!("CARGO_PKG_VERSION"),
            version_major,
            constellation,
            rinex.header.rinex_type
        ));
    } else {
        rnx2bin.custom_announce = Some(format!(
            "rtk-rs/rinex2bin v{} from V{} {}",
            env!("CARGO_PKG_VERSION"),
            version_major,
            rinex.header.rinex_type
        ));
    }

    if cli.skip_header() {
        rnx2bin.skip_header = true;
    }

    let output_path = if let Some(custom) = cli.custom_bin_name() {
        custom.to_string()
    } else {
        if gzip_output {
            rinex.standard_filename(short_v2, Some("bin..gz"), None)
        } else {
            rinex.standard_filename(short_v2, Some(".bin"), None)
        }
    };

    let fd = if let Some(stream) = cli.streaming() {
        OpenOptions::new()
            .write(true)
            .open(&stream)
            .unwrap_or_else(|e| panic!("Failed to open output stream {}: {}", stream.display(), e))
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&output_path)
            .unwrap_or_else(|e| panic!("Failed to create output file {}: {}", output_path, e))
    };

    if cli.custom_bin_name().is_some() {
        if output_path.ends_with(".gz") {
            let compression = Compression::new(5);
            let mut writer = BufWriter::new(GzEncoder::new(fd, compression));
            binex_streaming(&mut rnx2bin, &mut writer);
        } else {
            let mut writer = BufWriter::new(fd);
            binex_streaming(&mut rnx2bin, &mut writer);
        }
    } else {
        let mut writer = BufWriter::new(fd);
        binex_streaming(&mut rnx2bin, &mut writer);
    }

    Ok(())
}
