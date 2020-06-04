use crate::{Blob, CanisterId, Principal, RequestId};
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
        sender: &'a Principal,
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
        sender: &'a Principal,
        #[serde(skip_serializing_if = "Option::is_none")]
        compute_allocation: Option<u8>,
    },
    Call {
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
        sender: &'a Principal,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: &'a Option<Blob>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Envelope<T: Serialize> {
    pub content: T,
    pub sender_pubkey: Blob,
    pub sender_sig: Blob,
}

pub type AsyncRequest = Envelope<AsyncContent>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InstallCodeRequestMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "replace")]
    Replace,
    #[serde(rename = "upgrade")]
    Upgrade,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum AsyncContent {
    #[serde(rename = "create_canister")]
    CreateCanisterRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Blob>,
        sender: Principal,
        #[serde(skip_serializing_if = "Option::is_none")]
        desired_id: Option<Principal>,
    },
    #[serde(rename = "install_code")]
    InstallCodeRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Blob>,
        sender: Principal,
        canister_id: Principal,
        module: Blob,
        arg: Blob,
        #[serde(skip_serializing_if = "Option::is_none")]
        compute_allocation: Option<u8>,
        // #[serde(skip_serializing_if = "Option::is_none")]
        // memory_allocation: Option<u64>,
        // #[serde(skip_serializing_if = "Option::is_none")]
        // mode: Option<InstallCodeRequestMode>,
    },
    #[serde(rename = "call")]
    CallRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Blob>,
        sender: Principal,
        canister_id: Principal,
        // canister_id: Principal,
        method_name: String,
        arg: Blob,
    },
    // #[serde(rename = "set_controller")]
    // SetControllerRequest {
    //     #[serde(skip_serializing_if = "Option::is_none")]
    //     nonce: &'a Option<Blob>,
    //     sender: &'a Principal,
    //     canister_id: &'a CanisterId,
    //     controller: Principal,
    // },
}

pub type SyncRequest = Envelope<SyncContent>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum SyncContent {
    #[serde(rename = "request_status")]
    // RequestStatusRequest { request_id: Bytes },
    RequestStatusRequest { request_id: Blob },
    #[serde(rename = "query")]
    QueryRequest {
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Blob,
    },
}

pub type Response = ResponseContent;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ResponseContent {
    RequestStatusResponse(RequestStatusResponse),
    QueryResponse(QueryResponse),
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
    CreateCanisterReply(CreateCanisterReply),
    InstallCodeReply(InstallCodeReply),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateCanisterReply {
    // todo change
    // pub canister_id: Principal,
    pub canister_id: CanisterId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallCodeReply {}

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