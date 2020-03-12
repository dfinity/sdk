use crate::{Blob, CanisterId, PrincipalId, RequestId};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct QueryResponseReply {
    pub arg: Blob,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub(crate) enum ReadResponse<A> {
    Replied {
        reply: Option<A>,
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
    Read(ReadRequest<'a>),
    Submit(SubmitRequest<'a>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageWithSender<'a> {
    #[serde(flatten)]
    pub request: Request<'a>,
    pub sender: PrincipalId,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SignedMessage<'a> {
    #[serde(flatten)]
    pub request_with_sender: MessageWithSender<'a>,
    pub sender_pubkey: Blob,
    pub sender_sig: Blob,
}
