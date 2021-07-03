//! Self-installation of `wasm-pack`
//!
//! This module contains one public function which will self-install the
//! currently running executable as `wasm-pack`. Our goal is to install this in
//! a place that's already in `PATH`, ideally in an idiomatic location. To that
//! end we place `wasm-pack` next to the `rustup` executable in `PATH`.
//!
//! This installer is run directly (probably by clicking on it) on Windows,
//! meaning it will pop up a console (as we're a console app). Output goes to
//! the console and users interact with it through the console. On Unix this is
//! intended to be run from a shell script (docs/installer/init.sh) which is
//! downloaded via curl/sh, and then the shell script downloads this executable
//! and runs it.
//!
//! This may get more complicated over time (self upates anyone?) but for now
//! it's pretty simple! We're largely just moving over our currently running
//! executable to a different path.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use atty;
use failure::{self, ResultExt};
use which;

pub fn install() -> ! {
    if let Err(e) = do_install() {
        eprintln!("{}", e);
        for cause in e.iter_causes() {
            eprintln!("Caused by: {}", cause);
        }
    }

    // On Windows we likely popped up a console for the installation. If we were
    // to exit here immediately then the user wouldn't see any error that
    // happened above or any successful message. Let's wait for them to say
    // they've read everything and then continue.
    if cfg!(windows) {
        println!("Press enter to close this window...");
        let mut line = String::new();
        drop(io::stdin().read_line(&mut line));
    }

    process::exit(0);
}

fn do_install() -> Result<(), failure::Error> {
    // First check to see if the environmental variable $CARGO_HOME is set and
    // install wasm-pack into this directory.  Otherwise find `rustup.exe` in PATH,
    // and use its installation directory as our installation directory.
    let mut installation_dir: PathBuf = PathBuf::new();
    let cargo_home: Result<String, env::VarError> = env::var("CARGO_HOME");
    match cargo_home {
        Ok(path) => {
            installation_dir.push(path);
            installation_dir.push("bin");
        }
        Err(_) => {
            println!("$CARGO_HOME not set, default to rustup location for install");
            let rustup = match which::which("rustup") {
                Ok(path) => path,
                Err(_) => {
                    bail!(
                        "failed to find an installation of `rustup` in `PATH`, \
                         is rustup already installed?"
                    );
                }
            };
            let rustup_dir = match rustup.parent() {
                Some(parent) => parent,
                None => bail!(
                    "can't install when `rustup` is at the root of, \
                     the filesystem"
                ),
            };
            installation_dir.push(rustup_dir)
        }
    };

    let destination = installation_dir
        .join("wasm-pack")
        .with_extension(env::consts::EXE_EXTENSION);

    if destination.exists() {
        confirm_can_overwrite(&destination)?;
    }

    // Our relatively simple install step!
    let me = env::current_exe()?;
    match fs::copy(&me, &destination) {
        Ok(_val) => {
            println!(
                "info: successfully installed wasm-pack to `{}`",
                destination.display()
            );
        }
        Err(_) => {
            bail!(
                "Unable to write to {}.  Set $CARGO_HOME to install elsewhere.",
                destination.display()
            );
        }
    };

    // ... and that's it!
    Ok(())
}

fn confirm_can_overwrite(dst: &Path) -> Result<(), failure::Error> {
    // If the `-f` argument was passed, we can always overwrite everything.
    if env::args().any(|arg| arg == "-f") {
        return Ok(());
    }

    // If we're not attached to a TTY then we can't get user input, so there's
    // nothing to do except inform the user about the `-f` flag.
    if !atty::is(atty::Stream::Stdin) {
        bail!(
            "existing wasm-pack installation found at `{}`, pass `-f` to \
             force installation over this file, otherwise aborting \
             installation now",
            dst.display()
        );
    }

    // It looks like we're at an interactive prompt, so ask the user if they'd
    // like to overwrite the previous installation.
    eprintln!(
        "info: existing wasm-pack installation found at `{}`",
        dst.display()
    );
    eprint!("info: would you like to overwrite this file? [y/N]: ");
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .with_context(|_| "failed to read stdin")?;

    if line.starts_with('y') || line.starts_with('Y') {
        return Ok(());
    }

    bail!("aborting installation");
}
