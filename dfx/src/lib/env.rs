use crate::lib::api_client::{Client};

#[derive(Default)]
pub struct Env {
    pub client: Client,
}

impl Env {
    pub fn new(client: Client) -> Env {
        Env {
            client,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
