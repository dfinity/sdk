use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::NetworkOpt;

use anyhow::{anyhow, Context};
use clap::Parser;
use tokio::runtime::Runtime;

/// Pings an Internet Computer network and returns its status.
#[derive(Parser)]
pub struct PullOpts {
    /// Specifies the name of the canister you want to pull.
    /// If you don’t specify a canister name, all pull type canisters defined in the dfx.json file are pulled.
    canister_name: Option<String>,

    #[clap(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: PullOpts) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        fetch_root_key_if_needed(env).await?;

        let agent = env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        let callee_canister = match opts.canister_name {
            Some(s) => s,
            None => unimplemented!(),
        };

        let canister_id_store = CanisterIdStore::for_env(env)?;

        let canister_id = canister_id_store.get(&callee_canister)?;

        agent
            .read_state_canister_metadata(canister_id, "dfx:deps", false)
            .await
            .with_context(|| {
                format!(
                    "Failed to read `dfx:deps` metadata of canister {}.",
                    canister_id
                )
            })?;
        Ok(())
    })
}
