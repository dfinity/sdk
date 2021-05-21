use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::util::print_idl_blob;

use ic_agent::agent::replica_api::QueryResponse;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, RequestId};

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
        let envelope = hex::decode(&message.signed_request_status.unwrap())?;
        let response = transport.read_state(canister_id.clone(), envelope).await?;
        eprintln!("To see the content of response, copy-paste the encoded string into cbor.me.");
        eprint!("Response: ");
        println!("{}", hex::encode(response));
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
                    bail!(
                        r#"The Replica returned an error: code {}, message: "{}""#,
                        reject_code,
                        reject_message
                    );
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
            eprintln!("Alternatively, if you have the correct identity on this machine, using `dfx canister request-status` with following arguments.");
            eprint!("Request ID: ");
            println!("0x{}", String::from(request_id));
            eprint!("Canister ID: ");
            println!("{}", canister_id.to_string());
        }
        // message.validate() guarantee that call_type must be query or update
        _ => unreachable!(),
    }
    Ok(())
}
