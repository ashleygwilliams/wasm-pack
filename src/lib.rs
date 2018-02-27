extern crate failure;
extern crate npmrc;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::fs;
use std::fs::File;
use std::io::prelude::*;

use failure::Error;

#[derive(Deserialize)]
struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    description: String,
    version: String,
    license: String,
    repository: String,
}

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    description: String,
    author: String,
    version: String,
    license: String,
    repository: Repository,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")] ty: String,
    url: String,
}

fn read_cargo_toml() -> Result<CargoManifest, Error> {
    let mut cargo_file = File::open("Cargo.toml")?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

impl CargoManifest {
    fn into_npm(self) -> NpmPackage {
        NpmPackage {
            name: self.package.name,
            description: self.package.description,
            author: get_npm_author().unwrap_or("".to_string()),
            version: self.package.version,
            license: self.package.license,
            repository: Repository {
                ty: "git".to_string(),
                url: self.package.repository,
            },
        }
    }
}

fn create_pkg_dir() -> Result<(), Error> {
    fs::create_dir_all("./pkg")?;
    Ok(())
}

fn get_npm_author() -> Result<String, Error> {
    let npmrc_data = npmrc::read()?;
    if npmrc_data.init_author_name.is_empty() && npmrc_data.init_author_email.is_empty() {
        return Ok("".to_string());
    }
    Ok(format!(
        "{} <{}>",
        npmrc_data.init_author_name, npmrc_data.init_author_email
    ).to_string())
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json() -> Result<(), Error> {
    create_pkg_dir()?;
    let mut pkg_file = File::create("./pkg/package.json")?;
    let crate_data = read_cargo_toml()?;
    let npm_data = crate_data.into_npm();
    let npm_json = serde_json::to_string(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    Ok(())
}
