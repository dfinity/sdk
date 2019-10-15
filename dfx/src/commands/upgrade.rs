use crate::lib::{
    env::VersionEnv,
    error::{DfxError, DfxResult},
};
use clap::{App, Arg, ArgMatches, SubCommand};
use semver::Version;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("upgrade")
        .about("Upgrade DFX.")
        .arg(
            Arg::with_name("current-version")
                .hidden(true)
                .long("current-version")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("release-root")
                .default_value("http://localhost:8080/")
                .hidden(true)
                .long("release-root")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Verbose output.")
                .long("verbose"),
        )
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<HashMap<String, Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let tags: HashMap<String, String> = Deserialize::deserialize(deserializer)?;
    let mut result = HashMap::<String, Version>::new();

    for (tag, version) in tags.into_iter() {
        result.insert(
            tag,
            semver::Version::parse(&version)
                .map_err(|e| serde::de::Error::custom(format!("invalid SemVer version: {}", e)))?,
        );
    }

    Ok(result)
}

fn deserialize_versions<'de, D>(deserializer: D) -> Result<Vec<Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let versions: Vec<String> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(versions.len());

    for version in versions.into_iter() {
        result.push(
            semver::Version::parse(&version)
                .map_err(|e| serde::de::Error::custom(format!("invalid SemVer version: {}", e)))?,
        );
    }

    Ok(result)
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Manifest {
    #[serde(deserialize_with = "deserialize_tags")]
    tags: HashMap<String, Version>,
    #[serde(deserialize_with = "deserialize_versions")]
    versions: Vec<Version>,
}

fn get_latest_version(release_root: &str) -> DfxResult<Version> {
    let url = reqwest::Url::parse(release_root)
        .map_err(|e| DfxError::InvalidArgument(format!("invalid release root: {}", e)))?;
    let manifest_url = url
        .join("manifest.json")
        .map_err(|e| DfxError::InvalidArgument(format!("invalid manifest URL: {}", e)))?;
    let manifest: Manifest = reqwest::get(manifest_url)
        .map_err(DfxError::Reqwest)?
        .json()
        .map_err(|e| DfxError::InvalidData(format!("invalid manifest: {}", e)))?;
    manifest
        .tags
        .get("latest")
        .ok_or(DfxError::InvalidData(
            "expected field 'latest' in 'tags'".to_string(),
        ))
        .map(|v| v.clone())
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: VersionEnv,
{
    let current_version = Version::parse(env.get_version())
        .map_err(|e| DfxError::InvalidData(format!("invalid version: {}", e)))?;
    println!("Current version: {}", current_version);
    let release_root = args.value_of("release-root").unwrap();
    let latest_version = get_latest_version(release_root)?;

    if latest_version > current_version {
        println!("New version available: {}", latest_version);
    } else {
        println!("Already up to date");
    }

    Ok(())
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
        let manifest: Manifest = serde_json::from_str(&MANIFEST).unwrap();
        let mut tags = HashMap::new();
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
        let latest_version = get_latest_version(&mockito::server_url());
        assert_eq!(latest_version.unwrap(), Version::parse("0.4.1").unwrap());
        let _m = mockito::mock("GET", "/manifest.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("Not a valid JSON object")
            .create();
        let latest_version = get_latest_version(&mockito::server_url());
        assert!(latest_version.is_err());
    }
}
