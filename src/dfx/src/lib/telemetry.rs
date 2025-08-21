#![allow(unused)] // remove when there are no more todos

use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::CliOpts;
use anyhow::Context;
use candid::Principal;
use chrono::{Datelike, Local, NaiveDateTime};
use clap::parser::ValueSource;
use clap::{ArgMatches, Command, CommandFactory};
use dfx_core::config::directories::project_dirs;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::{
    CanisterTypeProperties, Config, ConfigCanistersCanister, TelemetryState,
};
use dfx_core::config::model::local_server_descriptor::LocalNetworkScopeDescriptor;
use dfx_core::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use dfx_core::fs;
use dfx_core::identity::IdentityType;
use fd_lock::{RwLock as FdRwLock, RwLockWriteGuard};
use fn_error_context::context;
use ic_agent::agent::RejectResponse;
use ic_agent::agent_error::Operation;
use ic_agent::AgentError;
use reqwest::StatusCode;
use semver::Version;
use serde::Serialize;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Seek;
use std::io::{Read, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

use super::environment::Environment;

static TELEMETRY: OnceLock<Option<Mutex<Telemetry>>> = OnceLock::new();

const SEND_SIZE_THRESHOLD_BYTES: u64 = 256 * 1024;

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
    last_reject: Option<RejectResponse>,
    last_operation: Option<Operation>,
    identity_type: Option<IdentityType>,
    cycles_host: Option<CyclesHost>,
    canisters: Option<Vec<CanisterRecord>>,
    network_type: Option<NetworkType>,
    allowlisted_canisters: BTreeSet<Principal>,
    week: Option<String>,
    publish: bool,
}

impl Telemetry {
    pub fn init(mode: TelemetryState) {
        if mode.should_collect() {
            TELEMETRY
                .set(Some(Mutex::new(
                    Telemetry::default().with_publish(mode.should_publish()),
                )))
                .expect("Telemetry already initialized");
        } else {
            TELEMETRY.set(None).expect("Telemetry already initialized");
        }
    }

    fn with_publish(&self, publish: bool) -> Self {
        let mut new = self.clone();
        new.publish = publish;
        new
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

    pub fn get_send_time_path() -> DfxResult<PathBuf> {
        Ok(Self::get_telemetry_dir()?.join("send-time.txt"))
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

    pub fn set_identity_type(identity_type: IdentityType) {
        with_telemetry(|telemetry| telemetry.identity_type = Some(identity_type));
    }

    pub fn set_cycles_host(host: CyclesHost) {
        with_telemetry(|telemetry| telemetry.cycles_host = Some(host));
    }

    pub fn set_week() {
        with_telemetry(|telemetry| {
            let iso_week = Local::now().naive_local().iso_week();
            let week = format!("{:04}-{:02}", iso_week.year(), iso_week.week());
            telemetry.week = Some(week);
        });
    }

    pub fn set_elapsed(elapsed: Duration) {
        with_telemetry(|telemetry| {
            telemetry.elapsed = Some(elapsed);
        });
    }

    pub fn set_error(error: &anyhow::Error) {
        with_telemetry(|telemetry| {
            for source in error.chain() {
                if let Some(agent_err) = source.downcast_ref::<AgentError>() {
                    if let AgentError::CertifiedReject { reject, operation }
                    | AgentError::UncertifiedReject { reject, operation } = agent_err
                    {
                        telemetry.last_reject = Some(reject.clone());
                        if let Some(operation) = operation {
                            telemetry.last_operation = Some(operation.clone());
                        }
                    }
                    break;
                }
            }
        });
    }

    pub fn set_canisters(canisters: Vec<CanisterRecord>) {
        with_telemetry(|telemetry| telemetry.canisters = Some(canisters));
    }

    pub fn allowlist_canisters(canisters: &[Principal]) {
        with_telemetry(|telemetry| telemetry.allowlisted_canisters.extend(canisters));
    }

    pub fn allowlist_all_asset_canisters(config: Option<&Config>, ids: &CanisterIdStore) {
        with_telemetry(|telemetry| {
            if let Some(config) = config {
                for (name, canister) in config.config.canisters.iter().flatten() {
                    if let CanisterTypeProperties::Assets { .. } = &canister.type_specific {
                        if let Ok(canister_id) = ids.get(name) {
                            telemetry.allowlisted_canisters.insert(canister_id);
                        }
                    }
                }
            }
        })
    }

    pub fn set_network(network: &NetworkDescriptor) {
        with_telemetry(|telemetry| {
            telemetry.network_type = Some(
                if let NetworkTypeDescriptor::Playground { .. } = &network.r#type {
                    NetworkType::Playground
                } else if network.is_ic {
                    NetworkType::Ic
                } else if let Some(local) = &network.local_server_descriptor {
                    match &local.scope {
                        LocalNetworkScopeDescriptor::Project => NetworkType::ProjectLocal,
                        LocalNetworkScopeDescriptor::Shared { .. } => NetworkType::LocalShared,
                    }
                } else if network.is_ad_hoc {
                    NetworkType::UnknownUrl
                } else {
                    NetworkType::UnknownConfigured
                },
            )
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
            let reject = telemetry.last_reject.as_ref();
            let call_site = telemetry.last_operation.as_ref().map(|o| match o {
                Operation::Call { method, canister } => {
                    if telemetry.allowlisted_canisters.contains(canister) {
                        method
                    } else {
                        "<user-specified canister method>"
                    }
                }
                Operation::ReadState { .. } | Operation::ReadSubnetState { .. } => "/read_state",
            });
            let record = CommandRecord {
                tool: "dfx",
                version: dfx_version(),
                command: &telemetry.command,
                platform: &telemetry.platform,
                parameters: &telemetry.arguments,
                week: telemetry.week.as_deref(),
                exit_code,
                execution_time_ms: telemetry.elapsed.map(|e| e.as_millis()),
                replica_error_call_site: call_site,
                replica_error_code: reject.and_then(|r| r.error_code.as_deref()),
                replica_reject_code: reject.map(|r| r.reject_code as u8),
                cycles_host: telemetry.cycles_host,
                identity_type: telemetry.identity_type,
                network_type: telemetry.network_type,
                project_canisters: telemetry.canisters.as_deref(),
            };
            Self::append_record(&record)?;
            Ok(())
        })
    }

    pub fn maybe_publish() -> DfxResult {
        try_with_telemetry(|telemetry| {
            if telemetry.publish && (Self::check_send_time()? || Self::check_file_size()?) {
                Self::launch_publisher()?;
            }

            Ok(())
        })
    }

    #[context("failed to launch publisher")]
    pub fn launch_publisher() -> DfxResult {
        let mut exe = std::env::current_exe()?;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("_send-telemetry")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0); // Detach from parent’s process group
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW (prevents it from being killed if parent exits)
        }

        cmd.spawn()?; // Spawn and immediately detach

        Ok(())
    }

    // look at telemetry/telemetry.log file size to see if it's time to send
    fn check_file_size() -> DfxResult<bool> {
        let path = Self::get_log_path()?;
        let filesize = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Ok(filesize >= SEND_SIZE_THRESHOLD_BYTES)
    }

    // Look at telemetry/send-time.txt to see if it's time to send
    #[context("failed to check send trigger")]
    fn check_send_time() -> DfxResult<bool> {
        let send_time_path = Self::get_send_time_path()?;

        let file = match OpenOptions::new().read(true).open(&send_time_path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Self::try_create_send_time(&send_time_path)?;
                return Ok(false);
            }
            Err(e) => return Err(e.into()),
        };

        let readlock = FdRwLock::new(file);
        let Ok(readguard) = readlock.try_read() else {
            return Ok(false);
        };

        let Ok(send_time) = Self::read_send_time(&send_time_path) else {
            // If there's some problem reading the send time, trigger sending.
            // This will overwrite the file with a new send time.
            return Ok(true);
        };

        let current_time = Local::now().naive_local();
        Ok(send_time <= current_time)
    }

    fn try_create_send_time(path: &Path) -> DfxResult {
        let file = match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        let mut lock = FdRwLock::new(file);
        if let Ok(mut write_guard) = lock.try_write() {
            Self::write_send_time(&mut write_guard, None)?;
        }
        Ok(())
    }

    fn read_send_time(path: &Path) -> DfxResult<NaiveDateTime> {
        let send_time = fs::read_to_string(path)?;
        let send_time = send_time.trim();
        let send_time = NaiveDateTime::parse_from_str(send_time, "%Y-%m-%d %H:%M:%S")
            .with_context(|| format!("failed to parse send time: {:?}", send_time))?;
        Ok(send_time)
    }

    fn write_send_time(
        guard: &mut RwLockWriteGuard<File>,
        future_duration: Option<Duration>,
    ) -> DfxResult {
        let future_duration = future_duration.unwrap_or_else(|| {
            let has_prerelease = !dfx_version().pre.is_empty();
            let future_seconds = if has_prerelease {
                // random 0.75 - 1.25 days in the future
                86400.0 * (0.75 + rand::random::<f64>() * 0.5)
            } else {
                // random 2-4 days in the future
                86400.0 * (2.0 + rand::random::<f64>() * 2.0)
            };
            Duration::from_secs(future_seconds as u64)
        });
        let future_time = Local::now().naive_local() + future_duration;
        let future_time_str = future_time.format("%Y-%m-%d %H:%M:%S").to_string();

        writeln!(*guard, "{}", future_time_str)?;
        Ok(())
    }

    pub fn send(url: &Url) -> DfxResult {
        fs::create_dir_all(&Self::get_telemetry_dir()?)?;
        let send_time_path = Self::get_send_time_path()?;

        let mut file = FdRwLock::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .truncate(true)
                .open(send_time_path)?,
        );

        let Ok(mut lock) = file.try_write() else {
            // another instance of _send-telemetry is already running
            return Ok(());
        };

        let thirty_minutes = 30 * 60;
        Self::write_send_time(&mut lock, Some(Duration::from_secs(thirty_minutes)))?;

        Self::move_active_log_for_send()?;
        Self::transmit_any_batches(url)?;

        lock.seek(SeekFrom::Start(0))?;
        Self::write_send_time(&mut lock, None)?;
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

        batches
            .iter()
            .map(|batch| Self::transmit_batch(batch, url))
            .find_map(Result::err)
            .map_or(Ok(()), Err)
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
                .map_err(backoff::Error::transient)
                .and_then(|response| {
                    response
                        .error_for_status()
                        .map_err(backoff::Error::transient)
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
    week: Option<&'a str>,
    parameters: &'a [Argument],
    exit_code: i32,
    execution_time_ms: Option<u128>,
    replica_error_call_site: Option<&'a str>,
    replica_error_code: Option<&'a str>,
    replica_reject_code: Option<u8>,
    cycles_host: Option<CyclesHost>,
    identity_type: Option<IdentityType>,
    network_type: Option<NetworkType>,
    project_canisters: Option<&'a [CanisterRecord]>,
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CyclesHost {
    CyclesLedger,
    CyclesWallet,
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkType {
    LocalShared,
    ProjectLocal,
    Ic,
    Playground,
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

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct CanisterRecord {
    r#type: CanisterType,
}

impl CanisterRecord {
    pub fn from_canister(config: &ConfigCanistersCanister) -> Self {
        let r#type = match &config.type_specific {
            CanisterTypeProperties::Rust { .. } => CanisterType::Rust,
            CanisterTypeProperties::Assets { .. } => CanisterType::Assets,
            CanisterTypeProperties::Motoko => CanisterType::Motoko,
            CanisterTypeProperties::Custom { .. } => CanisterType::Custom,
            CanisterTypeProperties::Pull { .. } => CanisterType::Pull,
        };
        Self { r#type }
    }
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
