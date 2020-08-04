pub(crate) mod agent_config;
pub(crate) mod agent_error;
pub(crate) mod nonce;
pub(crate) mod replica_api;
pub(crate) mod response;

pub(crate) mod public {
    pub use super::agent_config::{AgentConfig, AgentRequestExecutor};
    pub use super::agent_error::AgentError;
    pub use super::nonce::NonceFactory;
    pub use super::response::{Replied, RequestStatusResponse};
    pub use super::Agent;
}

#[cfg(test)]
mod agent_test;

use crate::agent::replica_api::{AsyncContent, Envelope, SyncContent};
use crate::identity::Identity;
use crate::{to_request_id, Blob, CanisterId, Principal, RequestId};
use reqwest::Method;
use serde::Serialize;

use public::*;

const DOMAIN_SEPARATOR: &[u8; 11] = b"\x0Aic-request";

pub struct Agent {
    url: reqwest::Url,
    nonce_factory: NonceFactory,
    request_executor: Box<dyn AgentRequestExecutor>,
    identity: Box<dyn Identity>,
    default_waiter: delay::Delay,
}

impl Agent {
    pub fn new(config: AgentConfig<'_>) -> Result<Agent, AgentError> {
        let url = config.url;

        Ok(Agent {
            url: reqwest::Url::parse(url)
                .and_then(|url| url.join("api/v1/"))
                .map_err(|_| AgentError::InvalidClientUrl(String::from(url)))?,
            request_executor: config.request_executor,
            nonce_factory: config.nonce_factory,
            identity: config.identity,
            default_waiter: config.default_waiter,
        })
    }

    fn construct_message(&self, request_id: &RequestId) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice(DOMAIN_SEPARATOR);
        buf.extend_from_slice(Blob::from(*request_id).as_slice());
        buf
    }

    async fn execute<T: std::fmt::Debug + serde::Serialize>(
        &self,
        endpoint: &str,
        envelope: Envelope<T>,
    ) -> Result<Vec<u8>, AgentError> {
        let mut serialized_bytes = Vec::new();
        let mut serializer = serde_cbor::Serializer::new(&mut serialized_bytes);
        serializer.self_describe()?;
        envelope.serialize(&mut serializer)?;

        let url = self.url.join(endpoint)?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request.headers_mut().insert(
            reqwest::header::CONTENT_TYPE,
            "application/cbor".parse().unwrap(),
        );
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(serialized_bytes));

        let response = self.request_executor.execute(http_request).await?;
        if response.status().is_client_error() || response.status().is_server_error() {
            Err(AgentError::ServerError {
                status: response.status().into(),
                content_type: response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(|x| x.to_string()),
                content: response.text().await?,
            })
        } else {
            Ok(response.bytes().await?.to_vec())
        }
    }

    async fn read<A>(&self, request: SyncContent) -> Result<A, AgentError>
    where
        A: serde::de::DeserializeOwned,
    {
        let anonymous = Principal::anonymous();
        let request_id = to_request_id(&request)?;
        let sender = match &request {
            SyncContent::QueryRequest { sender, .. } => sender,
            SyncContent::RequestStatusRequest { .. } => &anonymous,
        };
        let msg = self.construct_message(&request_id);
        let signature = self.identity.sign(&msg, &sender)?;
        let bytes = self
            .execute(
                "read",
                Envelope {
                    content: request,
                    sender_pubkey: signature.public_key,
                    sender_sig: signature.signature,
                },
            )
            .await?;

        serde_cbor::from_slice(&bytes).map_err(AgentError::InvalidCborData)
    }

    async fn submit(&self, request: AsyncContent) -> Result<RequestId, AgentError> {
        let request_id = to_request_id(&request)?;
        let sender = match request.clone() {
            AsyncContent::CallRequest { sender, .. } => sender,
        };
        let msg = self.construct_message(&request_id);
        let signature = self.identity.sign(&msg, &sender)?;
        let _ = self
            .execute(
                "submit",
                Envelope {
                    content: request,
                    sender_pubkey: signature.public_key,
                    sender_sig: signature.signature,
                },
            )
            .await?;

        Ok(request_id)
    }

    /// The simplest for of query; sends a Blob and will return a Blob. The encoding is
    /// left as an exercise to the user.
    pub async fn query<'a>(
        &self,
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
    ) -> Result<Blob, AgentError> {
        self.read::<replica_api::QueryResponse>(SyncContent::QueryRequest {
            sender: self.identity.sender()?,
            canister_id: canister_id.clone(),
            method_name: method_name.to_string(),
            arg: arg.clone(),
        })
        .await
        .and_then(|response| match response {
            replica_api::QueryResponse::Replied { reply } => Ok(reply.arg),
            replica_api::QueryResponse::Rejected {
                reject_code,
                reject_message,
            } => Err(AgentError::ReplicaError {
                reject_code,
                reject_message,
            }),
        })
    }

    pub async fn request_status(&self, request_id: &RequestId) -> Result<Replied, AgentError> {
        self.request_status_and_wait(request_id, self.default_waiter.clone())
            .await
    }

    pub async fn request_status_raw(
        &self,
        request_id: &RequestId,
    ) -> Result<RequestStatusResponse, AgentError> {
        self.read(SyncContent::RequestStatusRequest {
            request_id: request_id.as_slice().into(),
        })
        .await
        .map(|response| match response {
            replica_api::RequestStatusResponse::Replied { reply } => {
                let reply = match reply {
                    replica_api::RequestStatusResponseReplied::CallReply(reply) => {
                        Replied::CallReplied(reply.arg)
                    }
                };

                RequestStatusResponse::Replied { reply }
            }
            replica_api::RequestStatusResponse::Unknown {} => RequestStatusResponse::Unknown,
            replica_api::RequestStatusResponse::Received {} => RequestStatusResponse::Received,
            replica_api::RequestStatusResponse::Processing {} => RequestStatusResponse::Processing,
            replica_api::RequestStatusResponse::Rejected {
                reject_code,
                reject_message,
            } => RequestStatusResponse::Rejected {
                reject_code,
                reject_message,
            },
        })
    }

    pub async fn request_status_and_wait<W: delay::Waiter>(
        &self,
        request_id: &RequestId,
        mut waiter: W,
    ) -> Result<Replied, AgentError> {
        waiter.start();

        loop {
            match self.request_status_raw(request_id).await? {
                RequestStatusResponse::Replied { reply } => return Ok(reply),
                RequestStatusResponse::Rejected {
                    reject_code,
                    reject_message,
                } => {
                    return Err(AgentError::ReplicaError {
                        reject_code,
                        reject_message,
                    })
                }
                RequestStatusResponse::Unknown => (),
                RequestStatusResponse::Received => (),
                RequestStatusResponse::Processing => (),
            };

            waiter
                .wait()
                .map_err(|_| AgentError::TimeoutWaitingForResponse)?;
        }
    }

    pub async fn call_and_wait<W: delay::Waiter>(
        &self,
        canister_id: &CanisterId,
        method_name: &str,
        arg: &Blob,
        waiter: W,
    ) -> Result<Blob, AgentError> {
        let request_id = self.call_raw(canister_id, method_name, arg).await?;
        match self.request_status_and_wait(&request_id, waiter).await? {
            Replied::CallReplied(arg) => Ok(arg),
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }

    pub async fn call(
        &self,
        canister_id: &CanisterId,
        method_name: &str,
        arg: &Blob,
    ) -> Result<Blob, AgentError> {
        self.call_and_wait(canister_id, method_name, arg, self.default_waiter.clone())
            .await
    }

    pub async fn call_raw(
        &self,
        canister_id: &CanisterId,
        method_name: &str,
        arg: &Blob,
    ) -> Result<RequestId, AgentError> {
        self.submit(AsyncContent::CallRequest {
            canister_id: canister_id.clone(),
            method_name: method_name.into(),
            arg: arg.clone(),
            nonce: self.nonce_factory.generate().map(|b| b.as_slice().into()),
            sender: self.identity.sender()?,
        })
        .await
    }

    pub async fn ping_once(&self) -> Result<serde_cbor::Value, AgentError> {
        let url = self.url.join("status")?;
        let mut http_request = reqwest::Request::new(Method::GET, url);
        http_request.headers_mut().insert(
            reqwest::header::CONTENT_TYPE,
            "application/cbor".parse().unwrap(),
        );
        let response = self.request_executor.execute(http_request).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            Err(AgentError::ServerError {
                status: response.status().into(),
                content_type: response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(|x| x.to_string()),
                content: response.text().await?,
            })
        } else {
            let bytes = response.bytes().await?.to_vec();
            Ok(serde_cbor::from_slice(&bytes).map_err(AgentError::InvalidCborData)?)
        }
    }

    pub async fn ping<W: delay::Waiter>(
        &self,
        mut waiter: W,
    ) -> Result<serde_cbor::Value, AgentError> {
        waiter.start();
        loop {
            // Break if the server/replica answered but was an error (compared to not being
            // able to reach the server).
            match self.ping_once().await {
                Ok(x) => return Ok(x),
                Err(AgentError::ReqwestError(_)) => {}
                Err(x) => return Err(x),
            }

            waiter
                .wait()
                .map_err(|_| AgentError::TimeoutWaitingForResponse)?;
        }
    }
}
