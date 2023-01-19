use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtensionManifest {
    pub name: String,            //"sns", // mandatory
    pub version: String,         //"0.1.0", // mandatory
    pub homepage: String,        //"https://github.com/dfinity/dfx-extensions/sns", // mandatory
    pub authors: Option<String>,         //"DFINITY", // optional
    pub summary: String, //"An extension to facilitate SNS testing and deployment", // mandatory
    pub categories: Vec<String>, // ["sns"],
    pub keywords: Option<Vec<String>>, // ["sns", "deployment"], // optional
    pub description: Option<String>, //"A really long description could go here, though it's a bit of a pain in the butt using JSON. Maybe better to have a README instead?", // optional
    pub commands: JsonValue,     //TODO //{ // mandatory
    //     		"init": {
    //         	"help": "Initialize the SNS canisters" // TODO: could kill this
    //     		// TODO: add args? Positional/flag
    //     	},
    //     	"import": {
    //         	"help": "Import the NNS canisters into your dfx.json",
    //         	"subcommands": { // optional; fictional example here to demonstrate subcommands
    //             	"governance": {
    //                 	"help": "Import the governance canister"
    //             	}
    //         	}
    //     	},
    // },
    pub dependencies: Option<HashMap<String, String>>, // { // optional; TODO: specify version ranges (using semver requirements)
                                               // 	"nns": "0.5.0",
                                               // 	"ckbtc": "0.1.1"
                                               // }

    // pub license: String,
    // pub repository: String,
    // pub files: Vec<String>,
}

impl Display for ExtensionManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
