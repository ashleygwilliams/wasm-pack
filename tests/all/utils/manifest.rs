use std::io::prelude::*;
use std::path::Path;
use std::{collections::HashMap, fs::File};

use failure::Error;
use serde_json;

#[derive(Deserialize)]
pub struct NpmPackage {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub repository: Repository,
    pub files: Vec<String>,
    #[serde(default = "default_none")]
    pub main: String,
    #[serde(default = "default_none")]
    pub module: String,
    #[serde(default = "default_none")]
    pub browser: String,
    #[serde(default = "default_none")]
    pub types: String,
    #[serde(default = "default_false", rename = "sideEffects")]
    pub side_effects: bool,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub dependencies: Option<HashMap<String, String>>,
}

fn default_none() -> String {
    "".to_string()
}

fn default_false() -> bool {
    false
}

#[derive(Deserialize)]
pub struct Repository {
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

pub fn read_package_json(path: &Path, out_dir: &Path) -> Result<NpmPackage, Error> {
    let manifest_path = path.join(out_dir).join("package.json");
    let mut pkg_file = File::open(manifest_path)?;
    let mut pkg_contents = String::new();
    pkg_file.read_to_string(&mut pkg_contents)?;

    Ok(serde_json::from_str(&pkg_contents)?)
}

pub fn create_wbg_package_json(out_dir: &Path, contents: &str) -> Result<(), Error> {
    let manifest_path = out_dir.join("package.json");
    Ok(std::fs::write(manifest_path, contents)?)
}
