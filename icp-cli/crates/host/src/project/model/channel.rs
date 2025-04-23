use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Channel {
    Named(NamedChannel),
    Version(Version),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NamedChannel {
    Stable,
    Beta,
}

impl Default for Channel {
    fn default() -> Self {
        Channel::Named(NamedChannel::Stable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn parses_named_channel() {
        let yaml = "channel: beta";
        #[derive(Deserialize)]
        struct ProjectModel {
            channel: Channel,
        }

        let parsed: ProjectModel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.channel, Channel::Named(NamedChannel::Beta));
    }

    #[test]
    fn parses_version_channel() {
        let yaml = "channel: \"1.2.3\"";
        #[derive(Deserialize)]
        struct ProjectModel {
            channel: Channel,
        }

        let parsed: ProjectModel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            parsed.channel,
            Channel::Version(Version::parse("1.2.3").unwrap())
        );
    }

    #[test]
    fn serializes_named_channel() {
        #[derive(Serialize)]
        struct ProjectModel {
            channel: Channel,
        }

        let model = ProjectModel {
            channel: Channel::Named(NamedChannel::Stable),
        };

        let yaml = serde_yaml::to_string(&model).unwrap();
        assert!(yaml.contains("channel: stable"));
    }

    #[test]
    fn serializes_version_channel() {
        #[derive(Serialize)]
        struct ProjectModel {
            channel: Channel,
        }

        let version = Version::parse("2.0.0").unwrap();
        let model = ProjectModel {
            channel: Channel::Version(version),
        };

        let yaml = serde_yaml::to_string(&model).unwrap();
        assert!(yaml.contains("channel: 2.0.0"));
    }
}
