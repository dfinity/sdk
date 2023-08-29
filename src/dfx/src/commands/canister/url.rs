use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use candid::Principal;
use clap::Parser;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::network::provider::command_line_provider_to_url;

/// Prints the URL of a canister.
#[derive(Parser)]
pub struct CanisterURLOpts {
    /// Specifies the name of the canister.
    canister: String,
}

fn print_local(canister_id: &str, address: &str) -> String {
    return format!("{}/?canisterId={}", address, canister_id);
}

fn print_ic(canister_id: &str) -> String {
    return format!("https://{}.icp0.io/", canister_id);
}

pub fn exec(env: &dyn Environment, opts: CanisterURLOpts) -> DfxResult {
    // Suppress warnings
    std::env::var("DFX_WARNING").unwrap_or_else(|_| "".to_string());
    env.get_config_or_anyhow()?;

    let canister_name = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister_name).or_else(|_| canister_id_store.get(canister_name))?;
    let network_descriptor = env.get_network_descriptor();
    let is_ic = NetworkDescriptor::is_ic(
        network_descriptor.name.as_str(),
        &network_descriptor.providers,
    );

    if is_ic {
        println!("{}", print_ic(&canister_id.to_text()));
    } else {
        let address = command_line_provider_to_url("local").unwrap();
        println!("{}", print_local(&canister_id.to_text(), &address));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn print_local() {
        // Should print the URL of the canister.
        assert_eq!(
            super::print_local("rrkah-fqaaa-aaaaa-aaaaq-cai", "http://127.0.0.1:8000",),
            "http://127.0.0.1:8000/?canisterId=rrkah-fqaaa-aaaaa-aaaaq-cai"
        );
        assert_eq!(
            super::print_local("ryjl3-tyaaa-aaaaa-aaaba-cai", "http://127.0.0.1:4943"),
            "http://127.0.0.1:4943/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
    }
    #[test]
    fn print_ic() {
        // Should print the URL of the canister.
        assert_eq!(
            super::print_ic("rrkah-fqaaa-aaaaa-aaaaq-cai"),
            "https://rrkah-fqaaa-aaaaa-aaaaq-cai.icp0.io/"
        );
        assert_eq!(
            super::print_ic("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
    }
}
