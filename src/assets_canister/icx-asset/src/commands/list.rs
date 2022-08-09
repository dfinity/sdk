use crate::support::Result;
use candid::{CandidType, Int, Nat};
use ic_utils::call::SyncCall;
use ic_utils::Canister;

use num_traits::ToPrimitive;
use serde::Deserialize;
use time::{format_description, OffsetDateTime};

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

    for entry in entries {
        for encoding in entry.encodings {
            let modified = encoding.modified;
            let modified =
                OffsetDateTime::from_unix_timestamp_nanos(modified.0.to_i128().unwrap())?;
            let timestamp_format =
                format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")?;

            eprintln!(
                "{:>20} {:>15} {:50} ({}, {})",
                modified.format(&timestamp_format)?,
                encoding.length.0,
                entry.key,
                entry.content_type,
                encoding.content_encoding
            );
        }
    }
    Ok(())
}
