use crate::json::structure::VersionReqWithJsonSchema;
use candid::Deserialize;
use schemars::JsonSchema;
use semver::Version;
use std::collections::HashMap;
use url::Url;
use crate::error::extension::FetchExtensionDependenciesError;

type ExtensionVersion = Version;
type DependencyName = String;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyRequirement {
    /// A SemVer version requirement, for example ">=0.17.0, <0.19.0".
    Version(VersionReqWithJsonSchema),
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct ExtensionDependencies(
    pub HashMap<ExtensionVersion, HashMap<DependencyName, DependencyRequirement>>,
);

impl ExtensionDependencies {
    pub fn fetch(base_url: &Url) -> Result<Self, FetchExtensionDependenciesError> {
        let url = base_url.join("dependencies.json").map_err(FetchExtensionDependenciesError::ParseUrl)?;
        let resp = reqwest::blocking::get(url).map_err(FetchExtensionDependenciesError::Get)?
            .error_for_status().map_err(FetchExtensionDependenciesError::Get)?;

        resp.json().map_err(FetchExtensionDependenciesError::ParseJson)
    }

    pub fn get_highest_compatible_version(
        &self,
        dfx_version: &Version,
    ) -> Option<Version> {

        let mut keys: Vec<&Version> = self.0.keys().collect();
        keys.sort();
        keys.reverse(); // check higher extension versions first

        for key in keys {
            let dependencies = self.0.get(key).unwrap();
            for (dependency, requirements) in dependencies {
                if dependency == "dfx" {
                    match requirements {
                        DependencyRequirement::Version(req) => {
                            if req.matches(dfx_version) {
                                return Some(key.clone());
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

#[test]
fn parse_test_file() {
    let f = r#"
{
  "0.3.4": {
    "dfx": {
      "version": ">=0.8, <0.9"
    }
  },
  "0.6.2": {
    "dfx": {
      "version": ">=0.9.6"
    }
  },
  "0.7.0": {
    "dfx": {
      "version": ">=0.9.9"
    }
  }
}
"#;
    let m: Result<ExtensionDependencies, serde_json::Error> = dbg!(serde_json::from_str(f));
    assert!(m.is_ok());
    let manifest = m.unwrap();

    let versions = manifest.0.keys().collect::<Vec<_>>();
    assert_eq!(versions.len(), 3);
    assert!(versions.contains(&&Version::new(0, 3, 4)));
    assert!(versions.contains(&&Version::new(0, 6, 2)));
    assert!(versions.contains(&&Version::new(0, 7, 0)));

    let v_3_4 = manifest.0.get(&Version::new(0, 3, 4)).unwrap();
    let dfx = v_3_4.get("dfx").unwrap();
    let DependencyRequirement::Version(req) = dfx;
    assert!(req.matches(&semver::Version::new(0, 8, 5)));
    assert!(!req.matches(&semver::Version::new(0, 9, 0)));

    let v_6_2 = manifest.0.get(&Version::new(0, 6, 2)).unwrap();
    let dfx = v_6_2.get("dfx").unwrap();
    let DependencyRequirement::Version(req) = dfx;
    assert!(req.matches(&semver::Version::new(0, 9, 6)));
    assert!(!req.matches(&semver::Version::new(0, 9, 5)));

    assert_eq!(manifest.get_highest_compatible_version(&Version::new(0, 8, 5)), Some(Version::new(0, 3, 4)));
    assert_eq!(manifest.get_highest_compatible_version(&Version::new(0, 9, 6)), Some(Version::new(0, 6, 2)));
    assert_eq!(manifest.get_highest_compatible_version(&Version::new(0, 9, 10)), Some(Version::new(0, 7, 0)));
}
