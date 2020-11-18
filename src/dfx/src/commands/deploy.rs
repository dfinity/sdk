use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::deploy_canisters;
use crate::lib::provider::create_agent_environment;
use crate::util::expiry_duration;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use tokio::runtime::Runtime;

/// Deploys all or a specific canister from the code in your project. By default, all canisters are deployed.
#[derive(Clap)]
#[clap(name("deploy"))]
pub struct DeployOpts {
    /// Specifies the name of the canister you want to deploy.
    /// If you don’t specify a canister name, all canisters defined in the dfx.json file are deployed.
    canister_name: Option<String>,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,

    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,
}

pub fn construct() -> App<'static> {
    DeployOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: DeployOpts = DeployOpts::from_arg_matches(args);
    let env = create_agent_environment(env, opts.network)?;

    let timeout = expiry_duration();
    let canister_name = opts.canister_name.as_deref();
    let argument = opts.argument.as_deref();
    let argument_type = opts.argument_type.as_deref();

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(deploy_canisters(
        &env,
        canister_name,
        argument,
        argument_type,
        timeout,
    ))
}
