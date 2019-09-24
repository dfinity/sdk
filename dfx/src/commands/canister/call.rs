use crate::lib::api_client::{call, Blob};
use crate::lib::env::ClientEnv;
use crate::lib::error::DfxResult;
use crate::lib::CanisterId;
use crate::util::clap::validators;
use clap::{App, Arg, ArgMatches, SubCommand};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about("Call a canister.")
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help("The canister ID (a number) to call.")
                .required(true)
                .validator(validators::is_canister_id),
        )
        .arg(
            Arg::with_name("method_name")
                .help("The method name file to use.")
                .required(true),
        )
        .arg(
            Arg::with_name("arguments")
                .help("Arguments to pass to the method.")
                .takes_value(true)
                .multiple(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    // Read the config.
    let canister_id = args.value_of("canister").unwrap().parse::<CanisterId>()?;
    let method_name = args.value_of("method_name").unwrap();
    let arguments: Option<Vec<&str>> = args.values_of("arguments").map(|args| args.collect());

    let client = env.get_client();
    let install = call(
        client,
        canister_id,
        method_name.to_owned(),
        arguments.map(|args| Blob(Vec::from(args[0]))),
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(install)?;

    Ok(())
}
