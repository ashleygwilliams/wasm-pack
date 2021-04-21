use install::Tool;
use serde::Deserialize;

/// Krate is a lens to the `crate` JSON field of a crates.io API response.
#[derive(Debug, Deserialize)]
pub struct Krate {
    /// contains the max version published, conforming to SemVer.
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct KrateResponse {
    #[serde(rename = "crate")]
    pub krate: Krate,
}

impl Krate {
    /// looks for the crate by name on crates.io and returns the data in its `crate` JSON field.
    pub fn new(name: &Tool) -> Result<Krate, failure::Error> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let client = reqwest::Client::new();
        let mut res = client.get(&krate_address).send()?;

        let kr: KrateResponse = serde_json::from_str(&res.text()?)?;
        Ok(kr.krate)
    }
}
