use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use dfx_core::identity::identity_manager::DelegatedIdentityConfiguration;
use dfx_core::identity::IdentityCreationParameters;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, Input};
use ic_agent::identity::{Delegation, SignedDelegation};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
    // #[error("An error occurred while managing identities: {0}")]
    // IdentityManagerError(String),
    #[error("An error occurred while parsing the identity: {0}")]
    IdentityError(String),
}

/// Lists existing identities.
#[derive(Parser)]
pub struct LoginOpts {}

pub fn exec(env: &dyn Environment, _opts: LoginOpts) -> DfxResult {
    let log = env.get_logger();
    let mgr = env.new_identity_manager()?;
    let identities = mgr.get_identity_names(log)?;
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

    let base_identity_name = base_identity_index.map(|i| identities[i].to_string());

    // if no base identity is selected reject with error
    if base_identity_name.is_none() {
        return Err(LoginError::IdentityError("No base identity selected".to_string()).into());
    }

    let base_identity_configuration =
        mgr.get_identity_config_or_default(base_identity_name.unwrap().as_str())?;

    let delegation_json = Input::<String>::new()
        .with_prompt("Enter the JSON-encoded delegation chain")
        .interact()?;

    let json = serde_json::from_str::<JSONDelegationChain>(&delegation_json)
        .map_err(|err| LoginError::IdentityError(err.to_string()))?;

    let id = json.to_identity_delegation()?;

    mgr.remove(log, "delegated_identity", false, Option::None)?;

    // use parameters from base identity

    let params: DelegatedIdentityConfiguration = DelegatedIdentityConfiguration {
        from_public_key: id.delegations[0].delegation.pubkey.clone(),
        signing_identity: base_identity_configuration.into(),
        chain: id.delegations,
    };

    mgr.create_new_identity(
        log,
        "delegated_identity",
        IdentityCreationParameters::Delegated(params),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::identity::login::JSONDelegationChain;

    #[test]
    fn test_parse_identity_delegation() {
        let s = r#"{
        "delegations":[{
            "delegation": {
                    "expiration":"17c106823a0b91b8","pubkey":"3059301306072a8648ce3d020106082a8648ce3d030107034200046bba01230defcec1c4ecc15d57d81410acfd8000fdec74b72fc09849fb2a6a26d309f8eadb06fd8a51f23408ac9563b81d0ef6568127fb239cfb0790a9c20c68"
            },
            "signature":"d9d9f7a26b63657274696669636174655901d7d9d9f7a2647472656583018301830183024863616e697374657283018301830183024a000000000000000701018301830183024e6365727469666965645f64617461820358206e3432e09b691812eb8e34c73535666e2ab2a2fa8acba5e468f269c1963574fc82045820eeebdca966b95b97cecdf624e005758a11b5d031b574818565faaa89fc5a258b820458205d905b85682bddd934e155c02a115b5ce2efe0aacb2b56c181a9f6b27c62194182045820af7540e4390f6a39c132b0c0044b0f2c3b5acf74e08b80029e575a7c38fd88a682045820fc22fa3e1a4a8ca24dc3cb14e9eed256b3881abb3cd0542e58dce0bafe7dcd6e8204582087e17fd396c410957630bf1a1f0afbfd29e3713361db1f5938508b4a7812c0cc820458200a5a0c8dc5ca2755078b3ab0d7a7b18fde686f4ce8af86625e4c780b266b383882045820fc291a2424a75f805247f7981ebc6bea10b0e52ce1f09f522903aa083eba6982830182045820ce754d5cf162883acf90ed966f98487c35f9abec8c4df7b1760adea872d1b00583024474696d65820349b8d9a3fddcfdade017697369676e61747572655830832f346060e5940b061e93491cb98eb5f5434eb912b39fddf8b4e4162d0f6f75260768c0156c227e9fdd03e239fbc53b64747265658301820458208581a09c85459df0ba918296790d18fddbf8df6143499b218add0da08876b3cd830243736967830258201cd61b12d26b0d78181d86049fef261832d6a790b9c2ca09b69d8318c3a04ec58302582079a54b5de0621b02e88214e9631a8e189e3a3d8a6909cd786286a56638da534a820340"
        }],
        "publicKey":"303c300c060a2b0601040183b8430102032c000a0000000000000007010103b7e1806206cc258fab4f8ac57ede1a4946026308cf92f28f4597243351e776"}"#;

        let chain: Result<JSONDelegationChain, serde_json::Error> = serde_json::from_str(s);

        let id = chain.unwrap().to_identity_delegation().unwrap();
        println!("{:?}", id);
    }
}
