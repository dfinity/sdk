#![allow(unused)] // remove when there are no more todos

use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::CliOpts;
use anyhow::Context;
use clap::parser::ValueSource;
use clap::{ArgMatches, Command, CommandFactory};
use dfx_core::config::directories::project_dirs;
use dfx_core::config::model::dfinity::TelemetryState;
use dfx_core::fs;
use fd_lock::RwLock as FdRwLock;
use reqwest::StatusCode;
use semver::Version;
use serde::Serialize;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

use super::environment::Environment;

static TELEMETRY: OnceLock<Option<Mutex<Telemetry>>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArgumentSource {
    CommandLine,
    Environment,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Argument {
    name: String,
    value: Option<String>,
    source: ArgumentSource,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Telemetry {
    command: String,
    arguments: Vec<Argument>,
    elapsed: Option<Duration>,
    platform: String,
}

impl Telemetry {
    pub fn init(mode: TelemetryState) {
        if mode.should_collect() {
            TELEMETRY
                .set(Some(Mutex::new(Telemetry::default())))
                .expect("Telemetry already initialized");
        } else {
            TELEMETRY.set(None).expect("Telemetry already initialized");
        }
    }

    pub fn set_command_and_arguments(args: &[OsString]) -> DfxResult {
        try_with_telemetry(|telemetry| {
            let arg_matches = CliOpts::command().try_get_matches_from(args)?;

            let command = CliOpts::command();
            let (command_names, deepest_matches, deepest_command) =
                get_deepest_subcommand(&arg_matches, &command);
            let command_name = command_names.join(" ");

            let arguments = get_sanitized_arguments(deepest_matches, deepest_command);

            telemetry.command = command_name;
            telemetry.arguments = arguments;

            Ok(())
        })
    }

    pub fn get_telemetry_dir() -> DfxResult<PathBuf> {
        Ok(project_dirs()?.cache_dir().join("telemetry"))
    }

    pub fn get_log_path() -> DfxResult<PathBuf> {
        Ok(Self::get_telemetry_dir()?.join("telemetry.log"))
    }

    pub fn get_send_dir() -> DfxResult<PathBuf> {
        Ok(Self::get_telemetry_dir()?.join("send"))
    }

    pub fn set_platform() {
        with_telemetry(|telemetry| {
            telemetry.platform =
                if cfg!(target_os = "linux") && std::env::var_os("WSL_DISTRO_NAME").is_some() {
                    "wsl".to_string()
                } else {
                    std::env::consts::OS.to_string()
                }
        });
    }

    pub fn set_elapsed(elapsed: Duration) {
        with_telemetry(|telemetry| {
            telemetry.elapsed = Some(elapsed);
        });
    }

    pub fn append_record<T: Serialize>(record: &T) -> DfxResult<()> {
        let record = serde_json::to_string(record)?;
        let record = record.trim();
        let log_path = Self::get_log_path()?;
        fs::create_dir_all(log_path.parent().unwrap())?;
        let mut file = FdRwLock::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::get_log_path()?)?,
        );
        let mut lock = file.write()?;
        writeln!(*lock, "{}", record)?;
        Ok(())
    }

    pub fn append_current_command_timestamped(exit_code: i32) -> DfxResult<()> {
        try_with_telemetry(|telemetry| {
            let record = CommandRecord {
                tool: "dfx",
                version: dfx_version(),
                command: &telemetry.command,
                platform: &telemetry.platform,
                parameters: &telemetry.arguments,
                exit_code,
                execution_time_ms: telemetry.elapsed.map(|e| e.as_millis()),
                replica_error_call_site: None,
                replica_error_code: None,
                replica_reject_code: None,
                cycles_host: None,
                identity_type: None,
                network_type: None,
                project_canisters: None,
            };
            Self::append_record(&record)?;
            Ok(())
        })
    }

    pub fn send(url: &Url) -> DfxResult {
        Self::move_active_log_for_send()?;
        Self::transmit_any_batches(url)?;
        Ok(())
    }

    fn move_active_log_for_send() -> DfxResult {
        let log_path = Self::get_log_path()?;
        if !log_path.exists() {
            return Ok(());
        }

        let send_dir = Self::get_send_dir()?;
        fs::create_dir_all(&send_dir)?;

        let batch_id = Uuid::new_v4();
        eprintln!("Assigning telemetry.log contents to batch {:?}", batch_id);
        let batch_path = send_dir.join(batch_id.to_string());

        let mut file = FdRwLock::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?,
        );
        let lock = file.write()?;

        fs::rename(&log_path, &batch_path)?;
        Ok(())
    }

    fn transmit_any_batches(url: &Url) -> DfxResult {
        let batches = Self::list_batches()?;

        eprintln!("Batches to send:");
        for batch in &batches {
            eprintln!("  {:?}", batch);
        }

        let results = batches
            .iter()
            .map(|batch| Self::transmit_batch(&batch, url))
            .collect::<Vec<_>>();

        // return the first error, or Ok
        let x = results.into_iter().find_map(|r| r.err());
        match x {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn transmit_batch(batch: &Uuid, url: &Url) -> DfxResult {
        eprintln!("Transmitting batch: {:?}", batch);
        let batch_path = Self::get_send_dir()?.join(batch.to_string());

        let original_content = fs::read_to_string(&batch_path)?;
        let final_payload = Self::add_batch_and_sequence_to_batch(original_content, batch)?;

        let client = reqwest::blocking::Client::new();

        let op = || {
            client
                .post(url.as_str())
                .body(final_payload.clone())
                .send()
                .map_err(|e| backoff::Error::transient(e))
                .and_then(|response| {
                    response
                        .error_for_status()
                        .map_err(|e| backoff::Error::transient(e))
                })
                .map(|_| ())
        };
        let notify = |err, dur| {
            println!("Error happened at {:?}: {}", dur, err);
        };

        let policy = backoff::ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some(Duration::from_secs(180)))
            .build();

        backoff::retry_notify(policy, op, notify)?;

        fs::remove_file(&batch_path)?;

        Ok(())
    }

    fn add_batch_and_sequence_to_batch(content: String, batch: &Uuid) -> DfxResult<String> {
        // Process each line, adding batch ID and sequence number
        let modified_json_docs: Vec<String> = content
            .lines()
            .enumerate()
            .map(|(idx, line)| Self::add_batch_and_sequence(line, batch, idx as u64))
            .collect::<Result<_, _>>()?;

        // Reassemble into a newline-delimited JSON string
        Ok(modified_json_docs.join("\n"))
    }

    fn add_batch_and_sequence(content: &str, batch: &Uuid, sequence: u64) -> DfxResult<String> {
        let mut json: serde_json::Value = serde_json::from_str(content)?;

        json["batch"] = serde_json::Value::String(batch.to_string());
        json["sequence"] = serde_json::Value::Number(sequence.into());

        serde_json::to_string(&json).map_err(|e| e.into())
    }

    fn list_batches() -> DfxResult<Vec<Uuid>> {
        let send_dir = Self::get_send_dir()?;
        if !send_dir.exists() {
            return Ok(vec![]);
        }
        let send_dir = Self::get_send_dir()?;
        let dir_content = dfx_core::fs::read_dir(&send_dir)?;

        let batches = dir_content
            .filter_map(|v| {
                let dir_entry = v.ok()?;
                if dir_entry.file_type().is_ok_and(|e| e.is_file()) {
                    Uuid::parse_str(&dir_entry.file_name().to_string_lossy()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(batches)
    }
}

fn try_with_telemetry(f: impl FnOnce(&mut Telemetry) -> DfxResult) -> DfxResult {
    if let Some(telemetry) = TELEMETRY.get().unwrap() {
        f(&mut telemetry.lock().unwrap())?;
    }
    Ok(())
}

fn with_telemetry(f: impl FnOnce(&mut Telemetry)) {
    let _ = try_with_telemetry(|t| {
        f(t);
        Ok(())
    });
}

#[derive(Serialize, Debug)]
struct CommandRecord<'a> {
    tool: &'a str,
    version: &'a Version,
    command: &'a str,
    platform: &'a str,
    parameters: &'a [Argument],
    exit_code: i32,
    execution_time_ms: Option<u128>,
    replica_error_call_site: Option<&'a str>,  //todo
    replica_error_code: Option<&'a str>,       //todo
    replica_reject_code: Option<u8>,           //todo
    cycles_host: Option<CyclesHost>,           //todo
    identity_type: Option<IdentityType>,       //todo
    network_type: Option<NetworkType>,         //todo
    project_canisters: Option<&'a [Canister]>, //todo
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum CyclesHost {
    CyclesLedger,
    CyclesWallet,
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum IdentityType {
    Keyring,
    Plaintext,
    EncryptedLocal,
    Hsm,
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum NetworkType {
    LocalShared,
    ProjectLocal,
    Ic,
    UnknownConfigured,
    UnknownUrl,
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum CanisterType {
    Motoko,
    Rust,
    Assets,
    Custom,
    Pull,
}

#[derive(Serialize, Copy, Clone, Debug)]
struct Canister {
    r#type: CanisterType,
}

/// Finds the deepest subcommand in both `ArgMatches` and `Command`.
fn get_deepest_subcommand<'a>(
    matches: &'a ArgMatches,
    command: &'a clap::Command,
) -> (Vec<String>, &'a ArgMatches, &'a clap::Command) {
    let mut command_names = vec![];
    let mut deepest_matches = matches;
    let mut deepest_command = command;

    while let Some((subcommand_name, sub_matches)) = deepest_matches.subcommand() {
        command_names.push(subcommand_name.to_string());
        if let Some(sub_command) = deepest_command
            .get_subcommands()
            .find(|cmd| cmd.get_name() == subcommand_name)
        {
            deepest_matches = sub_matches;
            deepest_command = sub_command;
        } else {
            break;
        }
    }

    (command_names, deepest_matches, deepest_command)
}

fn get_sanitized_arguments(matches: &ArgMatches, command: &Command) -> Vec<Argument> {
    let mut arguments = vec![];

    let ids = matches.ids().map(|id| id.as_str()).collect::<Vec<_>>();

    for id in &ids {
        if matches!(matches.try_contains_id(id), Ok(c) if c) {
            let source = match matches.value_source(id) {
                Some(ValueSource::CommandLine) => ArgumentSource::CommandLine,
                Some(ValueSource::EnvVariable) => ArgumentSource::Environment,
                Some(ValueSource::DefaultValue) => continue,
                _ => continue, // ValueSource isn't exhaustive
            };

            let possible_values = command
                .get_arguments()
                .find(|arg| arg.get_id() == *id)
                .map(|arg| arg.get_possible_values());

            let sanitized_value = match (possible_values, matches.try_get_one::<String>(id)) {
                (Some(possible_values), Ok(Some(s)))
                    if possible_values.iter().any(|pv| pv.matches(s, true)) =>
                {
                    Some(s.clone())
                }
                _ => None,
            };

            let argument = Argument {
                name: id.to_string(),
                value: sanitized_value,
                source,
            };
            arguments.push(argument);
        }
    }
    arguments
}

#[cfg(test)]
impl Telemetry {
    /// Resets telemetry state. This is safe to call multiple times.
    pub fn init_for_test() {
        let mutex = TELEMETRY
            .get_or_init(|| Some(Mutex::new(Telemetry::default())))
            .as_ref()
            .unwrap();
        let mut telemetry = mutex.lock().unwrap();
        *telemetry = Telemetry::default(); // Reset the contents of the Mutex
    }

    pub fn get_for_test() -> Telemetry {
        TELEMETRY
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use std::sync::MutexGuard;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Sets up the test environment by locking TEST_LOCK and resetting telemetry state.
    #[must_use = "must store in a variable"]
    fn setup_test() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        Telemetry::init_for_test();
        guard
    }

    fn full_command_to_args(full_command: &str) -> Vec<OsString> {
        full_command
            .split_whitespace()
            .map(OsString::from)
            .collect()
    }

    fn full_command_to_telemetry(full_command: &str) -> Telemetry {
        let args = full_command_to_args(full_command);
        Telemetry::set_command_and_arguments(&args).unwrap();
        Telemetry::get_for_test()
    }

    #[test]
    fn simple() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx deploy");
        let expected = Telemetry {
            command: "deploy".to_string(),
            arguments: vec![],
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn hide_parameter_value() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx canister update-settings --add-log-viewer=evtzg-5hnpy-uoy4t-tn3fa-7c4ca-nweso-exmhj-nt3vs-htbce-pys7c-yqe e2e_project");
        let expected = Telemetry {
            command: "canister update-settings".to_string(),
            arguments: vec![
                Argument {
                    name: "add_log_viewer".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "LogVisibilityOpt".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "canister".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn network_param() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx deploy --network local");
        let expected = Telemetry {
            command: "deploy".to_string(),
            arguments: vec![
                Argument {
                    name: "network".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "NetworkOpt".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "network-select".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn network_param_in_middle() {
        let _guard = setup_test();
        let actual =
            full_command_to_telemetry("dfx canister --network local --wallet default call a b");
        let expected = Telemetry {
            command: "canister call".to_string(),
            arguments: vec![
                Argument {
                    name: "canister_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "method_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "network".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "wallet".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn ic_param() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx deploy --ic");
        let expected = Telemetry {
            command: "deploy".to_string(),
            arguments: vec![
                Argument {
                    name: "ic".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "NetworkOpt".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "network-select".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn canister_call_with_output_type() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx canister call mycan mymeth --output idl");
        let expected = Telemetry {
            command: "canister call".to_string(),
            arguments: vec![
                Argument {
                    name: "canister_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "method_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "output".to_string(),
                    value: Some("idl".to_string()),
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual)
    }

    #[test]
    fn numeric_parameter() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx canister create abc --compute-allocation 60");
        let expected = Telemetry {
            command: "canister create".to_string(),
            arguments: vec![
                Argument {
                    name: "canister_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "compute_allocation".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual)
    }

    #[test]
    fn bool_param() {
        let _guard = setup_test();
        let actual = full_command_to_telemetry("dfx canister create abc --no-wallet");
        let expected = Telemetry {
            command: "canister create".to_string(),
            arguments: vec![
                Argument {
                    name: "canister_name".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
                Argument {
                    name: "no_wallet".to_string(),
                    value: None,
                    source: ArgumentSource::CommandLine,
                },
            ],
            ..Default::default()
        };
        assert_eq!(expected, actual)
    }
}
