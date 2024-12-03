use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use slog::info;

pub fn skip_remote_canister(env: &dyn Environment, canister: &str) -> DfxResult<bool> {
    let config = env.get_config_or_anyhow()?;
    let config_interface = config.get_config();
    let network = env.get_network_descriptor();
    let canister_is_remote = config_interface.is_remote_canister(canister, &network.name)?;
    if canister_is_remote {
        info!(
            env.get_logger(),
            "Skipping canister '{canister}' because it is remote for network '{}'", &network.name,
        );
    }
    Ok(canister_is_remote)
}
