use crate::lib::error::{DfxError, DfxResult};
use crate::{error_invalid_argument, error_invalid_data};
use anyhow::Context;
use fn_error_context::context;
use indicatif::{ProgressBar, ProgressDrawTarget};
use semver::Version;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

fn parse_semver<'de, D>(version: &str) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    semver::Version::parse(version)
        .map_err(|e| serde::de::Error::custom(format!("invalid SemVer: {}", e)))
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<BTreeMap<String, Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let tags: BTreeMap<String, String> = Deserialize::deserialize(deserializer)?;
    let mut result = BTreeMap::<String, Version>::new();

    for (tag, version) in tags.into_iter() {
        result.insert(tag, parse_semver::<D>(&version)?);
    }

    Ok(result)
}

fn deserialize_versions<'de, D>(deserializer: D) -> Result<Vec<Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let versions: Vec<String> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(versions.len());

    for version in versions.iter() {
        result.push(parse_semver::<D>(version)?);
    }

    Ok(result)
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deserialize_tags")]
    tags: BTreeMap<String, Version>,
    #[serde(deserialize_with = "deserialize_versions")]
    versions: Vec<Version>,
}

pub fn is_upgrade_necessary(latest_version: Option<&Version>, current: &Version) -> bool {
    match latest_version {
        Some(latest) => latest > current && current.pre.is_empty(),
        None => true,
    }
}

#[context("Failed to fetch latest version.")]
pub fn get_latest_version(
    release_root: &str,
    timeout: Option<std::time::Duration>,
) -> DfxResult<Version> {
    let url = reqwest::Url::parse(release_root)
        .map_err(|e| error_invalid_argument!("invalid release root: {}", e))?;
    let manifest_url = url
        .join("manifest.json")
        .map_err(|e| error_invalid_argument!("invalid manifest URL: {}", e))?;

    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Checking for latest dfx version...");
    b.enable_steady_tick(80);

    let client = match timeout {
        Some(timeout) => reqwest::blocking::Client::builder().timeout(timeout),
        None => reqwest::blocking::Client::builder(),
    };

    let client = client.build().context("Failed to build client.")?;
    let response = client.get(manifest_url).send().map_err(DfxError::new)?;
    let status_code = response.status();
    b.finish_and_clear();

    if !status_code.is_success() {
        return Err(error_invalid_data!(
            "unable to fetch manifest: {}",
            status_code.canonical_reason().unwrap_or("unknown error"),
        ));
    }

    let manifest: Manifest = response
        .json()
        .map_err(|e| error_invalid_data!("invalid manifest: {}", e))?;
    manifest
        .tags
        .get("latest")
        .ok_or_else(|| error_invalid_data!("expected field 'latest' in 'tags'"))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
      "tags": {
        "latest": "0.4.1"
      },
      "versions": [
        "0.3.1",
        "0.4.0",
        "0.4.1"
      ]
}"#;

    #[test]
    fn test_parse_manifest() {
        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();
        let mut tags = BTreeMap::new();
        tags.insert(
            "latest".to_string(),
            semver::Version::parse("0.4.1").unwrap(),
        );
        let versions: Vec<Version> = vec!["0.3.1", "0.4.0", "0.4.1"]
            .into_iter()
            .map(|v| semver::Version::parse(v).unwrap())
            .collect();
        assert_eq!(manifest.versions, versions);
    }

    #[test]
    fn test_get_latest_version() {
        let _ = env_logger::try_init();
        let _m = mockito::mock("GET", "/manifest.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(MANIFEST)
            .create();
        let latest_version = get_latest_version(&mockito::server_url(), None);
        assert_eq!(latest_version.unwrap(), Version::parse("0.4.1").unwrap());
        let _m = mockito::mock("GET", "/manifest.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("Not a valid JSON object")
            .create();
        let latest_version = get_latest_version(&mockito::server_url(), None);
        assert!(latest_version.is_err());
    }
}
