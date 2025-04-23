use crate::project::model::channel::{Channel, NamedChannel};
use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProjectModel {
    pub canisters: Vec<String>,

    #[serde(default)]
    pub channel: Channel,
}

#[test]
fn test_channel_deserialization() {
    use serde_yaml;

    let yaml_stable = r#"
canisters:
  - a
  - b
"#;

    let model: ProjectModel = serde_yaml::from_str(yaml_stable).unwrap();
    assert_eq!(model.channel, Channel::Named(NamedChannel::Stable));

    let yaml_beta = r#"
canisters:
  - x
  - y
channel: beta
"#;
    let model: ProjectModel = serde_yaml::from_str(yaml_beta).unwrap();
    assert_eq!(model.channel, Channel::Named(NamedChannel::Beta));

    let yaml_version = r#"
canisters:
  - z
channel: "1.2.3"
"#;
    let model: ProjectModel = serde_yaml::from_str(yaml_version).unwrap();
    assert_eq!(
        model.channel,
        Channel::Version(Version::parse("1.2.3").unwrap())
    );
}
