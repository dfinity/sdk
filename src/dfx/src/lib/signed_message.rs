use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessageV1 {
    version: usize,
    pub request_type: String,
    pub sender: Principal,
    pub canister_id: Principal,
    pub method_name: String,
    pub content: String, // hex::encode the Vec<u8>
}

impl SignedMessageV1 {
    pub fn new(sender: Principal, canister_id: Principal, method_name: String) -> Self {
        Self {
            version: 1,
            request_type: String::new(),
            sender,
            canister_id,
            method_name,
            content: String::new(),
        }
    }

    pub fn with_request_type(mut self, request_type: String) -> Self {
        self.request_type = request_type;
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
}
