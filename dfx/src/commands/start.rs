use crate::lib::env::BinaryResolverEnv;
use crate::lib::error::DfxResult;
use clap::ArgMatches;

/// Find the binary path for the client, then start the node manager.
pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryResolverEnv,
{
    println!("Starting up the DFINITY node manager...");

    let client_pathbuf = env.get_binary_command_path("client")?;
    let client = client_pathbuf.as_path();

    let mut cmd = env.get_binary_command("nodemanager")?;
    cmd.args(&[client]);

    let _child = cmd.spawn()?;

    println!("DFINITY node manager started...");

    Ok(())
}
