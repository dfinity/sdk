use crate::config::directories::get_user_dfx_config_dir;
use crate::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use crate::error::wallet_config::WalletConfigError;
use crate::error::wallet_config::WalletConfigError::GetWalletConfigPathFailed;
use crate::identity::{Identity, WALLET_CONFIG_FILENAME};
use candid::Principal;
use std::path::PathBuf;

pub fn get_wallet_config_path(
    network: &NetworkDescriptor,
    name: &str,
) -> Result<PathBuf, WalletConfigError> {
    Ok(match &network.r#type {
        NetworkTypeDescriptor::Persistent | NetworkTypeDescriptor::Playground { .. } => {
            // Using the global
            get_user_dfx_config_dir()
                .map_err(|e| {
                    GetWalletConfigPathFailed(
                        Box::new(name.to_string()),
                        Box::new(network.name.clone()),
                        e,
                    )
                })?
                .join("identity")
                .join(name)
                .join(WALLET_CONFIG_FILENAME)
        }
        NetworkTypeDescriptor::Ephemeral { wallet_config_path } => wallet_config_path.clone(),
    })
}

pub fn wallet_canister_id(
    network: &NetworkDescriptor,
    name: &str,
) -> Result<Option<Principal>, WalletConfigError> {
    let wallet_path = get_wallet_config_path(network, name)?;
    if !wallet_path.exists() {
        return Ok(None);
    }

    let config = Identity::load_wallet_config(&wallet_path)?;

    let maybe_wallet_principal = config
        .identities
        .get(name)
        .and_then(|wallet_network| wallet_network.networks.get(&network.name).cloned());
    Ok(maybe_wallet_principal)
}
