use ic_agent::RequestId;
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessageV1 {
    version: usize,
    pub call_type: String,
    pub sender: String,
    pub canister_id: String,
    pub method_name: String,
    pub request_id: String,   // only useful for update call
    pub content: String,      // hex::encode the Vec<u8>
}

impl SignedMessageV1 {
    pub fn new(sender: Principal, canister_id: Principal, method_name: String) -> Self {
        Self {
            version: 1,
            call_type: String::new(),
            sender: sender.to_string(),
            canister_id: canister_id.to_string(),
            method_name,
            request_id: String::new(), 
            content: String::new(),
        }
    }

    pub fn with_call_type(mut self, request_type: String) -> Self {
        self.call_type = request_type;
        self
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = String::from(request_id);
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
}
