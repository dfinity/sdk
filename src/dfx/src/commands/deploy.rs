use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::deploy_canisters;
use crate::lib::provider::create_agent_environment;
use crate::util::expiry_duration;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

/// Deploys all or a specific canister from the code in your project. By default, all canisters are deployed.
#[derive(Clap)]
pub struct DeployOpts {
    /// Specifies the name of the canister you want to deploy.
    /// If you don’t specify a canister name, all canisters defined in the dfx.json file are deployed.
    #[clap(long)]
    canister_name: Option<String>,

    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,
}

pub fn construct() -> App<'static> {
    DeployOpts::into_app().name("rename")
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: DeployOpts = DeployOpts::from_arg_matches(args);
    let env = create_agent_environment(env, args)?;

    let timeout = expiry_duration();
    let canister = opts.canister_name.and_then(|v| Some(v.as_str()));

    let argument = opts.argument.and_then(|v| Some(v.as_str()));
    let argument_type = opts.argument_type.and_then(|v| Some(v.as_str()));

    deploy_canisters(&env, canister, argument, argument_type, timeout)
}
