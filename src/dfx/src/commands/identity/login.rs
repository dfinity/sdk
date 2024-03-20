use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Input};
use ic_agent::identity::Delegation;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("An error occurred while managing identities: {0}")]
    IdentityManagerError(String),
    #[error("An error occurred while parsing the identity: {0}")]
    IdentityError(String),
}

/// Lists existing identities.
#[derive(Parser)]
pub struct LoginOpts {}

pub fn exec(env: &dyn Environment, _opts: LoginOpts) -> DfxResult {
    let mgr = env.new_identity_manager()?;
    let identities = mgr.get_identity_names(env.get_logger())?;
    let current_identity = mgr.get_selected_identity_name();
    let current_identity_index = identities
        .iter()
        .position(|i| i == current_identity.as_str())
        .unwrap_or(0);

    let base_identity_index = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the identity to use as a base key for the delegation")
        .default(current_identity_index)
        .items(&identities)
        .interact_opt()?;

    let base_identity = base_identity_index.map(|i| identities[i].to_string());

    if let Some(base_identity) = base_identity {
        println!("Using identity: {}", base_identity);
    }

    let delegation_json = Input::<String>::new()
        .with_prompt("Enter the JSON-encoded delegation chain")
        .interact()?;

    let delegation = serde_json::from_str(&delegation_json)
        .map_err(|err| LoginError::IdentityError(err.to_string()))?;

    Ok(())
}

//  //   baseIdentity: "default",
//   delegations: [
//     {
//       delegation: {
//         expiration: '1655f29d787c0000',
//         pubkey: '302a...',
//         targets: [ '00000000002000030101' ]
//       },
//       signature: 'ba46e...'
//     }
//   ],



#[derive(Parser, Serialize, Deserialize, Debug)]
struct IdentityDelegation {
    base_identity: String,
    delegations: Vec<Delegation>,
}

impl FromStr for IdentityDelegation {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        Ok(Delegation {
            expiration: parts[0].to_string(),
            // hex decode
            pubkey: hex::decode(parts[1]).unwrap(),
            targets: parts[2].split(';').map(|s| s.to_string()).collect(),
            signature: parts[3].to_string(),
        })
    }
}
