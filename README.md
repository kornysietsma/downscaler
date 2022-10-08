# Downscaler

Uses [ffmpeg](https://ffmpeg.org/) to process a directory tree of videos and downscale them
to 720p with a high compression ratio, e.g. for putting them on a kid's tablet.
(this can compress them to less than 1/4 of the original size!)

Assumes you have ffmpeg installed an in your path.  Ffmpeg is doing all the real work here!

I'm sharing this mostly s I think it's a nice example of using rust where once I might have written a convoluted shell script.

It also makes a nice example rust program - if people are scared by all the "rust is complex" stuff - in many cases it really isn't.  This code doesn't care about threads or `async` or the borrow checker or anything - it's very simple procedural code.  All error handling is pretty transparent.

(yes, for non-trivial rust development you will need to understand the borrow checker - but I'm just pointing out that for many specific areas of code, it won't be relevant)

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
