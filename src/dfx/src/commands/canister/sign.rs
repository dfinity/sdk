use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::signed_message::SignedMessageV1;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type};

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::agent::ReplicaV1Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::principal::Principal;
use slog::info;
use std::option::Option;
use std::path::PathBuf;
use std::pin::Pin;
use std::{fs::File, path::Path};
use std::{future::Future, io::Write};
use thiserror::Error;

/// Sign a canister call and generate message file in json
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

    /// Specifies the output file name.
    #[clap(long, default_value("message.json"))]
    output: String,

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

#[derive(Error, Debug)]
enum SerializeStatus {
    #[error("{0}")]
    Success(String),
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
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let path = Path::new(&s.file_name);
            let mut file =
                File::create(&path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!("Query message generated at [{}]", &s.file_name))
                    .into(),
            ))
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
            let message = s
                .message_template
                .clone()
                .with_call_type("update".to_string())
                .with_request_id(request_id)
                .with_content(hex::encode(&envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let path = Path::new(&s.file_name);
            let mut file =
                File::create(&path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!("Update message generated at [{}]", &s.file_name))
                    .into(),
            ))
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
    let log = env.get_logger();
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

    let network = env
        .get_network_descriptor()
        .expect("Cannot get network descriptor from environment.")
        .providers
        .first()
        .expect("Cannot get network provider (url).")
        .to_string();

    let mut identity_manager = IdentityManager::new(env)?;
    identity_manager.instantiate_selected_identity()?;
    let sender = match identity_manager.get_selected_identity_principal() {
        Some(p) => p,
        None => bail!("Cannot get sender's principle"),
    }; // TODO: use call_sender?

    let message_template = SignedMessageV1::new(
        network,
        sender,
        canister_id.clone(),
        method_name.to_string(),
    );

    let file_name = opts.output;
    if Path::new(&file_name).exists() {
        bail!(
            "[{}] already exists, please specify a different output file name.",
            file_name
        );
    }

    let mut sign_agent = agent.clone();
    sign_agent.set_transport(SignReplicaV1Transport::new(file_name, message_template));

    let timeout = expiry_duration(); // TODO: configurable

    if is_query {
        let res = sign_agent
            .query(&canister_id, method_name)
            .with_arg(&arg_value)
            .call()
            .await;
        match res {
            Err(AgentError::TransportError(b)) => {
                info!(log, "{}", b);
                Ok(())
            }
            Err(e) => bail!(e),
            Ok(_) => unreachable!(),
        }
    } else {
        let res = sign_agent
            .update(&canister_id, method_name)
            .with_arg(&arg_value)
            .expire_after(timeout)
            .call()
            .await;
        match res {
            Err(AgentError::TransportError(b)) => {
                info!(log, "{}", b);
                Ok(())
            }
            Err(e) => bail!(e),
            Ok(_) => unreachable!(),
        }
    }
}
