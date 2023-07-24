use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;

/// Get wallet name.
#[derive(Parser)]
pub struct NameOpts {}

pub async fn exec(env: &dyn Environment, _opts: NameOpts) -> DfxResult {
    let (maybe_name,): (Option<String>,) = wallet_query(env, "name", ()).await?;
    match maybe_name {
        Some(name) => println!("{}", name),
        None => println!(
            "Name hasn't been set. Call `dfx wallet set-name` to give this cycles wallet a name."
        ),
    };
    Ok(())
}
