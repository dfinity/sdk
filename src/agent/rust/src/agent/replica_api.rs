use crate::{Blob, CanisterId, RequestId};
use serde::{Deserialize, Serialize};

// pub type Principal = Vec<u8>;
// pub type Pubkey = Vec<u8>;
// pub type Signature = Vec<u8>;
//
// #[derive(Debug, Clone)]
// pub struct SenderSignature {
//     sender_pubkey: Pubkey,
//     sender_sig: Signature,
// }
//
// #[derive(Debug, Clone)]
// pub struct Envelope<T: Serialize> {
//     content: T,
//     signatures: Vec<SenderSignature>,
// }
//
// pub type AsyncRequest = Envelope<AsyncContent>;
//
// #[derive(Debug, Clone, Serialize)]
// #[serde(tag = "request_type", rename_all = "snake_case")]
// pub enum AsyncContent {
//     CreateCanister {
//         nonce: Vec<u8>,
//         sender: Principal,
//
//         #[serde(skip_serializing_if = "Option::is_none")]
//         desired_id: Option<Principal>,
//     },
//     InstallCode {
//         nonce: Vec<u8>,
//         sender: Principal,
//         canister_id: Principal,
//         module: Vec<u8>,
//         arg: Vec<u8>,
//     },
// }

/// Request payloads for the /api/v1/read endpoint.
/// This never needs to be deserialized.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
pub enum ReadRequest<'a> {
    Query {
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
    },
    RequestStatus {
        request_id: &'a RequestId,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct QueryResponseReply {
    pub arg: Blob,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub(crate) enum ReadResponse {
    Replied {
        reply: QueryResponseReply,
    },
    Rejected {
        reject_code: u16,
        reject_message: String,
    },
    Pending,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
pub enum SubmitRequest<'a> {
    InstallCode {
        canister_id: &'a CanisterId,
        module: &'a Blob,
        arg: &'a Blob,
        nonce: &'a Option<Blob>,
        compute_allocation: u8,
    },
    Call {
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
        nonce: &'a Option<Blob>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Request<'a> {
    Submit(SubmitRequest<'a>),
    Query(ReadRequest<'a>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageWithSender<T: Serialize> {
    #[serde(flatten)]
    pub request: T,
    pub sender: Blob,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SignedMessage<'a> {
    #[serde(rename = "content")]
    pub request_with_sender: Request<'a>,
    pub sender_pubkey: Blob,
    #[serde(rename = "sender_sig")]
    pub signature: Blob,
}
