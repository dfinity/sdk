use crate::commands::deploy::MAINNET_CANDID_INTERFACE_PRINCIPAL;
use crate::lib::{error::DfxResult, named_canister::UI_CANISTER};
use crate::Environment;
use anyhow::{bail, Context};
use url::{Host::Domain, Url};

pub(crate) fn get_candid_ui_url(env: &dyn Environment) -> DfxResult<String> {
    let network_descriptor = env.get_network_descriptor();

    if network_descriptor.is_ic {
        Ok(format!(
            "https://{}.raw.icp0.io",
            MAINNET_CANDID_INTERFACE_PRINCIPAL
        ))
    } else {
        let Ok(candid_ui_id) = env.get_canister_id_store()?.get(UI_CANISTER) else {
            bail!(
                "Candid UI not installed on network {}.",
                network_descriptor.name
            )
        };
        let mut url = Url::parse(&network_descriptor.providers[0]).with_context(|| {
            format!(
                "Failed to parse network provider {}.",
                &network_descriptor.providers[0]
            )
        })?;
        if let Some(Domain(domain)) = url.host() {
            let host = format!("{}.{}", candid_ui_id, domain);
            url.set_host(Some(&host))
                .with_context(|| format!("Failed to set host to {}", &host))?;
        } else {
            let query = format!("canisterId={}", candid_ui_id);
            url.set_query(Some(&query));
        }
        Ok(url.to_string())
    }
}
