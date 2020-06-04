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
        //change to u64
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
