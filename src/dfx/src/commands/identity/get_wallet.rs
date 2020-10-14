use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use clap::{App, Arg, ArgMatches, SubCommand};
use tokio::runtime::Runtime;

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
    let agent_env = create_agent_environment(env, args)?;
    let identity = IdentityManager::new(&agent_env)?.instantiate_selected_identity()?;
    let network = get_network_descriptor(&agent_env, args)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        println!(
            "{}",
            identity
                .get_or_create_wallet(&agent_env, &network, true)
                .await?
        );
        DfxResult::Ok(())
    })?;

    Ok(())
}
