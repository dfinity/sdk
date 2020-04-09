use crate::Blob;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
/// The response of /api/v1/read with "request_status" request type
pub enum RequestStatusResponse {
    Unknown,
    Pending,
    Replied {
        reply: Replied,
    },
    Rejected {
        reject_code: u16,
        reject_message: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Replied {
    CodeCallReplied { arg: Blob },
    Empty {},
}
