use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::signed_message::SignedMessageV1;
//use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type};

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::agent::ReplicaV1Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::principal::Principal;
use std::option::Option;
use std::path::PathBuf;
use std::pin::Pin;
use std::{fs::File, path::Path};
use std::{future::Future, io::Write};

/// Sign a canister call to be sent
#[derive(Clap)]
pub struct CanisterSignOpts {
    /// Specifies the name of the canister to build.
    /// You must specify either a canister name or the --all option.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    /// Sends a query request to a canister.
    #[clap(long)]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[clap(long, conflicts_with("query"))]
    update: bool,

    /// Specifies the argument to pass to the method.
    argument: Option<String>,

    /// Specifies the config for generating random argument.
    #[clap(long, conflicts_with("argument"))]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    r#type: Option<String>,
}

// TODO: extract this function to util
fn get_local_cid_and_candid_path(
    env: &dyn Environment,
    canister_name: &str,
    maybe_canister_id: Option<Principal>,
) -> DfxResult<(Principal, Option<PathBuf>)> {
    let config = env.get_config_or_anyhow()?;
    let canister_info = CanisterInfo::load(&config, canister_name, maybe_canister_id)?;
    Ok((
        canister_info.get_canister_id()?,
        canister_info.get_output_idl_path(),
    ))
}

struct SignReplicaV1Transport {
    file_name: String,
    message_template: SignedMessageV1,
}

impl SignReplicaV1Transport {
    pub fn new<U: Into<String>>(file_name: U, message_template: SignedMessageV1) -> Self {
        Self {
            file_name: file_name.into(),
            message_template,
        }
    }
}

impl ReplicaV1Transport for SignReplicaV1Transport {
    fn read<'a>(
        &'a self,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &SignReplicaV1Transport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
            let message = s
                .message_template
                .clone()
                .with_call_type("query".to_string())
                .with_content(hex::encode(&envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::TransportError(Box::new(x)))?;
            let path = Path::new(&s.file_name);
            let mut file =
                File::create(&path).map_err(|x| AgentError::TransportError(Box::new(x)))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::TransportError(Box::new(x)))?;
            Err(AgentError::MessageError("read complete".to_string()))
        }

        Box::pin(run(self, envelope))
    }

    fn submit<'a>(
        &'a self,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        async fn run(
            s: &SignReplicaV1Transport,
            envelope: Vec<u8>,
            request_id: RequestId,
        ) -> Result<(), AgentError> {
            // print!(
            //     "submit\n{}\n\n{}",
            //     hex::encode(request_id.as_slice()),
            //     hex::encode(envelope)
            // );
            println!("submit");
            println!("file_name: {}", s.file_name);
            println!("content: {}", hex::encode(envelope));
            println!("request_id{}", hex::encode(request_id.as_slice()));
            Err(AgentError::MessageError(String::new()))
        }

        Box::pin(run(self, envelope, request_id))
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(_: &SignReplicaV1Transport) -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "status calls not supported".to_string(),
            ))
        }

        Box::pin(run(self))
    }
}

pub async fn exec(env: &dyn Environment, opts: CanisterSignOpts) -> DfxResult {
    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let (canister_id, maybe_candid_path) = match Principal::from_text(callee_canister) {
        Ok(id) => {
            if let Some(canister_name) = canister_id_store.get_name(callee_canister) {
                get_local_cid_and_candid_path(env, canister_name, Some(id))?
            } else {
                // TODO fetch candid file from remote canister
                (id, None)
            }
        }
        Err(_) => {
            let canister_id = canister_id_store.get(callee_canister)?;
            get_local_cid_and_candid_path(env, callee_canister, Some(canister_id))?
        }
    };

    let method_type = maybe_candid_path.and_then(|path| get_candid_type(&path, method_name));
    let is_query_method = match &method_type {
        Some((_, f)) => Some(f.is_query()),
        None => None,
    };

    let arguments = opts.argument.as_deref();
    let arg_type = opts.r#type.as_deref();
    // let output_type = opts.output.as_deref();
    let is_query = match is_query_method {
        Some(true) => !opts.update,
        Some(false) => {
            if opts.query {
                bail!(
                    "Invalid method call: {} is not a query method.",
                    method_name
                );
            } else {
                false
            }
        }
        None => opts.query,
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(arguments, opts.random.as_deref(), arg_type, &method_type)?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    //fetch_root_key_if_needed(env).await?;
    let mut identity_manager = IdentityManager::new(env)?;
    identity_manager.instantiate_selected_identity()?;
    let sender = match identity_manager.get_selected_identity_principal() {
        Some(p) => p,
        None => bail!("Cannot get sender's principle"),
    };

    let message_template =
        SignedMessageV1::new(sender, canister_id.clone(), method_name.to_string());

    let mut sign_agent = agent.clone();
    sign_agent.set_transport(SignReplicaV1Transport::new(
        "message.json", // TODO: configurable
        message_template,
    ));

    let timeout = expiry_duration(); // TODO: configurable

    if is_query {
        let res = sign_agent
            .query(&canister_id, method_name)
            .with_arg(&arg_value)
            .call()
            .await;
        println!("{:?}", res);
    } else {
        let request_id = sign_agent
            .update(&canister_id, method_name)
            .with_arg(&arg_value)
            .expire_after(timeout)
            .call()
            .await?;
        println!("{:?}", request_id);
    }

    Ok(())
}
