pub(crate) mod agent_config;
pub(crate) mod agent_error;
pub(crate) mod nonce;
pub(crate) mod replica_api;
pub(crate) mod response;

pub(crate) mod public {
    pub use super::agent_config::AgentConfig;
    pub use super::agent_error::AgentError;
    pub use super::nonce::NonceFactory;
    pub use super::replica_api::{MessageWithSender, SignedMessage};
    pub use super::response::RequestStatusResponse;
    pub use super::signer::Signer;
    pub use super::Agent;
}

#[cfg(test)]
mod agent_test;

pub mod signer;

use crate::agent::replica_api::{QueryResponseReply, ReadRequest, ReadResponse, SubmitRequest};
use crate::agent::signer::Signer;
use crate::{Blob, CanisterAttributes, CanisterId, RequestId};

use public::*;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};

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
            url: reqwest::Url::parse(url)
                .and_then(|url| url.join("api/v1/"))
                .map_err(|_| AgentError::InvalidClientUrl(String::from(url)))?,
            client: Client::builder().default_headers(default_headers).build()?,
            nonce_factory: config.nonce_factory,
            signer: config.signer,
        })
    }

    async fn read<A>(&self, request: ReadRequest<'_>) -> Result<ReadResponse<A>, AgentError>
    where
        A: serde::de::DeserializeOwned,
    {
        let record = serde_cbor::to_vec(&request)?;
        let url = self.url.join("read")?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(record));

        let bytes = self
            .client
            .execute(http_request)
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        serde_cbor::from_slice(&bytes).map_err(AgentError::InvalidCborData)
    }

    async fn submit(&self, request: SubmitRequest<'_>) -> Result<RequestId, AgentError> {
        // We need to calculate the signature, and thus also the
        // request id initially.
        let request: Box<SubmitRequest<'_>> = Box::new(request);
        let (request_id, signed_request) = self.signer.sign(request)?;

        let record = serde_cbor::to_vec(&signed_request)?;
        let url = self.url.join("submit")?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(record));

        // Clippy doesn't like when return values are not used.
        let _ = self
            .client
            .execute(http_request)
            .await?
            .error_for_status()?;

        Ok(request_id)
    }

    /// The simplest for of query; sends a Blob and will return a Blob. The encoding is
    /// left as an exercise to the user.
    pub async fn query<'a>(
        &self,
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
    ) -> Result<Option<Blob>, AgentError> {
        self.read::<QueryResponseReply>(ReadRequest::Query {
            canister_id,
            method_name,
            arg,
        })
        .await
        .and_then(|response| match response {
            ReadResponse::Replied { reply } => Ok(reply.map(|r| r.arg)),
            ReadResponse::Rejected {
                reject_code,
                reject_message,
            } => Err(AgentError::ClientError(reject_code, reject_message)),
            ReadResponse::Unknown => Err(AgentError::InvalidClientResponse),
            ReadResponse::Pending => Err(AgentError::InvalidClientResponse),
        })
    }

    pub async fn request_status(
        &self,
        request_id: &RequestId,
    ) -> Result<RequestStatusResponse, AgentError> {
        self.read(ReadRequest::RequestStatus { request_id })
            .await
            .and_then(|response| Ok(RequestStatusResponse::from(response)))
    }

    pub async fn request_status_and_wait<W: delay::Waiter>(
        &self,
        request_id: &RequestId,
        mut waiter: W,
    ) -> Result<Option<Blob>, AgentError> {
        waiter.start();

        loop {
            match self.request_status(request_id).await? {
                RequestStatusResponse::Replied { reply } => return Ok(reply),
                RequestStatusResponse::Rejected { code, message } => {
                    return Err(AgentError::ClientError(code, message))
                }
                RequestStatusResponse::Unknown => (),
                RequestStatusResponse::Pending => (),
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
    ) -> Result<Option<Blob>, AgentError> {
        let request_id = self.call(canister_id, method_name, arg).await?;
        self.request_status_and_wait(&request_id, waiter).await
    }

    pub async fn call(
        &self,
        canister_id: &CanisterId,
        method_name: &str,
        arg: &Blob,
    ) -> Result<RequestId, AgentError> {
        self.submit(SubmitRequest::Call {
            canister_id,
            method_name,
            arg,
            nonce: &self.nonce_factory.generate(),
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
    ) -> Result<Option<Blob>, AgentError> {
        let request_id = self.install(canister_id, module, arg).await?;
        self.request_status_and_wait(&request_id, waiter).await
    }

    pub async fn install_with_attrs(
        &self,
        canister_id: &CanisterId,
        module: &Blob,
        arg: &Blob,
        attributes: &CanisterAttributes,
    ) -> Result<RequestId, AgentError> {
        self.submit(SubmitRequest::InstallCode {
            canister_id,
            module,
            arg,
            nonce: &self.nonce_factory.generate(),
            compute_allocation: attributes.compute_allocation.0,
        })
        .await
    }

    pub async fn ping_once(&self) -> Result<(), AgentError> {
        let url = self.url.join("read")?;
        let http_request = reqwest::Request::new(Method::GET, url);
        let response = self.client.execute(http_request).await?;

        if response.status().as_u16() == 405 {
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
