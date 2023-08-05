use crate::error::extension::ExtensionError;
use clap::ArgAction;
use serde::Deserialize;
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
    pub multiple: bool,
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
        if self.multiple {
            arg = arg.num_args(0..);
        }
        Ok(arg
            // let's not enforce any restrictions
            .allow_hyphen_values(true)
            .required(false)
            .action(ArgAction::Append))
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
      "about": "Subcommands for working with configuration.",
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
      "about": "Subcommand for creating an SNS."
    },
    "import": {
      "about": "Subcommand for importing sns API definitions and canister IDs.",
      "args": {
        "network_mapping": {
          "about": "Networks to import canisters ids for.\n  --network-mapping <network name in both places>\n  --network-mapping <network name here>=<network name in project being imported>\nExamples:\n  --network-mapping ic\n  --network-mapping ic=mainnet",
          "long": "network-mapping"
        }
      }
    },
    "download": {
      "about": "Subcommand for downloading SNS WASMs.",
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
      "about": "Subcommand for installing something.",
      "args": {
        "accounts": {
          "about": "some arg that accepts multiple values separated by spaces",
          "long": "accounts",
          "multiple": true
        }
      }
    }
  }
}
"#;

    let m: Result<ExtensionManifest, serde_json::Error> = serde_json::from_str(f);
    dbg!(&m);
    assert!(m.is_ok());

    let subcmds = m.unwrap().into_clap_commands().unwrap();
    dbg!(&subcmds);
    for s in &subcmds {
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
                    "value",
                    "value2",
                ]);
                assert!(matches.is_err());
            }
            "install" => {
                let matches = s.clone().get_matches_from(vec![
                    "install",
                    "--accounts",
                    "value1",
                    "value2",
                    "value3",
                    "value4",
                ]);
                assert_eq!(
                    vec!["value1", "value2", "value3", "value4"],
                    matches
                        .get_many::<String>("accounts")
                        .unwrap()
                        .map(|x| x.as_str())
                        .collect::<Vec<&str>>()
                );
            }
            _ => {}
        }
    }

    let cli = clap::Command::new("sns").subcommands(subcmds);
    cli.debug_assert();
}
