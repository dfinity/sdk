use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{duration_parser, timestamp_parser};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::FetchCanisterLogsResponse;
use std::time::{SystemTime, UNIX_EPOCH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Get the canister logs.
#[derive(Parser)]
pub struct LogsOpts {
    /// Specifies the name or id of the canister to get its canister information.
    canister: String,

    /// Specifies to show the last N lines of the logs, use '-N' to specify the number of lines.
    #[arg(long)]
    tail: bool,

    /// Specifies the number of logs to show for the '--tail' option. Defaults to 10.
    #[arg(short = 'N', requires("tail"), default_value("10"))]
    lines: Option<u64>,

    /// Specifies to show the logs newer than a relative duration, with the valid units 's', 'm', 'h', 'd'.
    #[arg(long, conflicts_with("tail"), value_parser = duration_parser)]
    since: Option<u64>,

    /// Specifies to show the logs newer than a specific timestamp.
    /// Required either nanoseconds since epoch or RFC3339 format (e.g. '2021-05-06T19:17:10.000000002Z').
    #[arg(long, conflicts_with("tail"), value_parser = timestamp_parser)]
    since_time: Option<u64>,
}

fn format_bytes(bytes: &[u8]) -> String {
    format!("(bytes) 0x{}", hex::encode(bytes))
}

fn format_canister_logs(logs: FetchCanisterLogsResponse, opts: &LogsOpts) -> Vec<String> {
    let filtered_logs = if opts.tail {
        let number = opts.lines.unwrap_or(10);
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
    } else {
        &logs.canister_log_records
    };

    if filtered_logs.is_empty() {
        return vec!["No logs".to_string()];
    }

    filtered_logs
        .iter()
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

#[test]
fn test_format_canister_logs() {
    use ic_utils::interfaces::management_canister::CanisterLogRecord;

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
            // 5 seconds later
            CanisterLogRecord {
                idx: 45,
                timestamp_nanos: 1_620_328_635_000_000_001,
                content: vec![192, 255, 238],
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

    assert_eq!(
        format_canister_logs(
            logs.clone(),
            &LogsOpts {
                canister: "2vxsx-fae".to_string(),
                tail: false,
                lines: None,
                since: None,
                since_time: None,
            }
        ),
        vec![
            "[42. 2021-05-06T19:17:10.000000001Z]: Some text message".to_string(),
            "[43. 2021-05-06T19:17:10.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[44. 2021-05-06T19:17:10.000000003Z]: (bytes) 0xc0ffee".to_string(),
            "[45. 2021-05-06T19:17:15.000000001Z]: (bytes) 0xc0ffee".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test tail
    assert_eq!(
        format_canister_logs(
            logs.clone(),
            &LogsOpts {
                canister: "2vxsx-fae".to_string(),
                tail: true,
                lines: Some(4),
                since: None,
                since_time: None,
            }
        ),
        vec![
            "[44. 2021-05-06T19:17:10.000000003Z]: (bytes) 0xc0ffee".to_string(),
            "[45. 2021-05-06T19:17:15.000000001Z]: (bytes) 0xc0ffee".to_string(),
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
    assert_eq!(
        format_canister_logs(
            logs.clone(),
            &LogsOpts {
                canister: "2vxsx-fae".to_string(),
                tail: false,
                lines: None,
                since: Some(duration_seconds),
                since_time: None,
            }
        ),
        vec![
            "[45. 2021-05-06T19:17:15.000000001Z]: (bytes) 0xc0ffee".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );

    // Test since_time
    assert_eq!(
        format_canister_logs(
            logs,
            &LogsOpts {
                canister: "2vxsx-fae".to_string(),
                tail: false,
                lines: None,
                since: None,
                since_time: Some(1_620_328_635_000_000_000),
            }
        ),
        vec![
            "[45. 2021-05-06T19:17:15.000000001Z]: (bytes) 0xc0ffee".to_string(),
            "[46. 2021-05-06T19:17:15.000000002Z]: (bytes) 0xc0ffee".to_string(),
            "[47. 2021-05-06T19:17:15.000000003Z]: (bytes) 0xc0ffee".to_string(),
        ]
    );
}

pub async fn exec(env: &dyn Environment, opts: LogsOpts, call_sender: &CallSender) -> DfxResult {
    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;
    println!("{}", format_canister_logs(logs, &opts).join("\n"));

    Ok(())
}
