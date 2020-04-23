use crate::agent::signer::{DummyIdentity, Signer};
use crate::NonceFactory;

pub struct AgentConfig<'a> {
    pub url: &'a str,
    pub nonce_factory: NonceFactory,
    pub signer: Box<dyn Signer>,
}

impl Default for AgentConfig<'_> {
    fn default() -> Self {
        Self {
            // Making sure this is invalid so users have to overwrite it.
            url: "-",
            nonce_factory: NonceFactory::random(),
            signer: Box::new(DummyIdentity {}),
        }
    }
}
