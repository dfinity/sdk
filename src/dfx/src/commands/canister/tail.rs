use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::FetchCanisterLogsResponse;
use slog::info;

/// Get the canister logs.
#[derive(Parser)]
pub struct TailOpts {
    /// Specifies the name or id of the canister to get its canister information.
    canister: String,
}

fn format_canister_logs(logs: FetchCanisterLogsResponse) -> Vec<String> {
    logs.canister_log_records
        .into_iter()
        .map(|r| {
            let time = chrono::NaiveDateTime::from_timestamp_nanos(r.timestamp_nanos as i64)
                .expect("invalid timestamp");
            fn format_bytes(bytes: &[u8]) -> String {
                format!(
                    "[{}]",
                    bytes
                        .iter()
                        .map(|&v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            let message = match String::from_utf8(r.content.clone()) {
                Ok(s) => {
                    if format!("{s:?}").contains("\\u{") {
                        format_bytes(&r.content)
                    } else {
                        s
                    }
                }
                Err(_) => format_bytes(&r.content),
            };
            format!("[{}. {}]: {}", r.idx, time, message)
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
                timestamp_nanos: 1620328630000000001,
                content: b"some text message".to_vec(),
            },
            CanisterLogRecord {
                idx: 43,
                timestamp_nanos: 1620328630000000002,
                content: vec![1, 2, 3],
            },
        ],
    };
    assert_eq!(
        format_canister_logs(logs),
        vec![
            "[42. 2021-05-06 19:17:10.000000001]: some text message".to_string(),
            "[43. 2021-05-06 19:17:10.000000002]: [1, 2, 3]".to_string(),
        ],
    );
}

pub async fn exec(env: &dyn Environment, opts: TailOpts, call_sender: &CallSender) -> DfxResult {
    let log = env.get_logger();

    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;

    info!(log, "{}", format_canister_logs(logs).join("\n"));

    Ok(())
}
