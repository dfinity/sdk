use actix_web::client::Client;
use ic_agent::agent::ReplicaV1Transport;
use ic_agent::{AgentError, RequestId};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct ActixWebClientHttpTransport<'a> {
    url: url::Url,
    client: &'a Client,
}

impl<'a> ActixWebClientHttpTransport<'a> {
    pub fn create(client: &Client, url: String) -> Result<Self, AgentError> {
        // let mut tls_config = rustls::ClientConfig::new();

        // Advertise support for HTTP/2
        // tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        // Mozilla CA root store
        // tls_config
        //     .root_store
        //     .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        Ok(Self {
            client,
            url: url::Url::parse(&url).map_err(|_| AgentError::InvalidReplicaUrl(url))?,
        })
    }
}

impl<'b> ReplicaV1Transport for ActixWebClientHttpTransport<'b> {
    fn read<'a>(
        &'a self,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send>> {
        async fn run(
            t: &ActixWebClientHttpTransport<'_>,
            envelope: Vec<u8>,
        ) -> Result<Vec<u8>, AgentError> {
            let mut response = t
                .client
                .post(t.url.join("/").unwrap())
                .send_body(envelope)
                .await
                .map_err(|err| AgentError::TransportError(Box::new(err)))?;

            Ok(response.body().await?.to_vec())
        }

        Box::pin(run(self, envelope))
    }

    fn submit<'a>(
        &'a self,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send>> {
        unimplemented!()
    }

    fn status<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send>> {
        unimplemented!()
    }
}
