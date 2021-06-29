//! Support for downloading and executing `wasm-opt`

use crate::child;
use crate::install::{self, Tool};
use crate::PBAR;
use binary_install::Cache;
use std::path::Path;
use std::process::Command;

/// Execute `wasm-opt` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `wasm-opt`.
pub fn run(
    cache: &Cache,
    out_dir: &Path,
    args: &[String],
    install_permitted: bool,
) -> Result<(), failure::Error> {
    let wasm_opt = match find_wasm_opt(cache, install_permitted)? {
        install::Status::Found(path) => path,
        install::Status::CannotInstall => {
            PBAR.info("Skipping wasm-opt as no downloading was requested");
            return Ok(());
        }
        install::Status::PlatformNotSupported => {
            PBAR.info("Skipping wasm-opt because it is not supported on this platform");
            return Ok(());
        }
    };

    let wasm_opt_path = wasm_opt.binary(&Tool::WasmOpt.to_string())?;
    PBAR.info("Optimizing wasm binaries with `wasm-opt`...");

    for file in out_dir.read_dir()? {
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let tmp = path.with_extension("wasm-opt.wasm");
        let mut cmd = Command::new(&wasm_opt_path);
        cmd.arg(&path).arg("-o").arg(&tmp).args(args);
        child::run(cmd, "wasm-opt")?;
        std::fs::rename(&tmp, &path)?;
    }

    Ok(())
}

/// Attempts to find `wasm-opt` in `PATH` locally, or failing that downloads a
/// precompiled binary.
///
/// Returns `Some` if a binary was found or it was successfully downloaded.
/// Returns `None` if a binary wasn't found in `PATH` and this platform doesn't
/// have precompiled binaries. Returns an error if we failed to download the
/// binary.
pub fn find_wasm_opt(
    cache: &Cache,
    install_permitted: bool,
) -> Result<install::Status, failure::Error> {
    let version = "version_97";
    Ok(install::download_prebuilt(
        &install::Tool::WasmOpt,
        cache,
        version,
        install_permitted,
    )?)
}
