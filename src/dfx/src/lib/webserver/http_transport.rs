use ic_agent::agent::agent_error::HttpErrorPayload;
use ic_agent::AgentError;
use ic_agent::RequestId;
use reqwest::Method;
use std::future::Future;
use std::pin::Pin;

/// A [ReplicaV1Transport] using Reqwest to make HTTP calls to the internet computer.
pub struct ReqwestHttpReplicaV1Transport {
    url: reqwest::Url,
    client: reqwest::Client,
}

impl ReqwestHttpReplicaV1Transport {
    pub fn create<U: Into<String>>(url: U) -> Result<Self, AgentError> {
        let mut tls_config = rustls::ClientConfig::new();

        // Advertise support for HTTP/2
        tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        // Mozilla CA root store
        tls_config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let url = url.into();

        Ok(Self {
            url: reqwest::Url::parse(&url)
                .and_then(|url| url.join("api/v1/"))
                .map_err(|_| AgentError::InvalidReplicaUrl(url.clone()))?,
            client: reqwest::Client::builder()
                .use_preconfigured_tls(tls_config)
                .build()
                .expect("Could not create HTTP client."),
        })
    }

    async fn request(
        &self,
        http_request: reqwest::Request,
    ) -> Result<(reqwest::StatusCode, reqwest::header::HeaderMap, Vec<u8>), AgentError> {
        let response = self
            .client
            .execute(
                http_request
                    .try_clone()
                    .expect("Could not clone a request."),
            )
            .await
            .map_err(|x| AgentError::TransportError(Box::new(x)))?;

        let http_status = response.status();
        let response_headers = response.headers().clone();
        let bytes = response
            .bytes()
            .await
            .map_err(|x| AgentError::TransportError(Box::new(x)))?
            .to_vec();

        Ok((http_status, response_headers, bytes))
    }

    async fn execute(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, AgentError> {
        let url = self.url.join(endpoint)?;
        let mut http_request = reqwest::Request::new(method, url);
        http_request.headers_mut().insert(
            reqwest::header::CONTENT_TYPE,
            "application/cbor".parse().unwrap(),
        );

        *http_request.body_mut() = body.map(reqwest::Body::from);

        let (status, headers, body) = self.request(http_request.try_clone().unwrap()).await?;

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AgentError::CannotUseAuthenticationOnNonSecureUrl());
        } else if status.is_client_error() || status.is_server_error() {
            Err(AgentError::HttpError(HttpErrorPayload {
                status: status.into(),
                content_type: headers
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(|x| x.to_string()),
                content: body,
            }))
        } else {
            Ok(body)
        }
    }
}

impl ic_agent::agent::ReplicaV1Transport for ReqwestHttpReplicaV1Transport {
    fn read<'a>(
        &'a self,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(
            s: &ReqwestHttpReplicaV1Transport,
            envelope: Vec<u8>,
        ) -> Result<Vec<u8>, AgentError> {
            s.execute(Method::POST, "read", Some(envelope)).await
        }

        Box::pin(run(self, envelope))
    }

    fn submit<'a>(
        &'a self,
        envelope: Vec<u8>,
        _request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        async fn run(
            s: &ReqwestHttpReplicaV1Transport,
            envelope: Vec<u8>,
        ) -> Result<(), AgentError> {
            s.execute(Method::POST, "submit", Some(envelope)).await?;
            Ok(())
        }
        Box::pin(run(self, envelope))
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &ReqwestHttpReplicaV1Transport) -> Result<Vec<u8>, AgentError> {
            s.execute(Method::GET, "status", None).await
        }

        Box::pin(run(self))
    }
}
