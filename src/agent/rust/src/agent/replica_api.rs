use crate::{Blob, CanisterId, Principal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Envelope<T: Serialize> {
    pub content: T,
    pub sender_pubkey: Blob,
    pub sender_sig: Blob,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum AsyncContent {
    #[serde(rename = "call")]
    CallRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Blob>,
        sender: Principal,
        canister_id: CanisterId,
        method_name: String,
        arg: Blob,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum SyncContent {
    #[serde(rename = "request_status")]
    RequestStatusRequest { request_id: Blob },
    #[serde(rename = "query")]
    QueryRequest {
        sender: Principal,
        canister_id: CanisterId,
        method_name: String,
        arg: Blob,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status")]
pub enum RequestStatusResponse {
    #[serde(rename = "unknown")]
    Unknown {},
    #[serde(rename = "received")]
    Received {},
    #[serde(rename = "processing")]
    Processing {},
    #[serde(rename = "replied")]
    Replied { reply: RequestStatusResponseReplied },
    #[serde(rename = "rejected")]
    Rejected {
        reject_code: u64,
        reject_message: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequestStatusResponseReplied {
    CallReply(CallReply),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallReply {
    pub arg: Blob,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status")]
pub enum QueryResponse {
    #[serde(rename = "replied")]
    Replied { reply: CallReply },
    #[serde(rename = "rejected")]
    Rejected {
        reject_code: u64,
        reject_message: String,
    },
}
