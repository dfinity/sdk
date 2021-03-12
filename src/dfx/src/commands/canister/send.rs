use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::signed_message::SignedMessageV1;

use clap::Clap;
use ic_agent::agent::ReplicaV1Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV1Transport, RequestId};

use std::{fs::File, path::Path};
use std::{io::Read, str::FromStr};

/// Send a signed message
#[derive(Clap)]
pub struct CanisterSendOpts;

pub async fn exec(_env: &dyn Environment, _opts: CanisterSendOpts) -> DfxResult {
    let file_name = "message.json"; // TODO: configurable
    let path = Path::new(&file_name);
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;
    let message: SignedMessageV1 = serde_json::from_str(&json)?;

    let replica = "http://localhost:8000/"; // TODO: configurable or be a field in the message?
    let transport = ReqwestHttpReplicaV1Transport::create(replica)?;
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
