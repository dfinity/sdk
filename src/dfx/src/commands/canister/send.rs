use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::util::print_idl_blob;

use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::agent::replica_api::{Certificate, QueryResponse, ReadStateResponse};
use ic_agent::agent::{lookup_request_status, ReplicaV2Transport, RequestStatusResponse};
use ic_agent::AgentError;
use ic_agent::RequestId;

use anyhow::{anyhow, bail, Context};
use clap::Clap;
use ic_types::Principal;
use std::{fs::File, path::Path};
use std::{io::Read, str::FromStr};

/// Send a signed message
#[derive(Clap)]
pub struct CanisterSendOpts {
    /// Specifies the file name of the message
    file_name: String,

    /// Send the signed request-status call in the message
    #[clap(long)]
    status: bool,
}

pub async fn exec(
    _env: &dyn Environment,
    opts: CanisterSendOpts,
    call_sender: &CallSender,
) -> DfxResult {
    if *call_sender != CallSender::SelectedId {
        bail!("`sign` currently doesn't support proxy through wallet canister, please use `dfx canister --no-wallet send ...`.");
    }
    let file_name = opts.file_name;
    let path = Path::new(&file_name);
    let mut file = File::open(&path).map_err(|_| anyhow!("Message file doesn't exist."))?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| anyhow!("Cannot read the message file."))?;
    let message: SignedMessageV1 =
        serde_json::from_str(&json).map_err(|_| anyhow!("Invalid json message."))?;
    message.validate()?;

    let network = message.network.clone();
    let transport = ReqwestHttpReplicaV2Transport::create(network)?;
    let content = hex::decode(&message.content)?;
    let canister_id = Principal::from_text(message.canister_id.clone())?;

    if opts.status {
        if message.call_type.clone().as_str() != "update" {
            bail!("Can only check request_status on update calls.");
        }
        if message.signed_request_status.is_none() {
            bail!("No signed_request_status in [{}].", file_name);
        }
        let request_id = RequestId::from_str(
            &message
                .request_id
                .expect("Cannot get request_id from the update message."),
        )?;
        let envelope = hex::decode(&message.signed_request_status.unwrap())?;
        let response = transport.read_state(canister_id.clone(), envelope).await?;
        let read_state_response: ReadStateResponse = serde_cbor::from_slice(&response)?;
        let certificate: Certificate = serde_cbor::from_slice(&read_state_response.certificate)?;
        // certificate is not verified here because we are using Transport instead of Agent
        let request_status_response = lookup_request_status(certificate, &request_id)?;
        match request_status_response {
            RequestStatusResponse::Replied { reply } => {
                let ic_agent::agent::Replied::CallReplied(blob) = reply;
                print_idl_blob(&blob, Some("idl"), &None)
                    .context("Invalid data: Invalid IDL blob.")?;
            }
            RequestStatusResponse::Rejected {
                reject_code,
                reject_message,
            } => {
                return Err(DfxError::new(AgentError::ReplicaError {
                    reject_code,
                    reject_message,
                }))
            }
            RequestStatusResponse::Unknown => (),
            RequestStatusResponse::Received | RequestStatusResponse::Processing => {
                eprintln!("The update call has been received and is processing.")
            }
            RequestStatusResponse::Done => {
                return Err(DfxError::new(AgentError::RequestStatusDoneNoReply(
                    String::from(request_id),
                )))
            }
        }
        return Ok(());
    }

    eprintln!("Will send message:");
    eprintln!("  Creation:    {}", message.creation);
    eprintln!("  Expiration:  {}", message.expiration);
    eprintln!("  Network:     {}", message.network);
    eprintln!("  Call type:   {}", message.call_type);
    eprintln!("  Sender:      {}", message.sender);
    eprintln!("  Canister id: {}", message.canister_id);
    eprintln!("  Method name: {}", message.method_name);
    eprintln!("  Arg:         {:?}", message.arg);

    // Not using dialoguer because it doesn't support non terminal env like bats e2e
    eprintln!("\nOkay? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !["y", "yes"].contains(&input.to_lowercase().trim()) {
        return Ok(());
    }

    match message.call_type.as_str() {
        "query" => {
            let response = transport.query(canister_id, content).await?;
            let query_response: QueryResponse = serde_cbor::from_slice(&response)?;
            match query_response {
                QueryResponse::Replied { reply } => {
                    let blob = reply.arg;
                    print_idl_blob(&blob, Some("idl"), &None)
                        .context("Invalid data: Invalid IDL blob.")?;
                }
                QueryResponse::Rejected {
                    reject_code,
                    reject_message,
                } => {
                    return Err(DfxError::new(AgentError::ReplicaError {
                        reject_code,
                        reject_message,
                    }))
                }
            }
        }
        "update" => {
            let request_id = RequestId::from_str(
                &message
                    .request_id
                    .expect("Cannot get request_id from the update message."),
            )?;
            transport
                .call(canister_id.clone(), content, request_id)
                .await?;

            eprintln!(
                "To check the status of this update call, append `--status` to current command."
            );
            eprintln!("e.g. `dfx canister send message.json --status`");
        }
        // message.validate() guarantee that call_type must be query or update
        _ => unreachable!(),
    }
    Ok(())
}
