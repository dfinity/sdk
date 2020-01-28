use crate::NonceFactory;

pub struct AgentConfig<'a> {
    pub url: &'a str,
    pub nonce_factory: NonceFactory,
}

impl Default for AgentConfig<'_> {
    fn default() -> Self {
        Self {
            url: "-", // Making sure this is invalid so users have to overwrite it.
            nonce_factory: NonceFactory::random(),
        }
    }
}
