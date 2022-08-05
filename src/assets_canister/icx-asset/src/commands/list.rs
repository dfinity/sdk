use crate::support::Result;
use candid::{CandidType, Int, Nat};
use ic_utils::call::SyncCall;
use ic_utils::Canister;

use num_traits::ToPrimitive;
use serde::Deserialize;
use std::time::SystemTime;

pub async fn list(canister: &Canister<'_>) -> Result {
    #[derive(CandidType, Deserialize)]
    struct Encoding {
        modified: Int,
        content_encoding: String,
        sha256: Option<Vec<u8>>,
        length: Nat,
    }

    #[derive(CandidType, Deserialize)]
    struct ListEntry {
        key: String,
        content_type: String,
        encodings: Vec<Encoding>,
    }

    #[derive(CandidType, Deserialize)]
    struct EmptyRecord {}

    let (entries,): (Vec<ListEntry>,) = canister
        .query_("list")
        .with_arg(EmptyRecord {})
        .build()
        .call()
        .await?;

    use chrono::offset::Local;
    use chrono::DateTime;

    for entry in entries {
        for encoding in entry.encodings {
            let modified = encoding.modified;
            let modified = SystemTime::UNIX_EPOCH
                + std::time::Duration::from_nanos(modified.0.to_u64().unwrap());

            eprintln!(
                "{:>20} {:>15} {:50} ({}, {})",
                DateTime::<Local>::from(modified).format("%F %X"),
                encoding.length.0,
                entry.key,
                entry.content_type,
                encoding.content_encoding
            );
        }
    }
    Ok(())
}
