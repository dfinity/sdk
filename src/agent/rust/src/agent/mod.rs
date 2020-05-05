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
    pub use super::Signature;
}

#[cfg(test)]
mod agent_test;

use crate::{to_request_id, Blob, CanisterAttributes, CanisterId, RequestId, Signer};

use public::*;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Response};
use serde::Serialize;

/// A signature for a request.
pub struct Signature {
    pub public_key: Blob,
    pub signature: Blob,
}

pub struct Agent {
    url: reqwest::Url,
    client: reqwest::Client,
    nonce_factory: NonceFactory,
    signer: Box<dyn Signer>,
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
            url: reqwest::Url::parse(url).and_then(|url| url.join("api/v1/"))?,
            client: Client::builder().default_headers(default_headers).build()?,
            nonce_factory: config.nonce_factory,
            signer: config.signer,
        })
    }

    async fn execute<B>(&self, url: reqwest::Url, body: B) -> Result<Response, AgentError>
    where
        B: Serialize,
    {
        let mut record = Vec::new();
        let mut serializer = serde_cbor::Serializer::new(&mut record);
        serializer.self_describe()?;
        body.serialize(&mut serializer)?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(record));

        let response = self.client.execute(http_request).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            Err(AgentError::ServerError {
                status: response.status().into(),
                content_type: response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("<unknown>")
                    .to_string(),
                content: response.text().await?.to_string(),
            })
        } else {
            Ok(response)
        }
    }

    async fn read<A>(&self, request: replica_api::SyncContent) -> Result<A, AgentError>
    where
        A: serde::de::DeserializeOwned,
    {
        let request_id = to_request_id(&request)?;
        let signature = self.signer.sign(&request_id)?;

        let request = replica_api::SyncRequest {
            signatures: vec![replica_api::Signatures0 {
                sender_pubkey: signature.public_key.as_slice().to_vec(),
                sender_sig: signature.signature.as_slice().to_vec(),
            }],
            content: request,
        };

        let response = self.execute(self.url.join("read")?, request).await?;

        let bytes = response.bytes().await?;
        serde_cbor::from_slice(&bytes).map_err(AgentError::from)
    }

    async fn submit(&self, request: replica_api::AsyncContent) -> Result<RequestId, AgentError> {
        let request_id = to_request_id(&request)?;
        let signature = self.signer.sign(&request_id)?;

        // We need to calculate the signature, and thus also the
        // request id initially.
        let request = replica_api::AsyncRequest {
            signatures: vec![replica_api::Signatures0 {
                sender_pubkey: signature.public_key.as_slice().to_vec(),
                sender_sig: signature.signature.as_slice().to_vec(),
            }],
            content: request,
        };

        // Clippy doesn't like when return values are not used. We use the error part of the
        // Result.
        let _ = self.execute(self.url.join("submit")?, request).await?;
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
        self.read::<replica_api::QueryResponse>(replica_api::SyncContent::QueryRequest {
            sender: self.signer.sender().as_ref().to_vec(),
            canister_id: canister_id.as_bytes().to_vec(),
            method_name: method_name.to_string(),
            arg: arg.clone().as_slice().to_vec(),
        })
        .await
        .and_then(|response| match response {
            replica_api::QueryResponse::Replied { reply } => Ok(Blob::from(reply.arg)),
            replica_api::QueryResponse::Rejected {
                reject_code,
                reject_message,
            } => Err(AgentError::ClientError(reject_code, reject_message)),
        })
    }

    pub async fn request_status(
        &self,
        request_id: &RequestId,
    ) -> Result<RequestStatusResponse, AgentError> {
        self.read(replica_api::SyncContent::RequestStatusRequest {
            request_id: request_id.to_vec(),
        })
        .await
        .map(|response| match response {
            replica_api::RequestStatusResponse::Replied { reply } => {
                let reply = match reply {
                    replica_api::RequestStatusResponseReplied::CallReply(reply) => {
                        Replied::CallReplied(Blob::from(&reply.arg))
                    }
                    replica_api::RequestStatusResponseReplied::InstallCodeReply(_) => {
                        Replied::InstallCodeReplied
                    }
                    replica_api::RequestStatusResponseReplied::CreateCanisterReply(reply) => {
                        Replied::CreateCanisterReply(CanisterId::from_bytes(reply.canister_id))
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

    /// Request a status and return the response, but wait if the response is Unknown or
    /// Pending, and will return a [AgentError::ClientError] if the call is rejected. If
    /// the request is Replied, it will unpack the reply and return it.
    ///
    /// This is the same result as [request_status], with the exception that it is guaranteed to
    /// only return [RequestStatusReponse::Replied] or and Err(AgentError).
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
                } => return Err(AgentError::ClientError(reject_code, reject_message)),
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
        self.submit(replica_api::AsyncContent::CallRequest {
            canister_id: canister_id.as_bytes().to_vec(),
            method_name: method_name.into(),
            arg: arg.as_slice().to_vec(),
            nonce: self.nonce_factory.generate().map(|b| b.as_slice().to_vec()),
            sender: self.signer.sender().as_ref().to_vec(),
        })
        .await
    }

    pub async fn install(
        &self,
        canister_id: &CanisterId,
        module: &Blob,
        arg: &Blob,
    ) -> Result<RequestId, AgentError> {
        self.install_with_attrs(canister_id, module, arg, &CanisterAttributes::default())
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
        module: &Blob,
        arg: &Blob,
        attributes: &CanisterAttributes,
    ) -> Result<RequestId, AgentError> {
        self.submit(replica_api::AsyncContent::InstallCodeRequest {
            canister_id: canister_id.as_bytes().to_vec(),
            module: module.as_slice().to_vec(),
            arg: arg.as_slice().to_vec(),
            nonce: self.nonce_factory.generate().map(|b| b.as_slice().to_vec()),
            compute_allocation: Some(attributes.compute_allocation.0),
            memory_allocation: None,
            sender: vec![],
            mode: None,
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
