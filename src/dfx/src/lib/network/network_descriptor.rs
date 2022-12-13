use crate::config::dfinity::{NetworkType, PlaygroundConfig};
use crate::config::dfinity::{DEFAULT_IC_GATEWAY, DEFAULT_IC_GATEWAY_TRAILING_SLASH};
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;

use anyhow::bail;
use candid::Principal;
use fn_error_context::context;
use std::path::{Path, PathBuf};

//"rrkah-fqaaa-aaaaa-aaaaq-cai"
const MAINNET_MOTOKO_PLAYGROUND_CANISTER_ID: Principal = Principal::from_slice(&[
    0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const PLAYGROUND_NETWORK_NAME: &str = "playground";
const MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS: u32 = 1200;

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkTypeDescriptor {
    Ephemeral {
        wallet_config_path: PathBuf,
    },
    Playground {
        playground_cid: Principal,
        canister_timeout_seconds: u32,
    },
    Persistent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub r#type: NetworkTypeDescriptor,
    pub is_ic: bool,
    pub local_server_descriptor: Option<LocalServerDescriptor>,
}

impl NetworkTypeDescriptor {
    #[context("Failed to create NetworkTypeDescriptor.")]
    pub fn new(
        r#type: NetworkType,
        ephemeral_wallet_config_path: &Path,
        playground: Option<PlaygroundConfig>,
    ) -> DfxResult<Self> {
        if let Some(playground_config) = playground {
            Ok(NetworkTypeDescriptor::Playground {
                playground_cid: Principal::from_text(playground_config.playground_cid)?,
                canister_timeout_seconds: playground_config
                    .timeout
                    .unwrap_or(MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS),
            })
        } else {
            match r#type {
                NetworkType::Ephemeral => Ok(NetworkTypeDescriptor::Ephemeral {
                    wallet_config_path: ephemeral_wallet_config_path.to_path_buf(),
                }),
                NetworkType::Persistent => Ok(NetworkTypeDescriptor::Persistent),
            }
        }
    }
}

impl NetworkDescriptor {
    pub fn ic() -> Self {
        NetworkDescriptor {
            name: "ic".to_string(),
            providers: vec![DEFAULT_IC_GATEWAY.to_string()],
            r#type: NetworkTypeDescriptor::Persistent,
            is_ic: true,
            local_server_descriptor: None,
        }
    }

    /// Determines whether the provided connection is the official IC or not.
    #[allow(clippy::ptr_arg)]
    pub fn is_ic(network_name: &str, providers: &Vec<String>) -> bool {
        let name_match = matches!(
            network_name,
            "ic" | DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
        );
        let provider_match = {
            providers.len() == 1
                && matches!(
                    providers.get(0).unwrap().as_str(),
                    DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
                )
        };
        name_match || provider_match
    }

    pub fn is_playground(&self) -> bool {
        matches!(self.r#type, NetworkTypeDescriptor::Playground { .. })
    }

    /// Return the first provider in the list
    pub fn first_provider(&self) -> DfxResult<&str> {
        match self.providers.first() {
            Some(provider) => Ok(provider),
            None => bail!(
                "Network '{}' does not specify any network providers.",
                self.name
            ),
        }
    }

    pub fn local_server_descriptor(&self) -> DfxResult<&LocalServerDescriptor> {
        match &self.local_server_descriptor {
            Some(p) => Ok(p),
            None => bail!("The '{}' network must be a local network", self.name),
        }
    }

    /// Playground on mainnet
    pub(crate) fn default_playground_network() -> Self {
        Self {
            name: PLAYGROUND_NETWORK_NAME.to_string(),
            providers: vec![DEFAULT_IC_GATEWAY.to_string()],
            r#type: NetworkTypeDescriptor::Playground {
                playground_cid: MAINNET_MOTOKO_PLAYGROUND_CANISTER_ID,
                canister_timeout_seconds: MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS,
            },
            is_ic: true,
            local_server_descriptor: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ic_by_netname() {
        assert!(NetworkDescriptor::is_ic("ic", &vec![]));
        assert!(NetworkDescriptor::is_ic(DEFAULT_IC_GATEWAY, &vec![]));
        assert!(NetworkDescriptor::is_ic(
            DEFAULT_IC_GATEWAY_TRAILING_SLASH,
            &vec![]
        ));
    }

    #[test]
    fn ic_by_provider() {
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY.to_string()]
        ));
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY_TRAILING_SLASH.to_string()]
        ));
    }

    #[test]
    fn ic_by_netname_fail() {
        assert!(!NetworkDescriptor::is_ic("not_ic", &vec![]));
    }

    #[test]
    fn ic_by_provider_fail_string() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec!["not_ic_provider".to_string()]
        ));
    }

    #[test]
    fn ic_by_provider_fail_unique() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec![
                DEFAULT_IC_GATEWAY.to_string(),
                "some_other_provider".to_string()
            ]
        ));
    }
}
