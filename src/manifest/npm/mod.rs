mod commonjs;
mod esmodules;
mod nomodules;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;
pub use self::nomodules::NoModulesPackage;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
    NoModulesPackage(NoModulesPackage),
}

impl NpmPackage {
    pub fn add_file(&mut self, file: String) {
        match self {
            Self::CommonJSPackage(pkg) => pkg.files.push(file),
            Self::ESModulesPackage(pkg) => pkg.files.push(file),
            Self::NoModulesPackage(pkg) => pkg.files.push(file),
        }
    }
}
