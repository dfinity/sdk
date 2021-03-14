use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::signed_message::SignedMessageV1;

use ic_agent::agent::ReplicaV1Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV1Transport, RequestId};

use anyhow::anyhow;
use clap::Clap;
use slog::info;
use std::{fs::File, path::Path};
use std::{io::Read, str::FromStr};

/// Send a signed message
#[derive(Clap)]
pub struct CanisterSendOpts {
    /// Specifies the file name of the message
    file_name: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterSendOpts) -> DfxResult {
    let log = env.get_logger();
    let file_name = opts.file_name;
    let path = Path::new(&file_name);
    let mut file = File::open(&path).map_err(|_| anyhow!("Message file doesn't exist."))?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| anyhow!("Cannot read the message file."))?;
    let message: SignedMessageV1 =
        serde_json::from_str(&json).map_err(|_| anyhow!("Invalid json message."))?;
    message.validate()?;

    info!(log, "Will send message:");
    info!(log, "  Network:     {}", message.network);
    info!(log, "  Call type:   {}", message.call_type);
    info!(log, "  Sender:      {}", message.sender);
    info!(log, "  Canister id: {}", message.canister_id);
    info!(log, "  Method name: {}", message.method_name);
    info!(log, "  Arg:         {:?}", message.arg);
    info!(log, "\nOkay? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !["y", "yes"].contains(&input.to_lowercase().trim()) {
        return Ok(());
    }

    let network = message.network;
    let transport = ReqwestHttpReplicaV1Transport::create(network)?;
    let content = hex::decode(&message.content)?;

    match message.call_type.as_str() {
        "query" => {
            let result = transport.read(content).await?;
            eprint!("Result: ");
            println!("{}", hex::encode(result));
        }
        "update" => {
            let request_id = RequestId::from_str(&message.request_id)?;
            transport.submit(content, request_id).await?;
            eprint!("Request ID: ");
            println!("0x{}", String::from(request_id));
        }
        _ => {}
    }
    Ok(())
}
