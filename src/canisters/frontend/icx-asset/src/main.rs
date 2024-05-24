mod commands;
mod support;
use crate::commands::list::list;
use crate::commands::sync::sync;
use crate::commands::upload::upload;
use anstyle::{AnsiColor, Style};
use candid::Principal;
use clap::builder::Styles;
use clap::{crate_authors, crate_version, Parser};
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::{agent, Agent, Identity};
use std::path::PathBuf;

const DEFAULT_IC_GATEWAY: &str = "https://icp0.io";

#[derive(Parser)]
#[command(
    version = crate_version!(),
    author = crate_authors!(),
    propagate_version = true,
    styles = style(),
)]
struct Opts {
    /// Some input. Because this isn't an Option<T> it's required to be used
    #[arg(long, default_value = "http://localhost:4943/")]
    replica: String,

    /// An optional PEM file to read the identity from. If none is passed,
    /// a random identity will be created.
    #[arg(long)]
    pem: Option<PathBuf>,

    /// An optional field to set the expiry time on requests. Can be a human
    /// readable time (like `100s`) or a number of seconds.
    #[arg(long)]
    ttl: Option<humantime::Duration>,

    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// List keys from the asset canister.
    #[command(name = "ls")]
    List(ListOpts),

    /// Synchronize a directory to the asset canister
    Sync(SyncOpts),

    /// Uploads an asset to an asset canister.
    Upload(UploadOpts),
}

#[derive(Parser)]
struct ListOpts {
    /// The canister ID.
    canister_id: String,
}

#[derive(Parser)]
struct SyncOpts {
    /// The canister ID.
    canister_id: String,

    /// The directories to synchronize
    directory: Vec<PathBuf>,

    /// Do not delete files from the canister that are not present locally.
    #[arg(long)]
    no_delete: bool,
}

#[derive(Parser)]
struct UploadOpts {
    /// The asset canister ID to manage.
    canister_id: String,

    /// Files or folders to send.
    files: Vec<String>,
}

fn create_identity(maybe_pem: Option<PathBuf>) -> Box<dyn Identity + Sync + Send> {
    if let Some(pem_path) = maybe_pem {
        if let Ok(secp256k_identity) = Secp256k1Identity::from_pem_file(&pem_path) {
            Box::new(secp256k_identity)
        } else {
            Box::new(BasicIdentity::from_pem_file(pem_path).expect("Could not read the key pair."))
        }
    } else {
        Box::new(AnonymousIdentity)
    }
}

fn style() -> Styles {
    let green = Style::new().fg_color(Some(AnsiColor::Green.into()));
    let yellow = Style::new().fg_color(Some(AnsiColor::Yellow.into()));
    let red = Style::new()
        .fg_color(Some(AnsiColor::BrightRed.into()))
        .bold();
    Styles::styled()
        .literal(green)
        .placeholder(green)
        .error(red)
        .header(yellow)
        .invalid(yellow)
        .valid(green)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let logger = support::new_logger();

    let agent = Agent::builder()
        .with_transport(agent::http_transport::ReqwestTransport::create(
            opts.replica.clone(),
        )?)
        .with_boxed_identity(create_identity(opts.pem))
        .build()?;

    let normalized_replica = opts.replica.strip_suffix('/').unwrap_or(&opts.replica);
    if normalized_replica != DEFAULT_IC_GATEWAY {
        agent.fetch_root_key().await?;
    }

    match &opts.subcommand {
        SubCommand::List(o) => {
            let canister = ic_utils::Canister::builder()
                .with_agent(&agent)
                .with_canister_id(Principal::from_text(&o.canister_id)?)
                .build()?;
            list(&canister, &logger).await?;
        }
        SubCommand::Sync(o) => {
            let canister = ic_utils::Canister::builder()
                .with_agent(&agent)
                .with_canister_id(Principal::from_text(&o.canister_id)?)
                .build()?;
            sync(&canister, o, &logger).await?;
        }
        SubCommand::Upload(o) => {
            let canister = ic_utils::Canister::builder()
                .with_agent(&agent)
                .with_canister_id(Principal::from_text(&o.canister_id)?)
                .build()?;
            upload(&canister, o, &logger).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Opts;

    #[test]
    fn validate_cli() {
        Opts::command().debug_assert();
    }
}
