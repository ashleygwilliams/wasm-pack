//! Building a Rust crate into a `.wasm` binary.

use child;
use command::build::BuildProfile;
use emoji;
use failure::{Error, ResultExt};
use manifest::Crate;
use std::path::Path;
use std::process::Command;
use std::str;
use PBAR;

pub mod wasm_target;

/// Used when comparing the currently installed
/// wasm-pack version with the latest on crates.io.
pub struct WasmPackVersion {
    /// The currently installed wasm-pack version.
    pub local: String,
    /// The latest version of wasm-pack that's released at
    /// crates.io.
    pub latest: String,
}

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version() -> Result<String, Error> {
    let local_minor_version = rustc_minor_version();
    match local_minor_version {
        Some(mv) => {
            if mv < 30 {
                bail!(
                    "Your version of Rust, '1.{}', is not supported. Please install Rust version 1.30.0 or higher.",
                    mv.to_string()
                )
            } else {
                Ok(mv.to_string())
            }
        }
        None => bail!("We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher."),
    }
}

// from https://github.com/alexcrichton/proc-macro2/blob/79e40a113b51836f33214c6d00228934b41bd4ad/build.rs#L44-L61
fn rustc_minor_version() -> Option<u32> {
    macro_rules! otry {
        ($e:expr) => {
            match $e {
                Some(e) => e,
                None => return None,
            }
        };
    }
    let output = otry!(Command::new("rustc").arg("--version").output().ok());
    let version = otry!(str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    otry!(pieces.next()).parse().ok()
}

/// Checks and returns local and latest versions of wasm-pack
pub fn check_wasm_pack_versions() -> Result<WasmPackVersion, Error> {
    match wasm_pack_local_version() {
        Some(local) => Ok(WasmPackVersion {local, latest: Crate::return_wasm_pack_latest_version()?.unwrap_or_else(|| "".to_string())}),
        None => bail!("We can't figure out what your wasm-pack version is, make sure the installation path is correct.")
    }
}

fn wasm_pack_local_version() -> Option<String> {
    let output = env!("CARGO_PKG_VERSION");
    Some(output.to_string())
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[String],
) -> Result<(), Error> {
    let msg = format!("{}Compiling to Wasm...", emoji::CYCLONE);
    PBAR.info(&msg);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--lib");

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    match profile {
        BuildProfile::Profiling => {
            // Once there are DWARF debug info consumers, force enable debug
            // info, because builds that use the release cargo profile disables
            // debug info.
            //
            // cmd.env("RUSTFLAGS", "-g");
            cmd.arg("--release");
        }
        BuildProfile::Release => {
            cmd.arg("--release");
        }
        BuildProfile::Dev => {
            // Plain cargo builds use the dev cargo profile, which includes
            // debug info by default.
        }
    }

    cmd.arg("--target").arg("wasm32-unknown-unknown");
    cmd.args(extra_options);
    child::run(cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(())
}

/// Runs `cargo build --tests` targeting `wasm32-unknown-unknown`.
///
/// This generates the `Cargo.lock` file that we use in order to know which version of
/// wasm-bindgen-cli to use when running tests.
///
/// Note that the command to build tests and the command to run tests must use the same parameters, i.e. features to be
/// disabled / enabled must be consistent for both `cargo build` and `cargo test`.
///
/// # Parameters
///
/// * `path`: Path to the crate directory to build tests.
/// * `debug`: Whether to build tests in `debug` mode.
/// * `extra_options`: Additional parameters to pass to `cargo` when building tests.
pub fn cargo_build_wasm_tests(
    path: &Path,
    debug: bool,
    extra_options: &[String],
) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");

    cmd.current_dir(path).arg("build").arg("--tests");

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    if !debug {
        cmd.arg("--release");
    }

    cmd.arg("--target").arg("wasm32-unknown-unknown");

    cmd.args(extra_options);

    child::run(cmd, "cargo build").context("Compilation of your program failed")?;
    Ok(())
}
