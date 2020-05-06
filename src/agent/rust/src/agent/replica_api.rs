use crate::{Blob, CanisterId, RequestId};
use serde::{Deserialize, Serialize};

/// Request payloads for the /api/v1/read endpoint.
/// This never needs to be deserialized.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "mode")]
pub enum Mode {
   Install,
   Upgrade,
   Replace,
}

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
        #[serde(skip_serializing_if = "Option::is_none")]
        compute_allocation: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<& 'a str>,
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
