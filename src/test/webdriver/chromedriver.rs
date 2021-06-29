use super::{get_and_notify, Collector};
use binary_install::Cache;
use chrono::DateTime;
use failure::{self, ResultExt};
use install::InstallMode;
use stamps;
use std::path::PathBuf;
use target;

// Keep it up to date with each `wasm-pack` release.
// https://chromedriver.storage.googleapis.com/LATEST_RELEASE
const DEFAULT_CHROMEDRIVER_VERSION: &str = "86.0.4240.22";

const CHROMEDRIVER_LAST_UPDATED_STAMP: &str = "chromedriver_last_updated";
const CHROMEDRIVER_VERSION_STAMP: &str = "chromedriver_version";

/// Get the path to an existing `chromedriver`, or install it if no existing
/// binary is found or if there is a new binary version.
pub fn get_or_install_chromedriver(
    cache: &Cache,
    mode: InstallMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("chromedriver") {
        return Ok(path);
    }
    install_chromedriver(cache, mode.install_permitted())
}

/// Download and install a pre-built `chromedriver` binary.
pub fn install_chromedriver(
    cache: &Cache,
    installation_allowed: bool,
) -> Result<PathBuf, failure::Error> {
    let target = if target::LINUX && target::x86_64 {
        "linux64"
    } else if target::MACOS && target::x86_64 {
        "mac64"
    } else if target::WINDOWS {
        "win32"
    } else {
        bail!("chromedriver binaries are unavailable for this target")
    };

    let url = get_chromedriver_url(target);

    match get_and_notify(cache, installation_allowed, "chromedriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `chromedriver` binary found, and could not find a global \
             `chromedriver` on the `$PATH`. Not installing `chromedriver` because of noinstall \
             mode."
        ),
    }
}

/// Get `chromedriver` download URL.
///
/// _Algorithm_:
/// 1. Try to open `*.stamps` file and deserialize its content to JSON object.
/// 2. Try to compare current time with the saved one.
/// 3. If the saved time is older than 1 day or something failed
///    => fetch a new version and save version & time.
/// 4. If everything failed, use the default version.
/// 5. Return URL.
///
/// _Notes:_
///
/// It returns the latest one without checking the installed `Chrome` version
/// because it's not easy to find out `Chrome` version on `Windows` -
/// https://bugs.chromium.org/p/chromium/issues/detail?id=158372
///
/// The official algorithm for `chromedriver` version selection:
/// https://chromedriver.chromium.org/downloads/version-selection
fn get_chromedriver_url(target: &str) -> String {
    let fetch_and_save_version =
        || fetch_chromedriver_version().and_then(save_chromedriver_version);

    let chromedriver_version = match stamps::read_stamps_file_to_json() {
        Ok(json) => {
            if should_load_chromedriver_version_from_stamp(&json) {
                stamps::get_stamp_value(CHROMEDRIVER_VERSION_STAMP, &json)
            } else {
                fetch_and_save_version()
            }
        }
        Err(_) => fetch_and_save_version(),
    }
    .unwrap_or_else(|error| {
        log::warn!(
            "Cannot load or fetch chromedriver's latest version data, \
             the default version {} will be used. Error: {}",
            DEFAULT_CHROMEDRIVER_VERSION,
            error
        );
        DEFAULT_CHROMEDRIVER_VERSION.to_owned()
    });
    assemble_chromedriver_url(&chromedriver_version, target)
}

// ------ `get_chromedriver_url` helpers ------

fn save_chromedriver_version(version: String) -> Result<String, failure::Error> {
    stamps::save_stamp_value(CHROMEDRIVER_VERSION_STAMP, &version)?;

    let current_time = chrono::offset::Local::now().to_rfc3339();
    stamps::save_stamp_value(CHROMEDRIVER_LAST_UPDATED_STAMP, current_time)?;

    Ok(version)
}

fn should_load_chromedriver_version_from_stamp(json: &serde_json::Value) -> bool {
    let last_updated = stamps::get_stamp_value(CHROMEDRIVER_LAST_UPDATED_STAMP, json)
        .ok()
        .and_then(|last_updated| DateTime::parse_from_rfc3339(&last_updated).ok());

    match last_updated {
        None => false,
        Some(last_updated) => {
            let current_time = chrono::offset::Local::now();
            current_time.signed_duration_since(last_updated).num_hours() < 24
        }
    }
}

fn fetch_chromedriver_version() -> Result<String, failure::Error> {
    let mut handle = curl::easy::Easy2::new(Collector(Vec::new()));
    handle
        .url("https://chromedriver.storage.googleapis.com/LATEST_RELEASE")
        .context("URL to fetch chromedriver's LATEST_RELEASE is invalid")?;
    handle
        .perform()
        .context("fetching of chromedriver's LATEST_RELEASE failed")?;

    let content = handle.get_mut().take_content();
    let version =
        String::from_utf8(content).context("chromedriver's LATEST_RELEASE is not valid UTF-8")?;
    Ok(version)
}

fn assemble_chromedriver_url(chromedriver_version: &str, target: &str) -> String {
    format!(
        "https://chromedriver.storage.googleapis.com/{version}/chromedriver_{target}.zip",
        version = chromedriver_version,
        target = target,
    )
}
