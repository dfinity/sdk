//! This file is generated from requests.cddl.
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

pub type Principal = Vec<u8>;
pub type Pubkey = Vec<u8>;
pub type Signature = Vec<u8>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signatures0 {
    pub sender_pubkey: Pubkey,
    pub sender_sig: Signature,
}

#[derive(Debug, Clone, Serialize)]
pub struct Envelope<T: Serialize> {
    pub signatures: Vec<Signatures0>,
    pub content: T,
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
        nonce: Option<Vec<u8>>,
        sender: Principal,
        desired_id: Principal,
    },
    #[serde(rename = "install_code")]
    InstallCodeRequest {
        nonce: Option<Vec<u8>>,
        sender: Principal,
        canister_id: Principal,
        module: Vec<u8>,
        arg: Vec<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        compute_allocation: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        memory_allocation: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<InstallCodeRequestMode>,
    },
    #[serde(rename = "set_controller")]
    SetControllerRequest {
        nonce: Option<Vec<u8>>,
        sender: Principal,
        canister_id: Principal,
        controller: Principal,
    },
    #[serde(rename = "call")]
    CallRequest {
        nonce: Option<Vec<u8>>,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Vec<u8>,
    },
}

pub type SyncRequest = Envelope<SyncContent>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum SyncContent {
    #[serde(rename = "request_status")]
    RequestStatusRequest { request_id: Vec<u8> },
    #[serde(rename = "query")]
    QueryRequest {
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Vec<u8>,
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
    pub canister_id: Principal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallCodeReply {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallReply {
    pub arg: Vec<u8>,
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
