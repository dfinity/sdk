use crate::{Blob, CanisterId, RequestId};
use serde::{Deserialize, Serialize};

pub type Principal = Blob;
pub type Pubkey = Blob;
pub type Signature = Blob;

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
        sender: Principal,
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
#[serde(flatten)]
pub struct MessageWithSender<T: Serialize> {
    pub content: T,
    pub sender: Prin,
}

#[derive(Debug, Clone, Serialize)]
#[serde(flatten)]
pub struct SenderSignature {
    pub sender_pubkey: Pubkey,
    pub sender_sig: Signature,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SignedMessage<'a> {
    #[serde(flatten)]
    pub content: {
    Request
}<'a>,
    pub signatures: Vec<SenderSignature>,
}
