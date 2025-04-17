use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{duration_parser, timestamp_parser};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::{CanisterLogRecord, FetchCanisterLogsResponse};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Get the canister logs.
#[derive(Parser)]
pub struct LogsOpts {
    /// Specifies the name or id of the canister to get the logs of.
    canister: String,

    /// Specifies to show the last number of the logs.
    #[arg(long)]
    tail: Option<u64>,

    /// Specifies to show the logs newer than a relative duration, with the valid units 's', 'm', 'h', 'd'.
    #[arg(long, conflicts_with("tail"), conflicts_with("since_time"), conflicts_with("follow"), value_parser = duration_parser)]
    since: Option<u64>,

    /// Specifies to show the logs newer than a specific timestamp.
    /// Required either nanoseconds since Unix epoch or RFC3339 format (e.g. '2021-05-06T19:17:10.000000002Z').
    #[arg(long, conflicts_with("tail"), conflicts_with("since"), conflicts_with("follow"), value_parser = timestamp_parser)]
    since_time: Option<u64>,

    /// Specifies to fetch logs continuously until interrupted with Ctrl+C.
    #[arg(
        long,
        conflicts_with("tail"),
        conflicts_with("since"),
        conflicts_with("since_time")
    )]
    follow: bool,

    /// Specifies the interval in seconds between log fetches when following logs. Defaults to 2 seconds.
    #[arg(long, requires("follow"))]
    interval: Option<u64>,
}

struct FilterOpts {
    tail: Option<u64>,
    since: Option<u64>,
    since_time: Option<u64>,
    last_idx: Option<u64>,
}

fn filter_canister_logs<'a>(
    logs: &'a FetchCanisterLogsResponse,
    opts: FilterOpts,
) -> &'a [CanisterLogRecord] {
    if let Some(number) = opts.tail {
        &logs.canister_log_records[logs
            .canister_log_records
            .len()
            .saturating_sub(number as usize)..]
    } else if let Some(since) = opts.since {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
            - since * 1_000_000_000;

        let index = logs
            .canister_log_records
            .partition_point(|r| r.timestamp_nanos <= timestamp_nanos);
        &logs.canister_log_records[index..]
    } else if let Some(since_time) = opts.since_time {
        let index = logs
            .canister_log_records
            .partition_point(|r| r.timestamp_nanos <= since_time);
        &logs.canister_log_records[index..]
    } else if let Some(last_idx) = opts.last_idx {
        let index = logs
            .canister_log_records
            .partition_point(|r| r.idx <= last_idx);
        &logs.canister_log_records[index..]
    } else {
        &logs.canister_log_records
    }
}

fn format_bytes(bytes: &[u8]) -> String {
    format!("(bytes) 0x{}", hex::encode(bytes))
}

fn format_canister_logs(logs: &[CanisterLogRecord]) -> Vec<String> {
    logs.iter()
        .map(|r| {
            let time = OffsetDateTime::from_unix_timestamp_nanos(r.timestamp_nanos as i128)
                .expect("Invalid canister log record timestamp");

            let message = if let Ok(s) = String::from_utf8(r.content.clone()) {
                if format!("{s:?}").contains("\\u{") {
                    format_bytes(&r.content)
                } else {
                    s
                }
            } else {
                format_bytes(&r.content)
            };

            format!(
                "[{}. {}]: {}",
                r.idx,
                time.format(&Rfc3339).expect("Failed to format timestamp"),
                message
            )
        })
        .collect()
}

pub async fn exec(env: &dyn Environment, opts: LogsOpts, call_sender: &CallSender) -> DfxResult {
    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    if opts.follow {
        let interval = opts.interval.unwrap_or(2);
        let mut last_idx = 0u64;

        loop {
            let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;
            let filter_opts = FilterOpts {
                tail: None,
                since: None,
                since_time: None,
                last_idx: Some(last_idx),
            };
            let filtered_logs = filter_canister_logs(&logs, filter_opts);

            if !filtered_logs.is_empty() {
                println!("{}", format_canister_logs(filtered_logs).join("\n"));
                last_idx = filtered_logs.last().unwrap().idx;
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(interval)) => continue,
                _ = tokio::signal::ctrl_c() => break,
            }
        }
    } else {
        let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;

        let filter_opts = FilterOpts {
            tail: opts.tail,
            since: opts.since,
            since_time: opts.since_time,
            last_idx: None,
        };
        let filtered_logs = filter_canister_logs(&logs, filter_opts);

        if filtered_logs.is_empty() {
            println!("No logs");
        } else {
            println!("{}", format_canister_logs(filtered_logs).join("\n"));
        }
    }

    Ok(())
}

#[test]
fn test_format_canister_logs() {
    let logs = FetchCanisterLogsResponse {
        canister_log_records: vec![
            CanisterLogRecord {
                idx: 42,
                timestamp_nanos: 1_620_328_630_000_000_001,
                content: b"Some text message".to_vec(),
            },
            CanisterLogRecord {
                idx: 43,
                timestamp_nanos: 1_620_328_630_000_000_002,
                content: vec![192, 255, 238],
            },
            CanisterLogRecord {
                idx: 44,
                timestamp_nanos: 1_620_328_630_000_000_003,
                content: vec![192, 255, 238],
            },
            CanisterLogRecord {
                idx: 45,
                timestamp_nanos: 1_620_328_635_000_000_001,
                content: b"Five seconds later".to_vec(),
            },
            CanisterLogRecord {
                idx: 46,
                timestamp_nanos: 1_620_328_635_000_000_002,
                content: vec![192, 255, 238],
            },
            CanisterLogRecord {
                idx: 47,
                timestamp_nanos: 1_620_328_635_000_000_003,
                content: vec![192, 255, 238],
            },
        ],
    };

    let filtered_logs = filter_canister_logs(
        &logs,
        FilterOpts {
            tail: None,
            since: None,
            since_time: None,
            last_idx: None,
        },
    );
    assert_eq!(
        format_canister_logs(filtered_logs),
        vec![
            "[42. 2021-05-06T19:17:10.000000001Z]: Some text message".to_string(),
            "[43. 2021-05-06T19:17:10.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[44. 2021-05-06T19:17:10.000000003Z]: (bytes) 0xc0ffee".to_string(),
            "[45. 2021-05-06T19:17:15.000000001Z]: Five seconds later".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test tail
    let filtered_logs = filter_canister_logs(
        &logs,
        FilterOpts {
            tail: Some(4),
            since: None,
            since_time: None,
            last_idx: None,
        },
    );
    assert_eq!(
        format_canister_logs(filtered_logs),
        vec![
            "[44. 2021-05-06T19:17:10.000000003Z]: (bytes) 0xc0ffee".to_string(),
            "[45. 2021-05-06T19:17:15.000000001Z]: Five seconds later".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test since
    let duration_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 1620328631; // 1 second after the first log with idx 42.
    let filtered_logs = filter_canister_logs(
        &logs,
        FilterOpts {
            tail: None,
            since: Some(duration_seconds),
            since_time: None,
            last_idx: None,
        },
    );
    assert_eq!(
        format_canister_logs(filtered_logs),
        vec![
            "[45. 2021-05-06T19:17:15.000000001Z]: Five seconds later".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test since_time
    let filtered_logs = filter_canister_logs(
        &logs,
        FilterOpts {
            tail: None,
            since: None,
            since_time: Some(1_620_328_635_000_000_000),
            last_idx: None,
        },
    );
    assert_eq!(
        format_canister_logs(filtered_logs),
        vec![
            "[45. 2021-05-06T19:17:15.000000001Z]: Five seconds later".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test last index
    let filtered_logs = filter_canister_logs(
        &logs,
        FilterOpts {
            tail: None,
            since: None,
            since_time: None,
            last_idx: Some(44),
        },
    );
    assert_eq!(
        format_canister_logs(filtered_logs),
        vec![
            "[45. 2021-05-06T19:17:15.000000001Z]: Five seconds later".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );
}
