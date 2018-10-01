extern crate atty;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate human_panic;
extern crate structopt;
extern crate wasm_pack;
extern crate which;

use std::env;
use structopt::StructOpt;
use wasm_pack::{command::run_wasm_pack, logger, Cli};

mod installer;

fn main() {
    setup_panic!();
    if let Err(e) = run() {
        eprintln!("{}", e);
        for cause in e.iter_causes() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), failure::Error> {
    // Deprecate `init`
    if let Some("init") = env::args().nth(1).as_ref().map(|arg| arg.as_str()) {
        println!("wasm-pack init is deprecated, consider using wasm-pack build");
    }

    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me.file_stem().and_then(|s| s.to_str()) == Some("wasm-pack-init") {
            installer::install();
        }
    }

    let args = Cli::from_args();
    let log = logger::new(&args.cmd, args.verbosity)?;
    run_wasm_pack(args.cmd, &log)?;
    Ok(())
}
