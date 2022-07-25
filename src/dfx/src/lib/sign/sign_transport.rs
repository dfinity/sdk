use super::signed_message::SignedMessageV1;

use candid::Principal;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{AgentError, RequestId};

use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Read, Write};
use std::path::Path;
use std::pin::Pin;
use thiserror::Error;

#[derive(Error, Debug)]
enum SerializeStatus {
    #[error("{0}")]
    Success(String),
}

pub(crate) struct SignReplicaV2Transport {
    file_name: String,
    message_template: SignedMessageV1,
}

impl SignReplicaV2Transport {
    pub fn new<U: Into<String>>(file_name: U, message_template: SignedMessageV1) -> Self {
        Self {
            file_name: file_name.into(),
            message_template,
        }
    }
}

impl ReplicaV2Transport for SignReplicaV2Transport {
    fn read_state<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &SignReplicaV2Transport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
            let path = Path::new(&s.file_name);
            let mut file =
                File::open(&path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            let mut json = String::new();
            file.read_to_string(&mut json)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let message: SignedMessageV1 =
                serde_json::from_str(&json).map_err(|x| AgentError::MessageError(x.to_string()))?;
            let message = message.with_signed_request_status(hex::encode(&envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!(
                    "Signed request_status append to update message in [{}]",
                    &s.file_name
                ))
                .into(),
            ))
        }

        Box::pin(run(self, envelope))
    }

    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        async fn run(
            s: &SignReplicaV2Transport,
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

    fn query<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &SignReplicaV2Transport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
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

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(_: &SignReplicaV2Transport) -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "status calls not supported".to_string(),
            ))
        }

        Box::pin(run(self))
    }
}
