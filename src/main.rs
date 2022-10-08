#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Result;
use env_logger::Env;

use clap::Parser;
use log::debug;
use log::info;
use log::warn;

fn downscale(input: OsString, output: OsString) -> Result<()> {
    info!("downscaling {:?} to {:?}", input, output);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(input)
        .args([
            "-c:v",
            "libx265",
            "-crf",
            "28",
            "-preset",
            "fast",
            "-c:a",
            "copy",
            "-vf",
            "scale=-2:'min(720,ih)'",
            "-loglevel",
            "warning",
            "-nostats",
            "-hide_banner",
            "-x265-params",
            "log-level=error",
        ])
        .arg(output);

    let status = cmd.status()?;

    match status.code() {
        Some(0) => {
            info!("Succeeded");
            Ok(())
        }
        Some(code) => Err(anyhow!("Exited with status code: {}", code)),
        None => Err(anyhow!("Process terminated.")),
    }
}

fn downscale_recursive(root_source: &Path, root_dest: &Path, suffix: &Vec<OsString>) -> Result<()> {
    let mut source = PathBuf::from(root_source);
    let mut dest = PathBuf::from(root_dest);
    for dir in suffix {
        source.push(&dir);
        dest.push(&dir);
    }
    assert!(&source.is_dir(), "Source is not a directory?!");

    for entry in fs::read_dir(&source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let mut new_suffix: Vec<OsString> = suffix.clone();
            new_suffix.push(entry.file_name());
            downscale_recursive(root_source, root_dest, &new_suffix)?;
        } else if file_type.is_file() {
            let source_file = entry.path();
            if let Some(ext) = source_file.extension() {
                if ext == "mp4" || ext == "mkv" {
                    if !dest.is_dir() {
                        fs::create_dir_all(&dest)?;
                    }
                    let mut dest_file = dest.clone();
                    dest_file.push(Path::new(&entry.file_name()));
                    if dest_file.exists() {
                        debug!("not overwriting {:?}", &dest_file);
                    } else {
                        downscale(source_file.into_os_string(), dest_file.into_os_string())?;
                    }
                } else {
                    debug!("ignoring file - wrong extension {:?}", &source_file);
                }
            } else {
                debug!("ignoring file - no extension {:?}", &source_file);
            }
        } else {
            debug!("ignoring file type {:?}", file_type);
        }
    }

    Ok(())
}

fn validate_path_exists(path: &str) -> Result<(), String> {
    if !Path::new(path).is_dir() {
        Err(format!("Path {} does not exist", path))
    } else {
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Opts {
    #[clap(value_parser, short, long, value_parser=validate_path_exists)]
    source: PathBuf,
    #[clap(value_parser, short, long)]
    destination: PathBuf,
}

fn main() -> Result<()> {
    // set log level to info
    // override with `RUST_LOG=debug` or similar
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opts = Opts::try_parse()?;

    downscale_recursive(&opts.source, &opts.destination, &Vec::new())
}
