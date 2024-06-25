use crate::json::structure::VersionReqWithJsonSchema;
use candid::Deserialize;
use schemars::JsonSchema;
use semver::Version;
use std::collections::HashMap;

type ExtensionVersion = Version;
type DependencyName = String;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyRequirement {
    /// A SemVer version requirement, for example ">=0.17.0, <0.19.0".
    Version(VersionReqWithJsonSchema),
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct DependencyManifest(
    pub HashMap<ExtensionVersion, HashMap<DependencyName, DependencyRequirement>>,
);

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
  }
}
"#;
    let m: Result<DependencyManifest, serde_json::Error> = dbg!(serde_json::from_str(f));
    assert!(m.is_ok());
    let manifest = m.unwrap();

    let versions = manifest.0.keys().collect::<Vec<_>>();
    assert_eq!(versions.len(), 2);
    assert!(versions.contains(&&Version::new(0, 3, 4)));
    assert!(versions.contains(&&Version::new(0, 6, 2)));

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
}
