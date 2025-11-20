#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::env;
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

    // Create temp file paths in local temp directory
    let temp_dir = env::temp_dir();
    let input_path = Path::new(&input);
    let output_path = Path::new(&output);

    // Generate unique temp filenames based on the input/output filenames
    let temp_input = temp_dir.join(format!(
        "downscaler_input_{}",
        input_path.file_name().unwrap().to_string_lossy()
    ));
    let temp_output = temp_dir.join(format!(
        "downscaler_output_{}",
        output_path.file_name().unwrap().to_string_lossy()
    ));

    // Create working file path (in destination directory) by appending .working
    let mut working_output = output_path.as_os_str().to_os_string();
    working_output.push(".working");
    let working_output = PathBuf::from(working_output);

    // Clean up any pre-existing temp/working files
    if temp_input.exists() {
        debug!("removing pre-existing temp input {:?}", temp_input);
        fs::remove_file(&temp_input)?;
    }
    if temp_output.exists() {
        debug!("removing pre-existing temp output {:?}", temp_output);
        fs::remove_file(&temp_output)?;
    }
    if working_output.exists() {
        debug!("removing pre-existing working output {:?}", working_output);
        fs::remove_file(&working_output)?;
    }

    // Copy source to temp input
    info!("copying source to temp location {:?}", temp_input);
    fs::copy(input_path, &temp_input)?;

    // Run ffmpeg on temp files
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(&temp_input)
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
        .arg(&temp_output);
    // echo cmd to stderr
    warn!("{:?}", cmd);

    let status = cmd.status()?;

    match status.code() {
        Some(0) => {
            info!("ffmpeg succeeded");
        }
        Some(code) => {
            return Err(anyhow!("Exited with status code: {}", code));
        }
        None => {
            return Err(anyhow!("Process terminated."));
        }
    }

    // Copy temp output to working file in destination directory
    info!("copying result to working file {:?}", working_output);
    fs::copy(&temp_output, &working_output)?;

    // Atomically rename working file to final destination
    info!("renaming to final destination {:?}", output_path);
    fs::rename(&working_output, output_path)?;

    // Clean up temp files
    debug!("cleaning up temp files");
    fs::remove_file(&temp_input)?;
    fs::remove_file(&temp_output)?;

    info!("Succeeded");
    Ok(())
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

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Opts {
    #[clap(value_parser, short, long)]
    source: PathBuf,
    #[clap(value_parser, short, long)]
    destination: PathBuf,
}

fn main() -> Result<()> {
    // set log level to info
    // override with `RUST_LOG=debug` or similar
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opts = Opts::try_parse()?;

    if !Path::new(&opts.source).is_dir() {
        return Err(anyhow!("Source path {:?} does not exist", &opts.source));
    }

    downscale_recursive(&opts.source, &opts.destination, &Vec::new())
}
