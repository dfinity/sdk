use crate::error::extension::ExtensionError;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::path::PathBuf;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

pub static MANIFEST_FILE_NAME: &str = "extension.json";

type SubcmdName = String;
type ArgName = String;

#[derive(Debug, Deserialize)]
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
    pub dependencies: Option<HashMap<String, String>>,
    pub canister_type: Option<ExtensionCanisterType>,
}

impl ExtensionManifest {
    pub fn load(name: &str, extensions_root_dir: &Path) -> Result<Self, ExtensionError> {
        let manifest_path = Self::manifest_path(name, extensions_root_dir);
        let mut m: ExtensionManifest = crate::json::load_json_file(&manifest_path)
            .map_err(ExtensionError::LoadExtensionManifestFailed)?;
        m.name = name.to_string();
        Ok(m)
    }

    pub fn exists(name: &str, extensions_root_dir: &Path) -> bool {
        Self::manifest_path(name, extensions_root_dir).exists()
    }

    fn manifest_path(name: &str, extensions_root_dir: &Path) -> PathBuf {
        extensions_root_dir.join(name).join(MANIFEST_FILE_NAME)
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, Default)]
pub struct ExtensionSubcommandsOpts(BTreeMap<SubcmdName, ExtensionSubcommandOpts>);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionSubcommandOpts {
    pub about: Option<String>,
    pub args: Option<BTreeMap<ArgName, ExtensionSubcommandArgOpts>>,
    pub subcommands: Option<ExtensionSubcommandsOpts>,
}

#[derive(Debug, Deserialize)]
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

    let mut subcmds = dbg!(m.unwrap().into_clap_commands().unwrap());

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
