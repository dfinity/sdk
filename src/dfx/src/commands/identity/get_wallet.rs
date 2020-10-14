use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::get_network_descriptor;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("get-wallet")
        .about(UserMessage::IdentityGetWallet.to_str())
        .arg(
            Arg::with_name("network")
                .help("The network that the wallet exists on.")
                .long("network")
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let network = get_network_descriptor(env, args)?;

    println!("{}", identity.wallet_canister_id(env, &network)?);

    Ok(())
}
