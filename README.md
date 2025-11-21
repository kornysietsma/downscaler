# Downscaler

Uses [ffmpeg](https://ffmpeg.org/) to process a directory tree of videos and downscale them
with high compression using libx265 (H.265/HEVC), e.g. for putting them on a kid's tablet.
(this can compress them to less than 1/4 of the original size!)

Assumes you have ffmpeg installed an in your path.  Ffmpeg is doing all the real work here!

I'm sharing this mostly s I think it's a nice example of using rust where once I might have written a convoluted shell script.

It also makes a nice example rust program - if people are scared by all the "rust is complex" stuff - in many cases it really isn't.  This code doesn't care about threads or `async` or the borrow checker or anything - it's very simple procedural code.  All error handling is pretty transparent.

(yes, for non-trivial rust development you will need to understand the borrow checker - but I'm just pointing out that for many specific areas of code, it won't be relevant)

## Usage

```sh
downscaler --source <SOURCE> --destination <DESTINATION> [OPTIONS]
```

### Scaling Options

By default, videos are re-encoded without scaling (preserving original resolution). You can specify scaling behavior:

- `--scale <HEIGHT>`: Set default maximum height for all videos (e.g., `--scale 720` or `--scale 1080`)
- `--override <DIR:HEIGHT>`: Override scale for specific directories (e.g., `--override movies:1080`)

**Important notes:**
- Overrides match from the source root as whole path components
- `--override tv:720` matches `tv/kids/video.mp4` but NOT `movies/tv/video.mp4` or `tvfish/video.mp4`
- Subdirectories work: `--override tv/kids:480` matches only files under `tv/kids/`
- More specific overrides take precedence over less specific ones
- Scaling only downscales, never upscales (uses ffmpeg's `min()` function)

### Examples

```sh
# Re-encode without scaling (default)
cargo run -- -s /videos/source -d /videos/dest

# Downscale everything to 720p
cargo run -- -s /videos/source -d /videos/dest --scale 720

# No scaling by default, but 720p for tv shows and 1080p for movies
cargo run -- -s /videos/source -d /videos/dest \
  --override tv:720 \
  --override movies:1080

# 720p default, with specific overrides
cargo run -- -s /videos/source -d /videos/dest \
  --scale 720 \
  --override movies:1080 \
  --override movies/kids:480
```

In the last example, a file at `movies/kids/cartoon.mp4` gets 480p (most specific match wins).

## Error handling

Everything returns a [Result<T,E>](https://doc.rust-lang.org/std/result/index.html) - in this case implemented by `Anyhow::Result<T>` which basically means "return either a valid result type T or an Error type E".  The caller _must_ handle the error - to not handle it is a compilation error.

There are many many function calls that end with `?` - this is rust syntax for shortcut error handling.  Code like `downscale(...)?` calls `downscale` and if it returns an Error, immediately exit this function returning an Error - otherwise it unwraps the return value of `downscale` (in this case the return value is `()` - that is to say, nothing).  This is the standard Rust approach to error handling without exceptions.

This means that without any real code on my part, many many errors will be trapped and logged.  If navigating the directory tree meets a directory I can't read, then some I/O call will return an `Error`, which will bubble up to `main` and be displayed.

### libraries

I have a few standard libraries I like to use for command-line tools:

* [anyhow](https://crates.io/crates/anyhow) - makes simple error handling simple
* [clap](https://crates.io/crates/clap) - command-line parsing
* [log](https://crates.io/crates/log) - standard rust log facade
* [env_logger](https://crates.io/crates/env_logger) - a simple logger with configuration through environment variables

## logging

Specify log level by setting `RUST_LOG` e.g.:

```sh
RUST_LOG=debug cargo run
```
