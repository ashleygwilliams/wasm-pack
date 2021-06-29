use super::{get_and_notify, Collector};
use binary_install::Cache;
use chrono::DateTime;
use failure::{self, ResultExt};
use install::InstallMode;
use stamps;
use std::path::PathBuf;
use target;

// Keep it up to date with each `wasm-pack` release.
// https://github.com/mozilla/geckodriver/releases/latest
const DEFAULT_GECKODRIVER_VERSION: &str = "v0.27.0";

const GECKODRIVER_LAST_UPDATED_STAMP: &str = "geckodriver_last_updated";
const GECKODRIVER_VERSION_STAMP: &str = "geckodriver_version";

/// Get the path to an existing `geckodriver`, or install it if no existing
/// binary is found or if there is a new binary version.
pub fn get_or_install_geckodriver(
    cache: &Cache,
    mode: InstallMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("geckodriver") {
        log::info!("[geckodriver] Found geckodriver at {:?}", path);
        return Ok(path);
    }
    install_geckodriver(cache, mode.install_permitted())
}

/// Download and install a pre-built `geckodriver` binary.
pub fn install_geckodriver(
    cache: &Cache,
    installation_allowed: bool,
) -> Result<PathBuf, failure::Error> {
    let (target, ext) = if target::LINUX && target::x86 {
        ("linux32", "tar.gz")
    } else if target::LINUX && target::x86_64 {
        ("linux64", "tar.gz")
    } else if target::MACOS {
        ("macos", "tar.gz")
    } else if target::WINDOWS && target::x86 {
        ("win32", "zip")
    } else if target::WINDOWS && target::x86_64 {
        ("win64", "zip")
    } else {
        bail!("geckodriver binaries are unavailable for this target")
    };

    let url = get_geckodriver_url(target, ext);

    match get_and_notify(cache, installation_allowed, "geckodriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `geckodriver` binary found, and could not find a global `geckodriver` \
             on the `$PATH`. Not installing `geckodriver` because of noinstall mode."
        ),
    }
}

/// Get `geckodriver` download URL.
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
/// It returns the latest one without checking the installed `Firefox` version
/// - it should be relatively safe because each `geckodriver` supports many `Firefox` versions:
/// https://firefox-source-docs.mozilla.org/testing/geckodriver/Support.html#supported-platforms
fn get_geckodriver_url(target: &str, ext: &str) -> String {
    let fetch_and_save_version = || {
        fetch_latest_geckodriver_tag_json()
            .and_then(get_version_from_json)
            .and_then(save_geckodriver_version)
    };

    log::info!("[geckodriver] Looking up latest version of geckodriver...");
    let geckodriver_version = match stamps::read_stamps_file_to_json() {
        Ok(json) => {
            if should_load_geckodriver_version_from_stamp(&json) {
                stamps::get_stamp_value(GECKODRIVER_VERSION_STAMP, &json)
            } else {
                fetch_and_save_version()
            }
        }
        Err(_) => fetch_and_save_version(),
    }
    .unwrap_or_else(|error| {
        log::warn!(
            "[geckodriver] Cannot load or fetch geckodriver's latest version data, \
                 the default version {} will be used. Error: {}",
            DEFAULT_GECKODRIVER_VERSION,
            error
        );
        DEFAULT_GECKODRIVER_VERSION.to_owned()
    });
    let url = assemble_geckodriver_url(&geckodriver_version, target, ext);
    log::info!("[geckodriver] Fetching geckodriver at {}", url);
    url
}

// ------ `get_geckodriver_url` helpers  ------

fn save_geckodriver_version(version: String) -> Result<String, failure::Error> {
    stamps::save_stamp_value(GECKODRIVER_VERSION_STAMP, &version)?;

    let current_time = chrono::offset::Local::now().to_rfc3339();
    stamps::save_stamp_value(GECKODRIVER_LAST_UPDATED_STAMP, current_time)?;

    Ok(version)
}

fn should_load_geckodriver_version_from_stamp(json: &serde_json::Value) -> bool {
    let last_updated = stamps::get_stamp_value(GECKODRIVER_LAST_UPDATED_STAMP, json)
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

fn fetch_latest_geckodriver_tag_json() -> Result<String, failure::Error> {
    let mut headers = curl::easy::List::new();
    headers
        .append("Accept: application/json")
        .context("cannot fetch geckodriver's latest release data - appending header failed")?;

    let mut handle = curl::easy::Easy2::new(Collector(Vec::new()));
    handle
        .url("https://github.com/mozilla/geckodriver/releases/latest")
        .context("URL to fetch geckodriver's latest release data is invalid")?;
    handle
        .http_headers(headers)
        .context("cannot fetch geckodriver's latest release data - setting headers failed")?;
    // We will be redirected from the `latest` placeholder to the specific tag name.
    handle
        .follow_location(true)
        .context("cannot fetch geckodriver's latest release data - enabling redirects failed")?;
    handle
        .perform()
        .context("fetching of geckodriver's latest release data failed")?;

    let content = handle.get_mut().take_content();
    let version = String::from_utf8(content)
        .context("geckodriver's latest release data is not valid UTF-8")?;

    Ok(version)
}

/// JSON example: `{"id":15227534,"tag_name":"v0.24.0","update_url":"/mozzila...`
fn get_version_from_json(json: impl AsRef<str>) -> Result<String, failure::Error> {
    let json: serde_json::Value = serde_json::from_str(json.as_ref())
        .context("geckodriver's latest release data is not valid JSON")?;
    json.get("tag_name")
        .and_then(|tag_name| tag_name.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| {
            failure::err_msg("cannot get `tag_name` from geckodriver's latest release data")
        })
}

fn assemble_geckodriver_url(_tag: &str, target: &str, ext: &str) -> String {
    format!(
        "https://github.com/mozilla/geckodriver/releases/download/{tag}/geckodriver-{tag}-{target}.{ext}",
        // TODO: Temporary use v0.24.0 instead of latest. Latest is currently not working...
        tag="v0.24.0",
        target=target,
        ext=ext,
    )
}
