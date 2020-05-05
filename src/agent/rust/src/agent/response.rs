//! Types for the Public API part of the Agent. This should be exposed and returned from methods
//! on the agent. These are not serializable because they're not meant to be sent over the wire.
use crate::{Blob, CanisterId};

/// The response of /api/v1/read with "request_status" request type.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum RequestStatusResponse {
    Unknown,
    Pending,
    Replied {
        reply: Replied,
    },
    Rejected {
        reject_code: u64,
        reject_message: String,
    },
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Replied {
    CallReplied(Blob),
    CreateCanisterReply(CanisterId),
    InstallCodeReplied,
}
