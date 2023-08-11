use crate::error::extension::ExtensionError;
use clap::ArgAction;
use serde::{Deserialize, Deserializer};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

pub static MANIFEST_FILE_NAME: &str = "extension.json";

type SubcmdName = String;
type ArgName = String;

#[derive(Debug, Deserialize)]
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
    pub dependencies: Option<HashMap<String, String>>,
}

impl ExtensionManifest {
    pub fn new(name: &str, extensions_root_dir: &Path) -> Result<Self, ExtensionError> {
        let manifest_path = extensions_root_dir.join(name).join(MANIFEST_FILE_NAME);
        let mut m: ExtensionManifest = crate::json::load_json_file(&manifest_path)
            .map_err(ExtensionError::LoadExtensionManifestFailed)?;
        m.name = name.to_string();
        Ok(m)
    }

    pub fn into_clap_commands(self) -> Result<Vec<clap::Command>, ExtensionError> {
        self.subcommands
            .unwrap_or_default()
            .0
            .into_iter()
            .map(|(subcmd, opts)| opts.into_clap_command(subcmd))
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ExtensionSubcommandsOpts(BTreeMap<SubcmdName, ExtensionSubcommandOpts>);

#[derive(Debug, Deserialize)]
pub struct ExtensionSubcommandOpts {
    pub about: Option<String>,
    pub args: Option<BTreeMap<ArgName, ExtensionSubcommandArgOpts>>,
    pub subcommands: Option<ExtensionSubcommandsOpts>,
}

#[derive(Debug, Deserialize)]
pub struct ExtensionSubcommandArgOpts {
    pub about: Option<String>,
    pub long: Option<String>,
    pub short: Option<char>,
    #[serde(default)]
    pub values: ArgNumberOfValues,
}

#[derive(Debug)]
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
            StrOrUsize::Usize(n) => return Ok(Self::Number(n)),
            StrOrUsize::Str(s) => {
                dbg!(&s);
                if s == "unlimited" {
                    return dbg!(Ok(Self::Unlimited));
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
                        return dbg!(Ok(Self::Range(start..end + 1)));
                    }
                }
                return Err(serde::de::Error::custom(format!(
            "Invalid format for values: '{}'. Expected 'unlimited' or a positive integer or a range (for example '1..3')",
            s
        )));
            }
        }
    }
}

impl ExtensionSubcommandArgOpts {
    pub fn into_clap_arg(self, name: String) -> Result<clap::Arg, ExtensionError> {
        let mut arg = clap::Arg::new(name.clone());
        if let Some(about) = self.about {
            arg = arg.help(about);
        } else {
            return Err(ExtensionError::ExtensionSubcommandArgMissingDescription(
                name,
            ));
        }
        if let Some(l) = self.long {
            arg = arg.long(l);
        }
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        arg = dbg!(match dbg!(self.values) {
            ArgNumberOfValues::Number(n) => arg.num_args(n).action(ArgAction::Set),
            ArgNumberOfValues::Range(r) => arg.num_args(dbg!(r)),
            ArgNumberOfValues::Unlimited => arg.num_args(0..).action(ArgAction::Append),
        });
        Ok(arg
            // let's not enforce restrictions
            .allow_hyphen_values(true)
            .required(false))
    }
}

impl ExtensionSubcommandOpts {
    pub fn into_clap_command(self, name: String) -> Result<clap::Command, ExtensionError> {
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
          "about": "IC commit of SNS canister WASMs to download",
          "long": "ic-commit"
        },
        "wasms_dir": {
          "about": "Path to store downloaded SNS canister WASMs",
          "long": "wasms-dir"
        }
      }
    },
    "install": {
      "about": "About for install command. You're looking at the output of parsing test extension.json.",
      "args": {
        "accounts": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "accounts",
          "values": "unlimited"
        }
      }
    },
    "initialize-canister": {
      "about": "About for initialize-canister command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_id": {
          "about": "some arg that accepts multiple values separated by spaces"
        }
      }
    },
    "initialize-canisters": {
      "about": "About for initialize-canisters command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_ids": {
          "about": "some arg that accepts multiple values separated by spaces",
          "values": "unlimited"
        }
      }
    },
    "initialize-two-canisters": {
      "about": "About for initialize-two-canisters command. You're looking at the output of parsing test extension.json.",
      "args": {
        "canister_ids": {
          "about": "some arg that accepts multiple values separated by spaces",
          "values": 2
        }
      }
    },
    "initialize-two-or-three-canisters": {
      "about": "About for initialize-two-or-three-canisters command. You're looking at the output of parsing test extension.json.",
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

    use clap::error::ErrorKind;
    fn get_many<'a>(matches: &'a clap::ArgMatches, name: &'a str) -> Vec<&'a str> {
        matches
            .get_many::<String>(name)
            .unwrap()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
    }
    macro_rules! assert_err_kind {
        ($matches:expr, $kind:expr) => {
            assert_eq!($matches.as_ref().map_err(|e| e.kind()), Err($kind))
        };
    }

    let m: Result<ExtensionManifest, serde_json::Error> = dbg!(serde_json::from_str(f));
    assert!(m.is_ok());

    let mut subcmds = dbg!(m.unwrap().into_clap_commands().unwrap());

    for s in &mut subcmds {
        s.print_long_help().unwrap();
        match s.get_name() {
            "download" => {
                let matches = s
                    .clone()
                    .get_matches_from(vec!["download", "--ic-commit", "value"]);
                assert_eq!(
                    Some(&"value".to_string()),
                    matches.get_one::<String>("ic_commit")
                );
                let matches = s.clone().try_get_matches_from(vec![
                    "download",
                    "--ic-commit",
                    "commit1",
                    "commit2",
                ]);
                assert!(matches.is_err());
            }
            "install" => {
                let matches = s.clone().get_matches_from(vec![
                    "install",
                    "--accounts",
                    "accountA",
                    "accountB",
                    "accountC",
                    "accountD",
                ]);
                assert_eq!(
                    vec!["accountA", "accountB", "accountC", "accountD"],
                    get_many(&matches, "accounts")
                );
            }
            "initialize-canister" => {
                let matches = s
                    .clone()
                    .get_matches_from(vec!["initialize-canister", "one-canister"]);
                assert_eq!(vec!["one-canister"], get_many(&matches, "canister_id"));
                let matches = s.clone().try_get_matches_from(vec![
                    "initialize-canister",
                    "not-one-canister1",
                    "not-one-canister2",
                ]);
                assert_err_kind!(matches, ErrorKind::UnknownArgument);
            }
            "initialize-canisters" => {
                let matches = s.clone().get_matches_from(vec![
                    "initialize-canisters",
                    "can1",
                    "can2",
                    "can3",
                    "can4",
                    "can5",
                ]);
                assert_eq!(
                    vec!["can1", "can2", "can3", "can4", "can5"],
                    get_many(&matches, "canister_ids")
                );
            }
            "initialize-two-canisters" => {
                let matches =
                    s.clone()
                        .get_matches_from(vec!["initialize-canisters", "toucan1", "toucan2"]);
                assert_eq!(
                    vec!["toucan1", "toucan2"],
                    get_many(&matches, "canister_ids")
                );
                let matches = s
                    .clone()
                    .try_get_matches_from(vec!["initialize-canister", "not-toucan"]);
                assert_err_kind!(matches, ErrorKind::WrongNumberOfValues);
            }
            "initialize-two-or-three-canisters" => {
                let matches = s.clone().get_matches_from(vec![
                    "initialize-canisters",
                    "2or3can1",
                    "2or3can2",
                ]);
                assert_eq!(
                    vec!["2or3can1", "2or3can2"],
                    get_many(&matches, "canister_ids")
                );
                let matches = s.clone().get_matches_from(vec![
                    "initialize-canisters",
                    "2or3can1",
                    "2or3can2",
                    "2or3can3",
                ]);
                assert_eq!(
                    vec!["2or3can1", "2or3can2", "2or3can3"],
                    get_many(&matches, "canister_ids")
                );
                let matches = s
                    .clone()
                    .try_get_matches_from(vec!["initialize-canisters", "2or3can"]);
                assert_err_kind!(matches, ErrorKind::TooFewValues);
                let matches = s.clone().try_get_matches_from(vec![
                    "initialize-canisters",
                    "not2or3can1",
                    "not2or3can2",
                    "not2or3can3",
                    "not2or3can4",
                ]);
                assert_err_kind!(matches, ErrorKind::TooManyValues);
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
