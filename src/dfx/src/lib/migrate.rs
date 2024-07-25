use crate::lib::identity::wallet::wallet_canister_id;
use crate::lib::operations::canister::install_wallet;
use crate::lib::{environment::Environment, error::DfxResult, root_key::fetch_root_key_if_needed};
use anyhow::{bail, Context, Error};
use candid::{CandidType, Deserialize, Principal};
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::Identity;
use ic_agent::{Agent, Identity as _};
use ic_utils::{
    interfaces::{
        management_canister::builders::{CanisterSettings, InstallMode},
        WalletCanister,
    },
    Argument,
};
use itertools::Itertools;

pub async fn migrate(env: &dyn Environment, network: &NetworkDescriptor, fix: bool) -> DfxResult {
    fetch_root_key_if_needed(env).await?;
    let config = env.get_config_or_anyhow()?;
    let config = config.get_config();
    let agent = env.get_agent();
    let mut mgr = env.new_identity_manager()?;
    let ident = mgr.instantiate_selected_identity(env.get_logger())?;
    let mut did_migrate = false;
    let wallet = if let Some(principal) = wallet_canister_id(network, ident.name())? {
        principal
    } else {
        bail!("No wallet found; nothing to do");
    };
    let wallet = if let Ok(wallet) = WalletCanister::create(agent, wallet).await {
        wallet
    } else {
        let cbor = agent
            .read_state_canister_info(wallet, "controllers")
            .await?;
        let controllers: Vec<Principal> = serde_cbor::from_slice(&cbor)?;
        bail!("This identity isn't a controller of the wallet. You need to be one of these principals to upgrade the wallet: {}", controllers.into_iter().join(", "))
    };
    did_migrate |= migrate_wallet(env, agent, &wallet, fix).await?;
    if let Some(canisters) = &config.canisters {
        let store = env.get_canister_id_store()?;
        for name in canisters.keys() {
            if !config.is_remote_canister(name, &network.name)? {
                if let Some(id) = store.find(name) {
                    did_migrate |= migrate_canister(agent, &wallet, id, name, &ident, fix).await?;
                }
            }
        }
    }
    if did_migrate {
        println!("You can also make all of these changes at once with the `dfx fix` command");
    } else {
        println!("No problems found");
    }
    Ok(())
}

async fn migrate_wallet(
    env: &dyn Environment,
    agent: &Agent,
    wallet: &WalletCanister<'_>,
    fix: bool,
) -> DfxResult<bool> {
    if !wallet.version_supports_u128_cycles() {
        if fix {
            println!("Upgrading wallet... ");
            install_wallet(
                env,
                agent,
                *wallet.canister_id_(),
                InstallMode::Upgrade(None),
            )
            .await?
        } else {
            println!("The wallet is outdated; run `dfx wallet upgrade`");
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn migrate_canister(
    agent: &Agent,
    wallet: &WalletCanister<'_>,
    canister_id: Principal,
    canister_name: &str,
    ident: &Identity,
    fix: bool,
) -> DfxResult<bool> {
    let cbor = agent
        .read_state_canister_info(canister_id, "controllers")
        .await?;
    let mut controllers: Vec<Principal> = serde_cbor::from_slice(&cbor)?;
    if controllers.contains(wallet.canister_id_())
        && !controllers.contains(&ident.sender().unwrap())
    {
        if fix {
            println!(
                "Adding the {ident} identity to canister {canister_name}'s controllers...",
                ident = ident.name()
            );
            controllers.push(ident.sender().map_err(Error::msg)?);
            #[derive(CandidType, Deserialize)]
            struct In {
                canister_id: Principal,
                settings: CanisterSettings,
            }
            wallet
                .call(
                    Principal::management_canister(),
                    "update_settings",
                    Argument::from_candid((In {
                        canister_id,
                        settings: CanisterSettings {
                            controllers: Some(controllers),
                            compute_allocation: None,
                            freezing_threshold: None,
                            memory_allocation: None,
                            reserved_cycles_limit: None,
                            wasm_memory_limit: None,
                            log_visibility: None,
                        },
                    },)),
                    0,
                )
                .await
                .context("Could not update canister settings")?;
        } else {
            println!("Canister {canister_name} is outdated; run `dfx canister update-settings` with the --add-controller flag")
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
