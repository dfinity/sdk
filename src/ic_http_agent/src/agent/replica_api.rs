use crate::{Blob, CanisterId, RequestId};
use serde::{Deserialize, Serialize};

/// Request payloads for the /api/v1/read endpoint.
/// This never needs to be deserialized.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
pub(crate) enum ReadRequest<'a> {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
pub(crate) enum SubmitRequest<'a> {
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
