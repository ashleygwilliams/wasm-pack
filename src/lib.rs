//! Your favorite rust -> wasm workflow tool!

#![deny(missing_docs)]

extern crate cargo_metadata;
extern crate console;
extern crate strsim;
#[macro_use]
extern crate failure;
extern crate glob;
extern crate semver;
extern crate serde;
extern crate which;
#[macro_use]
extern crate serde_derive;
extern crate serde_ignored;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate binary_install;
extern crate chrono;
extern crate curl;
extern crate dialoguer;
extern crate log;
extern crate toml;
extern crate walkdir;

pub mod bindgen;
pub mod build;
pub mod cache;
pub mod child;
pub mod command;
pub mod emoji;
pub mod generate;
pub mod install;
pub mod license;
pub mod lockfile;
pub mod manifest;
pub mod npm;
pub mod progressbar;
pub mod readme;
pub mod stamps;
pub mod target;
pub mod test;
pub mod wasm_opt;

use progressbar::{LogLevel, ProgressOutput};

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();

/// 📦 ✨  pack and publish your wasm!
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// The subcommand to run.
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: command::Command,

    /// Log verbosity is based off the number of v used
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    pub verbosity: u8,

    #[structopt(long = "quiet", short = "q")]
    /// No output printed to stdout
    pub quiet: bool,

    #[structopt(long = "log-level", default_value = "info")]
    /// The maximum level of messages that should be logged by wasm-pack. [possible values: info, warn, error]
    pub log_level: LogLevel,
}
