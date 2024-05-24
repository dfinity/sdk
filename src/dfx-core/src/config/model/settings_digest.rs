use crate::config::model::dfinity::{ReplicaLogLevel, ReplicaSubnetType};
use crate::config::model::local_server_descriptor::LocalServerDescriptor;
use candid::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::borrow::Cow;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HttpHandlerPortSetting {
    Port {
        port: u16,
    },
    #[default]
    WritePortToPath,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct HttpHandlerSettings {
    pub port: HttpHandlerPortSetting,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct BtcAdapterSettings {
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CanisterHttpAdapterSettings {
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct ReplicaSettings {
    pub http_handler: HttpHandlerSettings,
    pub subnet_type: ReplicaSubnetType,
    pub btc_adapter: BtcAdapterSettings,
    pub canister_http_adapter: CanisterHttpAdapterSettings,
    pub log_level: ReplicaLogLevel,
    pub artificial_delay: u32,
    pub use_old_metering: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BackendSettings<'a> {
    Replica { settings: Cow<'a, ReplicaSettings> },
    PocketIc,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct Settings<'a> {
    pub ic_repo_commit: String,
    #[serde(flatten)]
    pub backend: BackendSettings<'a>,
}

pub fn get_settings_digest(
    ic_repo_commit: &str,
    local_server_descriptor: &LocalServerDescriptor,
    use_old_metering: bool,
    artificial_delay: u32,
    pocketic: bool,
) -> String {
    let backend = if pocketic {
        BackendSettings::PocketIc
    } else {
        get_replica_backend_settings(local_server_descriptor, use_old_metering, artificial_delay)
    };
    let settings = Settings {
        ic_repo_commit: ic_repo_commit.into(),
        backend,
    };
    let normalized = serde_json::to_string_pretty(&settings).unwrap();
    let hash: Vec<u8> = Sha256::digest(normalized).to_vec();
    hex::encode(hash)
}

fn get_replica_backend_settings(
    local_server_descriptor: &LocalServerDescriptor,
    use_old_metering: bool,
    artificial_delay: u32,
) -> BackendSettings {
    let http_handler = HttpHandlerSettings {
        port: if let Some(port) = local_server_descriptor.replica.port {
            HttpHandlerPortSetting::Port { port }
        } else {
            HttpHandlerPortSetting::WritePortToPath
        },
    };
    let btc_adapter = BtcAdapterSettings {
        enabled: local_server_descriptor.bitcoin.enabled,
    };
    let canister_http_adapter = CanisterHttpAdapterSettings {
        enabled: local_server_descriptor.canister_http.enabled,
    };
    let replica_settings = ReplicaSettings {
        http_handler,
        subnet_type: local_server_descriptor
            .replica
            .subnet_type
            .unwrap_or_default(),
        btc_adapter,
        canister_http_adapter,
        log_level: local_server_descriptor
            .replica
            .log_level
            .unwrap_or_default(),
        artificial_delay,
        use_old_metering,
    };
    BackendSettings::Replica {
        settings: Cow::Owned(replica_settings),
    }
}
