use candid::{CandidType, Int, Nat};
use ic_utils::Canister;
use ic_utils::call::SyncCall;
use num_traits::ToPrimitive;
use serde::Deserialize;
use slog::{Logger, info};
use time::{OffsetDateTime, format_description};

pub async fn list(canister: &Canister<'_>, logger: &Logger) -> anyhow::Result<()> {
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
    struct ListRequest {
        start: Option<Nat>,
        length: Option<Nat>,
    }

    let mut all_entries = Vec::new();
    let mut start = 0u64;
    let mut prev_page_size: Option<usize> = None;

    // Fetch assets in pages until we get 0 items or fewer items than the previous page
    loop {
        let (entries,): (Vec<ListEntry>,) = canister
            .query("list")
            .with_arg(ListRequest {
                start: Some(Nat::from(start)),
                length: None,
            })
            .build()
            .call()
            .await?;

        let num_entries = entries.len();
        if num_entries == 0 {
            break;
        }

        start += num_entries as u64;
        all_entries.extend(entries);

        // If we got fewer items than the previous page, we've reached the end
        if let Some(prev_size) = prev_page_size {
            if num_entries < prev_size {
                break;
            }
        }
        prev_page_size = Some(num_entries);
    }

    for entry in all_entries {
        for encoding in entry.encodings {
            let modified = encoding.modified;
            let modified =
                OffsetDateTime::from_unix_timestamp_nanos(modified.0.to_i128().unwrap())?;
            let timestamp_format =
                format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")?;

            info!(
                logger,
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
