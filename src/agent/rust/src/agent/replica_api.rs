//! This file is generated from requests.cddl.
#![allow(dead_code)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Bytes(pub Vec<u8>);
impl<'a> Deserialize<'a> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'a>>::Error>
    where
        D: Deserializer<'a>,
    {
        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = Bytes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a byte string (bytes)")
            }

            fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(Bytes(value.to_vec()))
            }
        }

        deserializer.deserialize_bytes(BytesVisitor)
    }
}
impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.as_slice())
    }
}
impl<T: AsRef<[u8]>> From<T> for Bytes {
    fn from(bytes: T) -> Self {
        Self(bytes.as_ref().to_vec())
    }
}

pub type Principal = Bytes;
pub type Pubkey = Bytes;
pub type Signature = Bytes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signatures0 {
    pub sender_pubkey: Pubkey,
    pub sender_sig: Signature,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Envelope<T: Serialize> {
    Envelope {
        sender_pubkey: Pubkey,
        sender_sig: Signature,
        content: T,
    },
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
        nonce: Option<Bytes>,
        sender: Principal,
        #[serde(skip_serializing_if = "Option::is_none")]
        desired_id: Option<Principal>,
    },
    #[serde(rename = "install_code")]
    InstallCodeRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Bytes>,
        sender: Principal,
        canister_id: Principal,
        module: Bytes,
        arg: Bytes,
        #[serde(skip_serializing_if = "Option::is_none")]
        compute_allocation: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        memory_allocation: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<InstallCodeRequestMode>,
    },
    #[serde(rename = "set_controller")]
    SetControllerRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Bytes>,
        sender: Principal,
        canister_id: Principal,
        controller: Principal,
    },
    #[serde(rename = "call")]
    CallRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<Bytes>,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Bytes,
    },
}

pub type SyncRequest = Envelope<SyncContent>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum SyncContent {
    #[serde(rename = "request_status")]
    RequestStatusRequest { request_id: Bytes },
    #[serde(rename = "query")]
    QueryRequest {
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Bytes,
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
    pub arg: Bytes,
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
