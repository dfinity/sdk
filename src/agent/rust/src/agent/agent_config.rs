use crate::identity::dummy::DummyIdentity;
use crate::identity::Identity;
use crate::NonceFactory;

pub struct AgentConfig<'a> {
    pub url: &'a str,
    pub nonce_factory: NonceFactory,
    pub identity: Box<dyn Identity>,
}

impl Default for AgentConfig<'_> {
    fn default() -> Self {
        Self {
            // Making sure this is invalid so users have to overwrite it.
            url: "-",
            nonce_factory: NonceFactory::random(),
            identity: Box::new(DummyIdentity {}),
        }
    }
}
