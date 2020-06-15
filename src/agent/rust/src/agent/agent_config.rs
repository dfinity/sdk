use crate::identity::dummy::DummyIdentity;
use crate::identity::Identity;
use crate::{AgentError, NonceFactory};
use async_trait::async_trait;

#[async_trait]
pub trait AgentRequestExecutor: Sync + Send {
    async fn execute(&self, request: reqwest::Request) -> Result<reqwest::Response, AgentError>;
}

pub struct DefaultAgentClient {
    client: reqwest::Client,
}

#[async_trait]
impl AgentRequestExecutor for DefaultAgentClient {
    async fn execute(&self, request: reqwest::Request) -> Result<reqwest::Response, AgentError> {
        self.client.execute(request).await.map_err(AgentError::from)
    }
}

pub struct AgentConfig<'a> {
    pub url: &'a str,
    pub nonce_factory: NonceFactory,
    pub identity: Box<dyn Identity>,
    pub request_executor: Box<dyn AgentRequestExecutor>,
}

impl Default for AgentConfig<'_> {
    fn default() -> Self {
        Self {
            // Making sure this is invalid so users have to overwrite it.
            url: "-",
            nonce_factory: NonceFactory::random(),
            identity: Box::new(DummyIdentity {}),
            request_executor: Box::new(DefaultAgentClient {
                client: reqwest::Client::builder()
                    .build()
                    .expect("Could not create HTTP client."),
            }),
        }
    }
}
