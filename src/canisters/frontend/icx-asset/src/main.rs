mod commands;
mod support;

use crate::commands::list::list;
use crate::commands::sync::sync;
use candid::Principal;
use clap::{crate_authors, crate_version, Parser};
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::{agent, Agent, Identity};

use crate::commands::upload::upload;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_IC_GATEWAY: &str = "https://ic0.app";

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    propagate_version(true),
)]
struct Opts {
    /// Some input. Because this isn't an Option<T> it's required to be used
    #[clap(long, default_value = "http://localhost:8000/")]
    replica: String,

    /// An optional PEM file to read the identity from. If none is passed,
    /// a random identity will be created.
    #[clap(long)]
    pem: Option<PathBuf>,

    /// An optional field to set the expiry time on requests. Can be a human
    /// readable time (like `100s`) or a number of seconds.
    #[clap(long)]
    ttl: Option<humantime::Duration>,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// List keys from the asset canister.
    #[clap(name = "ls")]
    List(ListOpts),

    /// Synchronize a directory to the asset canister
    Sync(SyncOpts),

    /// Uploads an asset to an asset canister.
    Upload(UploadOpts),
}

#[derive(Parser)]
struct ListOpts {
    /// The canister ID.
    #[clap()]
    canister_id: String,
}

#[derive(Parser)]
struct SyncOpts {
    /// The canister ID.
    #[clap()]
    canister_id: String,

    /// The directories to synchronize
    #[clap()]
    directory: Vec<PathBuf>,
}

#[derive(Parser)]
struct UploadOpts {
    /// The asset canister ID to manage.
    #[clap()]
    canister_id: String,

    /// Files or folders to send.
    #[clap()]
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

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> support::Result {
    let opts: Opts = Opts::parse();

    let ttl: std::time::Duration = opts
        .ttl
        .map(|ht| ht.into())
        .unwrap_or_else(|| Duration::from_secs(60 * 5)); // 5 minutes is max ingress timeout

    let agent = Agent::builder()
        .with_transport(
            agent::http_transport::ReqwestHttpReplicaV2Transport::create(opts.replica.clone())?,
        )
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
            list(&canister).await?;
        }
        SubCommand::Sync(o) => {
            let canister = ic_utils::Canister::builder()
                .with_agent(&agent)
                .with_canister_id(Principal::from_text(&o.canister_id)?)
                .build()?;
            sync(&canister, ttl, o).await?;
        }
        SubCommand::Upload(o) => {
            let canister = ic_utils::Canister::builder()
                .with_agent(&agent)
                .with_canister_id(Principal::from_text(&o.canister_id)?)
                .build()?;
            upload(&canister, o).await?;
        }
    }

    Ok(())
}
