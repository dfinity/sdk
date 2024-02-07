use crate::lib::error::DfxResult;
use anyhow::Context;
use console::Style;
use fn_error_context::context;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use url::Url;

const DFXVM_LATEST_DIST_MANIFEST_URL: &str =
    "https://github.com/dfinity/dfxvm/releases/latest/download/dist-manifest.json";

pub fn dfxvm_released() -> DfxResult<bool> {
    let latest_version = lookup_latest_version()?;
    let latest_version = semver::Version::parse(&latest_version)
        .with_context(|| format!("Failed to parse latest version '{latest_version}'"))?;
    Ok(latest_version.minor >= 2)
}

pub fn display_dfxvm_installation_instructions() {
    println!("You can install dfxvm by running the following command:");
    println!();
    let command = Style::new()
        .cyan()
        .apply_to(r#"sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)""#);
    println!("    {command}");
}

#[derive(Deserialize, Debug)]
struct Release {
    app_name: String,
    app_version: String,
}

#[derive(Deserialize, Debug)]
struct DistManifest {
    releases: Vec<Release>,
}

#[context(
    "Failed to lookup latest dfxvm version from {}",
    DFXVM_LATEST_DIST_MANIFEST_URL
)]
pub fn lookup_latest_version() -> DfxResult<String> {
    let url = Url::parse(DFXVM_LATEST_DIST_MANIFEST_URL).unwrap();
    let dist_manifest = attempt_fetch_json::<DistManifest>(url)?;
    let dfxvm_release = dist_manifest
        .releases
        .iter()
        .find(|release| release.app_name == "dfxvm")
        .context("No app named dfxvm in latest release")?;
    let latest_version = dfxvm_release.app_version.clone();
    Ok(latest_version)
}

#[context("failed to fetch json")]
fn attempt_fetch_json<T: DeserializeOwned>(url: Url) -> DfxResult<T> {
    let response = reqwest::blocking::get(url)
        .context("GET failed")?
        .error_for_status()?;
    let bytes = response.bytes().context("read bytes failed")?;
    let doc = serde_json::from_slice(&bytes).context("parse json failed")?;
    Ok(doc)
}
