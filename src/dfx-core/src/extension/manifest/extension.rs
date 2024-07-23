use crate::error::extension::{
    ConvertExtensionSubcommandIntoClapArgError, ConvertExtensionSubcommandIntoClapCommandError,
    LoadExtensionManifestError,
};
use crate::json::structure::VersionReqWithJsonSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::path::PathBuf;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

pub static MANIFEST_FILE_NAME: &str = "extension.json";
const DEFAULT_DOWNLOAD_URL_TEMPLATE: &str =
    "https://github.com/dfinity/dfx-extensions/releases/download/{{tag}}/{{basename}}.{{archive-format}}";

type SubcmdName = String;
type ArgName = String;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub homepage: String,
    pub authors: Option<String>,
    pub summary: String,
    pub categories: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub subcommands: Option<ExtensionSubcommandsOpts>,
    pub dependencies: Option<HashMap<String, ExtensionDependency>>,
    pub canister_type: Option<ExtensionCanisterType>,

    /// Components of the download url template are:
    /// - `{{tag}}`: the tag of the extension release, which will follow the form "<extension name>-v<extension version>"
    /// - `{{basename}}`: The basename of the release filename, which will follow the form "<extension name>-<arch>-<platform>", for example "nns-x86_64-unknown-linux-gnu"
    /// - `{{archive-format}}`: the format of the archive, for example "tar.gz"
    #[serde(
        default = "default_download_url_template",
        skip_serializing_if = "Option::is_none"
    )]
    pub download_url_template: Option<String>,
}

fn default_download_url_template() -> Option<String> {
    Some(DEFAULT_DOWNLOAD_URL_TEMPLATE.to_string())
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ExtensionDependency {
    /// A SemVer version requirement, for example ">=0.17.0".
    Version(VersionReqWithJsonSchema),
}

impl ExtensionManifest {
    pub fn load(
        name: &str,
        extensions_root_dir: &Path,
    ) -> Result<Self, LoadExtensionManifestError> {
        let manifest_path = Self::manifest_path(name, extensions_root_dir);
        let mut m: ExtensionManifest = crate::json::load_json_file(&manifest_path)?;
        m.name = name.to_string();
        Ok(m)
    }

    pub fn exists(name: &str, extensions_root_dir: &Path) -> bool {
        Self::manifest_path(name, extensions_root_dir).exists()
    }

    fn manifest_path(name: &str, extensions_root_dir: &Path) -> PathBuf {
        extensions_root_dir.join(name).join(MANIFEST_FILE_NAME)
    }

    pub fn download_url_template(&self) -> String {
        self.download_url_template
            .clone()
            .unwrap_or_else(|| DEFAULT_DOWNLOAD_URL_TEMPLATE.to_string())
    }

    pub fn into_clap_commands(
        self,
    ) -> Result<Vec<clap::Command>, ConvertExtensionSubcommandIntoClapCommandError> {
        self.subcommands
            .unwrap_or_default()
            .0
            .into_iter()
            .map(|(subcmd, opts)| opts.into_clap_command(subcmd))
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExtensionCanisterType {
    /// If one field depends on another and both specify a handlebars expression,
    /// list the fields in the order that they should be evaluated.
    #[serde(default)]
    pub evaluation_order: Vec<String>,

    /// Default values for the canister type. These values are used when the user does not provide
    /// values in dfx.json.
    /// The "metadata" field, if present, is appended to the metadata field from dfx.json, which
    /// has the effect of providing defaults.
    /// The "tech_stack field, if present, it merged with the tech_stack field from dfx.json,
    /// which also has the effect of providing defaults.
    #[serde(default)]
    pub defaults: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct ExtensionSubcommandsOpts(pub BTreeMap<SubcmdName, ExtensionSubcommandOpts>);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtensionSubcommandOpts {
    pub about: Option<String>,
    pub args: Option<BTreeMap<ArgName, ExtensionSubcommandArgOpts>>,
    pub subcommands: Option<ExtensionSubcommandsOpts>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtensionSubcommandArgOpts {
    pub about: Option<String>,
    pub long: Option<String>,
    pub short: Option<char>,
    #[serde(default)]
    #[deprecated(note = "use `values` instead")]
    pub multiple: bool,
    #[serde(default)]
    pub values: ArgNumberOfValues,
}

#[derive(Debug, JsonSchema, Eq, PartialEq)]
pub enum ArgNumberOfValues {
    /// zero or more values
    Number(usize),
    /// non-inclusive range
    Range(std::ops::Range<usize>),
    /// unlimited values
    Unlimited,
}

impl Default for ArgNumberOfValues {
    fn default() -> Self {
        Self::Number(1)
    }
}

impl<'de> Deserialize<'de> for ArgNumberOfValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StrOrUsize<'a> {
            Str(&'a str),
            Usize(usize),
        }

        match StrOrUsize::deserialize(deserializer)? {
            StrOrUsize::Usize(n) => Ok(Self::Number(n)),
            StrOrUsize::Str(s) => {
                if s == "unlimited" {
                    return Ok(Self::Unlimited);
                }
                if s.contains("..=") {
                    let msg = format!("Inclusive ranges are not supported: {}", s);
                    return Err(serde::de::Error::custom(msg));
                }
                if s.contains("..") {
                    let parts: Vec<&str> = s.split("..").collect();
                    if let (Ok(start), Ok(end)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                    {
                        return Ok(Self::Range(start..end + 1));
                    }
                }
                Err(serde::de::Error::custom(format!(
            "Invalid format for values: '{}'. Expected 'unlimited' or a positive integer or a range (for example '1..3')",
            s
        )))
            }
        }
    }
}

impl Serialize for ArgNumberOfValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Number(n) => serializer.serialize_u64(*n as u64),
            Self::Unlimited => serializer.serialize_str("unlimited"),
            Self::Range(range) => {
                let s = format!("{}..{}", range.start, range.end - 1);
                serializer.serialize_str(&s)
            }
        }
    }
}

impl ExtensionSubcommandArgOpts {
    pub fn into_clap_arg(
        self,
        name: String,
    ) -> Result<clap::Arg, ConvertExtensionSubcommandIntoClapArgError> {
        let mut arg = clap::Arg::new(name.clone());
        if let Some(about) = self.about {
            arg = arg.help(about);
        } else {
            return Err(ConvertExtensionSubcommandIntoClapArgError::ExtensionSubcommandArgMissingDescription(
                name,
            ));
        }
        if let Some(l) = self.long {
            arg = arg.long(l);
        }
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        #[allow(deprecated)]
        if self.multiple {
            arg = arg.num_args(0..);
        } else {
            arg = match self.values {
                ArgNumberOfValues::Number(n) => arg.num_args(n),
                ArgNumberOfValues::Range(r) => arg.num_args(r),
                ArgNumberOfValues::Unlimited => arg.num_args(0..),
            };
        }
        Ok(arg
            // let's allow values that start with a hyphen for args (for example, --calculator -2+2)
            .allow_hyphen_values(true)
            // don't enforce that args are required
            .required(false))
    }
}

impl ExtensionSubcommandOpts {
    pub fn into_clap_command(
        self,
        name: String,
    ) -> Result<clap::Command, ConvertExtensionSubcommandIntoClapCommandError> {
        let mut cmd = clap::Command::new(name);

        if let Some(about) = self.about {
            cmd = cmd.about(about);
        }

        if let Some(args) = self.args {
            for (name, opts) in args {
                cmd = cmd.arg(opts.into_clap_arg(name)?);
            }
        }

        if let Some(subcommands) = self.subcommands {
            for (name, subcommand) in subcommands.0 {
                cmd = cmd.subcommand(subcommand.into_clap_command(name)?);
            }
        }

        Ok(cmd)
    }
}

#[test]
fn parse_test_file() {
    let f = r#"
{
  "name": "sns",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Toolkit for simulating decentralizing a dapp via SNS.",
  "categories": [
    "sns",
    "nns"
  ],
  "dependencies": {
    "dfx": ">=0.8, <0.9"
  },
  "keywords": [
    "sns",
    "nns",
    "deployment"
  ],
  "subcommands": {
    "config": {
      "about": "About for config command. You're looking at the output of parsing test extension.json.",
      "subcommands": {
        "create": {
          "about": "Command line options for creating an SNS configuration."
        },
        "validate": {
          "about": "Command line options for validating an SNS configuration."
        }
      }
    },
    "deploy": {
      "about": "About for deploy command. You're looking at the output of parsing test extension.json."
    },
    "import": {
      "about": "About for import command. You're looking at the output of parsing test extension.json.",
      "args": {
        "network_mapping": {
          "about": "Networks to import canisters ids for.\n  --network-mapping <network name in both places>\n  --network-mapping <network name here>=<network name in project being imported>\nExamples:\n  --network-mapping ic\n  --network-mapping ic=mainnet",
          "long": "network-mapping"
        }
      }
    },
    "download": {
      "about": "About for download command. You're looking at the output of parsing test extension.json.",
      "args": {
        "ic_commit": {
          "about": "IC commit of SNS canister Wasm binaries to download",
          "long": "ic-commit"
        },
        "wasms_dir": {
          "about": "Path to store downloaded SNS canister Wasm binaries",
          "long": "wasms-dir"
        }
      }
    },
    "install": {
      "about": "About for install command. You're looking at the output of parsing test extension.json.",
      "args": {
        "account": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "account"
        },
        "accounts": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "accounts",
          "multiple": true
        },
        "two-accounts": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "two-accounts",
          "values": 2
        },
        "two-or-three-accounts": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "two-or-three-accounts",
          "values": "2..3"
        }
      }
    },
    "init-canister": {
      "about": "About for init-canister command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_id": {
          "about": "some arg that accepts multiple values separated by spaces"
        }
      }
    },
    "init-canisters": {
      "about": "About for init-canisters command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_ids": {
          "about": "some arg that accepts multiple values separated by spaces",
          "values": "unlimited"
        }
      }
    },
    "init-two-canisters": {
      "about": "About for init-two-canisters command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_ids": {
          "about": "some arg that accepts multiple values separated by spaces",
          "values": 2
        }
      }
    },
    "init-two-or-three-canisters": {
      "about": "About for init-two-or-three-canisters command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_ids": {
          "about": "some arg that accepts multiple values separated by spaces",
          "values": "2..3"
        }
      }
    }
  }
}
"#;
    macro_rules! test_cmd {
        ($cmd:expr, [$($cmds:expr),*], $arg_name:expr => [$($expected:expr),*]) => {{
            let commands = vec![$($cmds),*];
            let expected_values: Vec<&str> = vec![$($expected),*];
            let matches = $cmd.clone().get_matches_from(commands);
            let output = matches
                .get_many::<String>(&$arg_name)
                .unwrap()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>();
            assert_eq!(expected_values, output, "Arg: {}", $arg_name);
        }};
        ($cmd:expr, [$($cmds:expr),*], $err_kind:expr) => {{
            let commands = vec![$($cmds),*];
            let matches = dbg!($cmd.clone().try_get_matches_from(commands));
            assert_eq!(matches.as_ref().map_err(|e| e.kind()), Err($err_kind));
        }};
    }

    let m: Result<ExtensionManifest, serde_json::Error> = dbg!(serde_json::from_str(f));
    assert!(m.is_ok());
    let manifest = m.unwrap();

    let dependencies = manifest.dependencies.as_ref().unwrap();
    let dfx_dep = dependencies.get("dfx").unwrap();
    let ExtensionDependency::Version(req) = dfx_dep;
    assert!(req.matches(&semver::Version::new(0, 8, 5)));
    assert!(!req.matches(&semver::Version::new(0, 9, 0)));

    let mut subcmds = dbg!(manifest.into_clap_commands().unwrap());

    use clap::error::ErrorKind::*;
    for c in &mut subcmds {
        c.print_long_help().unwrap();
        match c.get_name() {
            subcmd @ "download" => {
                test_cmd!(c, [subcmd, "--ic-commit", "C"], "ic_commit" => ["C"]);
                test_cmd!(c, [subcmd, "--ic-commit", "c1", "c2"], UnknownArgument);
                test_cmd!(c, [subcmd, "--dosent-extist", "c1", "c2"], UnknownArgument);
            }
            #[rustfmt::skip]
            subcmd @ "install" => {
                test_cmd!(c, [subcmd, "--account", "A"], "account" => ["A"]);
                test_cmd!(c, [subcmd, "--account", "A", "B"], UnknownArgument);
                test_cmd!(c, [subcmd, "--accounts"], "accounts" => []);
                test_cmd!(c, [subcmd, "--accounts", "A", "B"], "accounts" => ["A", "B"]);
                test_cmd!(c, [subcmd, "--two-accounts", "A"], WrongNumberOfValues);
                test_cmd!(c, [subcmd, "--two-accounts", "A", "B"], "two-accounts" => ["A", "B"]);
                test_cmd!(c, [subcmd, "--two-accounts", "A", "B", "C"], UnknownArgument);
                test_cmd!(c, [subcmd, "--two-or-three-accounts", "A"], TooFewValues);
                test_cmd!(c, [subcmd, "--two-or-three-accounts", "A", "B"], "two-or-three-accounts" => ["A", "B"]);
                test_cmd!(c, [subcmd, "--two-or-three-accounts", "A", "B", "C"], "two-or-three-accounts" => ["A", "B", "C"]);
                test_cmd!(c, [subcmd, "--two-or-three-accounts", "A", "B", "C", "D"], UnknownArgument);
            }
            subcmd @ "init-canister" => {
                test_cmd!(c, [subcmd, "x1"], "canister_id" => ["x1"]);
                test_cmd!(c, [subcmd, "x1", "x2"], UnknownArgument);
            }
            subcmd @ "init-canisters" => {
                test_cmd!(c, [subcmd, "y1", "y2", "y3", "y4", "y5"], "canister_ids" => ["y1", "y2", "y3", "y4", "y5"]);
            }
            subcmd @ "init-two-canisters" => {
                test_cmd!(c, [subcmd, "z1"], WrongNumberOfValues);
                test_cmd!(c, [subcmd, "z1", "z2"], "canister_ids" => ["z1", "z2"]);
            }
            subcmd @ "init-two-or-three-canisters" => {
                test_cmd!(c, [subcmd, "1"], TooFewValues);
                test_cmd!(c, [subcmd, "1", "2"], "canister_ids" => ["1", "2"]);
                test_cmd!(c, [subcmd, "1", "2", "3"], "canister_ids" => ["1", "2", "3"]);
                test_cmd!(c, [subcmd, "1", "2", "3", "4"], TooManyValues);
            }
            _ => {}
        }
    }
    clap::Command::new("sns")
        .subcommands(&subcmds)
        .print_help()
        .unwrap();
    clap::Command::new("sns")
        .subcommands(&subcmds)
        .debug_assert();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_arg_number_of_values_number_serialization_deserialization() {
        let original = ArgNumberOfValues::Number(5);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ArgNumberOfValues = serde_json::from_str(&serialized).unwrap();

        assert_eq!(serialized, "5");
        assert_eq!(deserialized, ArgNumberOfValues::Number(5));
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_arg_number_of_values_unlimited_serialization_deserialization() {
        let original = ArgNumberOfValues::Unlimited;
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ArgNumberOfValues = serde_json::from_str(&serialized).unwrap();

        assert_eq!(serialized, "\"unlimited\"");
        assert_eq!(deserialized, ArgNumberOfValues::Unlimited);
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_arg_number_of_values_range_serialization_deserialization() {
        let original = ArgNumberOfValues::Range(1..4);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ArgNumberOfValues = serde_json::from_str(&serialized).unwrap();

        assert_eq!(serialized, "\"1..3\"");
        assert_eq!(deserialized, ArgNumberOfValues::Range(1_usize..4_usize));
        assert_eq!(original, deserialized);
    }
}
