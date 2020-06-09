pub(crate) mod agent_config;
pub(crate) mod agent_error;
pub(crate) mod nonce;
pub(crate) mod replica_api;
pub(crate) mod response;

pub(crate) mod public {
    pub use super::agent_config::AgentConfig;
    pub use super::agent_error::AgentError;
    pub use super::nonce::NonceFactory;
    pub use super::response::{Replied, RequestStatusResponse};
    pub use super::Agent;
}

#[cfg(test)]
mod agent_test;

use crate::agent::replica_api::{AsyncContent, Envelope, SyncContent};
use crate::identity::Identity;
use crate::{to_request_id, Blob, CanisterAttributes, CanisterId, Principal, RequestId};

use public::*;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use std::convert::TryInto;

pub struct Agent {
    url: reqwest::Url,
    client: reqwest::Client,
    nonce_factory: NonceFactory,
    identity: Box<dyn Identity>,
}

impl Agent {
    pub fn new(config: AgentConfig<'_>) -> Result<Agent, AgentError> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/cbor".parse().unwrap(),
        );

        let url = config.url;

        Ok(Agent {
            url: reqwest::Url::parse(url)
                .and_then(|url| url.join("api/v1/"))
                .map_err(|_| AgentError::InvalidClientUrl(String::from(url)))?,
            client: Client::builder().default_headers(default_headers).build()?,
            nonce_factory: config.nonce_factory,
            identity: config.identity,
        })
    }

    async fn execute<T: std::fmt::Debug + serde::Serialize>(
        &self,
        endpoint: &str,
        envelope: Envelope<T>,
    ) -> Result<Vec<u8>, AgentError> {
        let serialized_bytes = serde_cbor::to_vec(&envelope)?;
        let url = self.url.join(endpoint)?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(serialized_bytes));

        let response = self.client.execute(http_request).await?;
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
        let signature = self.identity.sign(&request_id, &sender)?;
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
            AsyncContent::CreateCanisterRequest { sender, .. } => sender,
            AsyncContent::CallRequest { sender, .. } => sender,
            AsyncContent::InstallCodeRequest { sender, .. } => sender,
        };
        let signature = self.identity.sign(&request_id, &sender)?;
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
            arg: arg.clone().into(),
        })
        .await
        .and_then(|response| match response {
            replica_api::QueryResponse::Replied { reply } => Ok(Blob::from(reply.arg)),
            replica_api::QueryResponse::Rejected {
                reject_code,
                reject_message,
            } => Err(AgentError::ReplicaError {
                reject_code,
                reject_message,
            }),
        })
    }

    pub async fn request_status(
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
                        Replied::CallReplied(Blob::from(reply.arg))
                    }
                    replica_api::RequestStatusResponseReplied::CreateCanisterReply(reply) => {
                        Replied::CreateCanisterReplied(reply.canister_id)
                    }
                    replica_api::RequestStatusResponseReplied::InstallCodeReply(_) => {
                        Replied::InstallCodeReplied
                    }
                };

                RequestStatusResponse::Replied { reply }
            }
            replica_api::RequestStatusResponse::Unknown {} => RequestStatusResponse::Unknown,
            replica_api::RequestStatusResponse::Received {} => RequestStatusResponse::Pending,
            replica_api::RequestStatusResponse::Processing {} => RequestStatusResponse::Pending,
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
            match self.request_status(request_id).await? {
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
                RequestStatusResponse::Unknown {} => (),
                RequestStatusResponse::Pending {} => (),
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
        let request_id = self.call(canister_id, method_name, arg).await?;
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
    ) -> Result<RequestId, AgentError> {
        self.submit(AsyncContent::CallRequest {
            canister_id: canister_id.clone(),
            method_name: method_name.into(),
            arg: arg.clone().into(),
            nonce: self.nonce_factory.generate().map(|b| b.as_slice().into()),
            sender: self.identity.sender()?,
        })
        .await
    }

    pub async fn create_canister(&self) -> Result<RequestId, AgentError> {
        self.create_canister_with_desired_id(None).await
    }

    pub async fn create_canister_with_desired_id(
        &self,
        desired_id: Option<CanisterId>,
    ) -> Result<RequestId, AgentError> {
        self.submit(AsyncContent::CreateCanisterRequest {
            sender: self.identity.sender()?,
            nonce: self.nonce_factory.generate().map(|b| b.into()),
            desired_id: desired_id.map(|id| id.as_bytes().try_into().unwrap()),
        })
        .await
    }

    pub async fn create_canister_and_wait<W: delay::Waiter>(
        &self,
        waiter: W,
    ) -> Result<CanisterId, AgentError> {
        let request_id = self.create_canister().await?;
        match self.request_status_and_wait(&request_id, waiter).await? {
            Replied::CreateCanisterReplied(id) => Ok(id),
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }

    pub async fn install(
        &self,
        canister_id: &CanisterId,
        module: &Blob,
        arg: &Blob,
    ) -> Result<RequestId, AgentError> {
        self.install_with_attrs(canister_id, "", module, arg, &CanisterAttributes::default())
            .await
    }

    pub async fn install_and_wait<W: delay::Waiter>(
        &self,
        canister_id: &CanisterId,
        module: &Blob,
        arg: &Blob,
        waiter: W,
    ) -> Result<(), AgentError> {
        let request_id = self.install(canister_id, module, arg).await?;
        match self.request_status_and_wait(&request_id, waiter).await? {
            Replied::InstallCodeReplied => Ok(()),
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }

    pub async fn install_with_attrs(
        &self,
        canister_id: &CanisterId,
        mode: &str,
        module: &Blob,
        arg: &Blob,
        attributes: &CanisterAttributes,
    ) -> Result<RequestId, AgentError> {
        println!("install_with_attrs {:?}", canister_id.clone().to_text());
        let mode = match mode {
            "install" => Some(mode.to_string()),
            "reinstall" => Some(mode.to_string()),
            "upgrade" => Some(mode.to_string()),
            &_ => None,
        };
        self.submit(AsyncContent::InstallCodeRequest {
            nonce: self.nonce_factory.generate().map(|b| b.as_slice().into()),
            sender: self.identity.sender()?,
            canister_id: canister_id.clone(),
            module: module.clone().into(),
            arg: arg.clone().into(),
            compute_allocation: attributes.compute_allocation.map(|x| x.into()),
            memory_allocation: None,
            mode: mode,
        })
        .await
    }

    pub async fn ping_once(&self) -> Result<(), AgentError> {
        let url = self.url.join("status")?;
        let http_request = reqwest::Request::new(Method::GET, url);
        let response = self.client.execute(http_request).await?;

        if response.status().as_u16() == 200 {
            Ok(())
        } else {
            // Verify the error is 2XX.
            response
                .error_for_status()
                .map(|_| ())
                .map_err(AgentError::from)
        }
    }

    pub async fn ping<W: delay::Waiter>(&self, mut waiter: W) -> Result<(), AgentError> {
        waiter.start();
        loop {
            if self.ping_once().await.is_ok() {
                break;
            }

            waiter
                .wait()
                .map_err(|_| AgentError::TimeoutWaitingForResponse)?;
        }
        Ok(())
    }
}
