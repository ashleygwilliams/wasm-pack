use cache;
use failure::Error;
use generate;
use install::{self, Tool};
use log::info;
use std::result;
use PBAR;

/// Executes the 'cargo-generate' command in the current directory
/// which generates a new rustwasm project from a template.
pub fn generate(
    template: String,
    name: String,
    install_permitted: bool,
) -> result::Result<(), Error> {
    info!("Generating a new rustwasm project...");
    let cargo_generate_tool = Tool::CargoGenerate;
    let version = install::Krate::new(&cargo_generate_tool)?.max_version;
    let download = install::download_prebuilt_or_cargo_install(
        cargo_generate_tool,
        &cache::get_wasm_pack_cache()?,
        &version,
        install_permitted,
    )?;
    generate::generate(&template, &name, &download)?;

    let msg = format!("🐑 Generated new project at /{}", name);
    PBAR.info(&msg);
    Ok(())
}
