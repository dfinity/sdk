use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::sign::signed_message::SignedMessageV1;
use anyhow::{bail, Context};
use candid::{IDLArgs, Principal};
use clap::Parser;
use dfx_core::identity::CallSender;
use dfx_core::json::load_json_file;
use ic_agent::agent::{CallResponse, RequestStatusResponse};
use ic_agent::Agent;
use ic_agent::RequestId;
use std::path::PathBuf;

/// Send a previously-signed message.
#[derive(Parser)]
pub struct CanisterSendOpts {
    /// Specifies the file name of the message
    file_name: PathBuf,

    /// Send the signed request-status call in the message
    #[arg(long)]
    status: bool,
}

pub async fn exec(
    _env: &dyn Environment,
    opts: CanisterSendOpts,
    call_sender: &CallSender,
) -> DfxResult {
    if *call_sender != CallSender::SelectedId {
        bail!("`send` currently doesn't support proxying through the wallet canister, please use `dfx canister send --no-wallet ...`.");
    }
    let file_name = opts.file_name;
    let message: SignedMessageV1 = load_json_file(&file_name)?;
    message.validate()?;

    let network = message.network.clone();
    let agent = Agent::builder().with_url(&network).build()?;
    let content = hex::decode(&message.content).context("Failed to decode message content.")?;
    let canister_id = Principal::from_text(&message.canister_id)
        .with_context(|| format!("Failed to parse canister id {:?}.", message.canister_id))?;

    if opts.status {
        if message.call_type != "update" {
            bail!("Can only check request_status on update calls.");
        }
        let Some(signed_request_status) = message.signed_request_status else {
            bail!("No signed_request_status in [{}].", file_name.display());
        };
        let envelope = hex::decode(signed_request_status).context("Failed to decode envelope.")?;
        let Some(request_id) = message.request_id else {
            bail!("No request_id in [{}].", file_name.display());
        };
        let request_id = request_id
            .parse::<RequestId>()
            .context("Failed to decode request ID.")?;
        let response = agent
            .request_status_signed(&request_id, canister_id, envelope)
            .await
            .with_context(|| format!("Failed to read canister state of {}.", canister_id))?;
        eprint!("Response: ");
        match response {
            RequestStatusResponse::Received => eprintln!("Received, not yet processing"),
            RequestStatusResponse::Processing => eprintln!("Processing, not yet done"),
            RequestStatusResponse::Rejected(response) => {
                if let Some(error_code) = response.error_code {
                    println!(
                        "Rejected ({:?}): {}, error code {}",
                        response.reject_code, response.reject_message, error_code
                    );
                } else {
                    println!(
                        "Rejected ({:?}): {}",
                        response.reject_code, response.reject_message
                    );
                }
            }
            RequestStatusResponse::Replied(response) => {
                eprint!("Replied: ");
                if let Ok(idl) = IDLArgs::from_bytes(&response.arg) {
                    println!("{idl}");
                } else {
                    println!("{}", hex::encode(&response.arg));
                }
            }
            RequestStatusResponse::Done => println!("Done, response no longer available"),
            RequestStatusResponse::Unknown => println!("Unknown"),
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
    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read stdin.")?;
    if !["y", "yes"].contains(&input.to_lowercase().trim()) {
        return Ok(());
    }

    match message.call_type.as_str() {
        "query" => {
            let response = agent
                .query_signed(canister_id, content)
                .await
                .with_context(|| format!("Query call to {} failed.", canister_id))?;
            eprint!("Response: ");
            if let Ok(idl) = IDLArgs::from_bytes(&response) {
                println!("{}", idl)
            } else {
                println!("{}", hex::encode(&response));
            }
        }
        "update" => {
            let call_response = agent
                .update_signed(canister_id, content)
                .await
                .with_context(|| format!("Update call to {} failed.", canister_id))?;
            match call_response {
                CallResponse::Poll(request_id) => {
                    eprintln!(
                        "To check the status of this update call, append `--status` to current command."
                    );
                    eprintln!("e.g. `dfx canister send message.json --status`");
                    eprintln!("Alternatively, if you have the correct identity on this machine, using `dfx canister request-status` with following arguments.");
                    eprint!("Request ID: ");
                    println!("0x{}", String::from(request_id));
                    eprint!("Canister ID: ");
                    println!("{}", canister_id);
                }
                CallResponse::Response(response) => {
                    eprint!("Response: ");
                    if let Ok(idl) = IDLArgs::from_bytes(&response) {
                        println!("{idl}");
                    } else {
                        println!("{}", hex::encode(&response));
                    }
                }
            }
        }
        // message.validate() guarantee that call_type must be query or update
        _ => unreachable!(),
    }
    Ok(())
}
