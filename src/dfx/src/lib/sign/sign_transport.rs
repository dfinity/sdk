use super::signed_message::SignedMessageV1;
use candid::Principal;
use ic_agent::agent::Transport;
use ic_agent::{AgentError, RequestId};
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::pin::Pin;
use thiserror::Error;

#[derive(Error, Debug)]
enum SerializeStatus {
    #[error("{0}")]
    Success(String),
}

pub(crate) struct SignTransport {
    file_name: PathBuf,
    message_template: SignedMessageV1,
}

impl SignTransport {
    pub fn new<U: Into<PathBuf>>(file_name: U, message_template: SignedMessageV1) -> Self {
        Self {
            file_name: file_name.into(),
            message_template,
        }
    }
}

impl Transport for SignTransport {
    fn read_state<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &SignTransport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
            let path = &s.file_name;
            let mut file = File::open(path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            let mut json = String::new();
            file.read_to_string(&mut json)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let message: SignedMessageV1 =
                serde_json::from_str(&json).map_err(|x| AgentError::MessageError(x.to_string()))?;
            let message = message.with_signed_request_status(hex::encode(envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!(
                    "Signed request_status append to update message in [{}]",
                    s.file_name.display()
                ))
                .into(),
            ))
        }

        Box::pin(run(self, envelope))
    }

    fn read_subnet_state(
        &self,
        _: Principal,
        _: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + '_>> {
        async fn run() -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "read_subnet_state calls not supported".to_string(),
            ))
        }
        Box::pin(run())
    }

    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        async fn run(
            s: &SignTransport,
            envelope: Vec<u8>,
            request_id: RequestId,
        ) -> Result<(), AgentError> {
            let message = s
                .message_template
                .clone()
                .with_call_type("update".to_string())
                .with_request_id(request_id)
                .with_content(hex::encode(envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let path = &s.file_name;
            let mut file =
                File::create(path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!(
                    "Update message generated at [{}]",
                    s.file_name.display()
                ))
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
        async fn run(s: &SignTransport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
            let message = s
                .message_template
                .clone()
                .with_call_type("query".to_string())
                .with_content(hex::encode(envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            let path = &s.file_name;
            let mut file =
                File::create(path).map_err(|x| AgentError::MessageError(x.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            Err(AgentError::TransportError(
                SerializeStatus::Success(format!(
                    "Query message generated at [{}]",
                    s.file_name.display()
                ))
                .into(),
            ))
        }

        Box::pin(run(self, envelope))
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(_: &SignTransport) -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "status calls not supported".to_string(),
            ))
        }

        Box::pin(run(self))
    }
}
