use crate::agent::agent_impl::{QueryResponseReply, ReadResponse};
use crate::Blob;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub enum RequestStatusResponse {
    Replied { reply: Blob },
    Rejected { code: u16, message: String },
    Unknown,
    Pending,
}

impl From<ReadResponse<QueryResponseReply>> for RequestStatusResponse {
    fn from(response: ReadResponse<QueryResponseReply>) -> Self {
        match response {
            ReadResponse::Unknown => RequestStatusResponse::Unknown,
            ReadResponse::Pending => RequestStatusResponse::Pending,
            ReadResponse::Rejected {
                reject_code,
                reject_message,
            } => RequestStatusResponse::Rejected {
                code: reject_code,
                message: reject_message,
            },
            ReadResponse::Replied { reply } => RequestStatusResponse::Replied {
                reply: reply.map(|r| r.arg).unwrap_or_else(Blob::empty),
            },
        }
    }
}
