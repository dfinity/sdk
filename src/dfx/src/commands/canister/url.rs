use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::{environment::Environment, named_canister};
use anyhow::Context;
use candid::Principal;
use clap::Parser;
use dfx_core::canister::url::{
    format_frontend_url, format_ui_canister_url_custom, format_ui_canister_url_ic,
};
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use fn_error_context::context;
use url::Url;

/// Prints the URL of a canister.
#[derive(Parser)]
pub struct CanisterURLOpts {
    /// Specifies the name of the canister.
    canister: String,
    #[command(flatten)]
    network: NetworkOpt,
}

#[context("Failed to construct frontend url for canister {} on network '{}'.", canister_id, network.name)]
pub fn construct_frontend_url(
    network: &NetworkDescriptor,
    canister_id: &Principal,
) -> DfxResult<Url> {
    let url = Url::parse(&network.providers[0]).with_context(|| {
        format!(
            "Failed to parse url for network provider {}.",
            &network.providers[0]
        )
    })?;

    Ok(format_frontend_url(&url, &canister_id.to_string()))
}

#[context("Failed to construct ui canister url for {} on network '{}'.", canister_id, network.name)]
pub fn construct_ui_canister_url(
    network: &NetworkDescriptor,
    canister_id: &Principal,
    ui_canister_id: Option<Principal>,
) -> DfxResult<Url> {
    let provider = Url::parse(&network.providers[0]).with_context(|| {
        format!(
            "Failed to parse url for network provider {}.",
            &network.providers[0]
        )
    })?;
    if network.is_ic {
        let formatted_url = format_ui_canister_url_ic(&canister_id.to_string())?;
        return Ok(formatted_url);
    } else {
        if let Some(ui_canister_id) = ui_canister_id {
            let formatted_url = format_ui_canister_url_custom(
                &canister_id.to_string(),
                &provider,
                &ui_canister_id.to_string().as_str(),
            );
            return Ok(formatted_url);
        } else {
            return Err(anyhow::anyhow!(
                "Canister {} does not have a ui canister id",
                canister_id
            ));
        }
    }
}

pub fn exec(env: &dyn Environment, opts: CanisterURLOpts) -> DfxResult {
    env.get_config_or_anyhow()?;
    let network_descriptor = create_network_descriptor(
        env.get_config(),
        env.get_networks_config(),
        opts.network.to_network_name(),
        None,
        LocalBindDetermination::AsConfigured,
    )?;
    let canister_name = opts.canister.as_str();
    let canister_id_store =
        CanisterIdStore::new(env.get_logger(), &network_descriptor, env.get_config())?;
    let canister_id =
        Principal::from_text(canister_name).or_else(|_| canister_id_store.get(canister_name))?;
    let config = env.get_config_or_anyhow()?;
    let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;

    let ui_canister_id = named_canister::get_ui_canister_id(&canister_id_store);
    // If the canister is an assets canister or has a frontend section, we can display a frontend url.
    if let Some(canisters) = &config.get_config().canisters {
        let canister_config = canisters.get(canister_name).unwrap();
        let is_assets = canister_info.is_assets() || canister_config.frontend.is_some();
        if is_assets {
            let url = construct_frontend_url(&network_descriptor, &canister_id)?;
            println!("{}", url.as_str());
            Ok(())
        } else {
            let url = construct_ui_canister_url(&network_descriptor, &canister_id, ui_canister_id)?;
            println!("{}", url.as_str());
            Ok(())
        }
    } else {
        Err(anyhow::anyhow!(
            "Canister {} does not have a frontend section",
            canister_name
        ))
    }
}

#[cfg(test)]
mod test {
    use candid::Principal;
    use dfx_core::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};

    fn test_url(url: &str) -> String {
        let local_network = NetworkDescriptor {
            name: String::from("Test Network"), // Initialize with a String
            providers: vec![String::from(url)], // Initialize with a Vec of Strings
            is_ic: false,
            r#type: NetworkTypeDescriptor::Persistent,
            local_server_descriptor: Option::None,
        };
        // Should print the URL of the canister.
        let canister = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai");
        let mut output = String::new();
        match canister {
            Ok(canister) => {
                let url = super::construct_frontend_url(&local_network, &canister);

                match url {
                    Ok(u) => {
                        output.push_str(u.as_str());
                    }
                    Err(_) => {
                        println!("Error: Failed to construct frontend url for canister");
                    }
                }
            }
            Err(_) => {
                println!("Error: Failed to parse canister id");
            }
        }
        return output;
    }

    #[test]
    fn print_local() {
        assert_eq!(
            test_url("http://127.0.0.1:4943"),
            "http://127.0.0.1:4943/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
    }

    #[test]
    fn print_ic() {
        assert_eq!(
            test_url("https://icp-api.io"),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
        assert_eq!(
            test_url("https://ic0.app"),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
        assert_eq!(
            test_url("https://icp0.io"),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
    }
}
