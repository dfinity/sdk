use crate::lib::error::DfxResult;

use ic_agent::RequestId;
use ic_types::principal::Principal;

use anyhow::{anyhow, bail};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessageV1 {
    version: usize,
    #[serde(with = "date_time_utc")]
    pub creation: DateTime<Utc>,
    #[serde(with = "date_time_utc")]
    pub expiration: DateTime<Utc>,
    pub network: String, // url of the network
    pub call_type: String,
    pub sender: String,
    pub canister_id: String,
    pub method_name: String,
    pub arg: Vec<u8>,
    pub request_id: Option<String>, // only useful for update call
    pub content: String,            // hex::encode the Vec<u8>
    pub signed_request_status: Option<String>, // hex::encode the Vec<u8>, only accompany update call
}

impl SignedMessageV1 {
    pub fn new(
        creation: DateTime<Utc>,
        expiration: DateTime<Utc>,
        network: String,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Vec<u8>,
    ) -> Self {
        Self {
            version: 1,
            creation,
            expiration,
            network,
            call_type: String::new(),
            sender: sender.to_string(),
            canister_id: canister_id.to_string(),
            method_name,
            arg,
            request_id: None,
            content: String::new(),
            signed_request_status: None,
        }
    }

    pub fn with_call_type(mut self, request_type: String) -> Self {
        self.call_type = request_type;
        self
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(String::from(request_id));
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn with_signed_request_status(mut self, signed_request_status: String) -> Self {
        self.signed_request_status = Some(signed_request_status);
        self
    }

    pub fn validate(&self) -> DfxResult {
        if self.version != 1 {
            bail!("Invalid message: version must be 1");
        }

        if !["query", "update"].contains(&self.call_type.as_str()) {
            bail!("Invalid message: call_type must be `query` or `update`");
        }

        let content = hex::decode(&self.content)?;

        let cbor: Value = serde_cbor::from_slice(&content)
            .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;

        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
                let ingress_expiry = m
                    .get(&Value::Text("ingress_expiry".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Integer(s) = ingress_expiry {
                    let seconds_since_epoch_cbor = Duration::from_nanos(*s as u64).as_secs();
                    let expiration_from_cbor = Utc.timestamp(seconds_since_epoch_cbor as i64, 0);
                    let diff = self.expiration.signed_duration_since(expiration_from_cbor);
                    if diff > chrono::Duration::seconds(5) || diff < chrono::Duration::seconds(-5) {
                        bail!(
                            "Invalid message: expiration not match\njson: {}\ncbor: {}",
                            self.expiration,
                            expiration_from_cbor
                        )
                    }
                    if Utc::now() > expiration_from_cbor {
                        bail!("The message has been expired at: {}", expiration_from_cbor);
                    }
                }
                let sender = m
                    .get(&Value::Text("sender".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Bytes(s) = sender {
                    let sender_from_cbor =
                        Principal::try_from(s).map_err(|_| anyhow!("Invalid cbor content."))?;
                    let sender_from_json = Principal::from_text(&self.sender)
                        .map_err(|_| anyhow!("Invalid json: sender."))?;
                    if !sender_from_cbor.eq(&sender_from_json) {
                        bail!(
                            "Invalid message: sender principle not match\njson: {}\ncbor: {}",
                            sender_from_json,
                            sender_from_cbor
                        )
                    }
                }

                let canister_id = m
                    .get(&Value::Text("canister_id".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Bytes(s) = canister_id {
                    let canister_id_from_cbor =
                        Principal::try_from(s).map_err(|_| anyhow!("Invalid cbor content."))?;
                    let canister_id_from_json = Principal::from_text(&self.canister_id)
                        .map_err(|_| anyhow!("Invalid json: canister_id."))?;
                    if !canister_id_from_cbor.eq(&canister_id_from_json) {
                        bail!(
                            "Invalid message: canister_id not match\njson: {}\ncbor: {}",
                            canister_id_from_json,
                            canister_id_from_cbor
                        )
                    }
                }

                let method_name = m
                    .get(&Value::Text("method_name".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Text(s) = method_name {
                    if !s.eq(&self.method_name) {
                        bail!(
                            "Invalid message: method_name not match\njson: {}\ncbor: {}",
                            self.method_name,
                            s
                        )
                    }
                }

                let arg = m
                    .get(&Value::Text("arg".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Bytes(s) = arg {
                    if !s.eq(&self.arg) {
                        bail!(
                            "Invalid message: arg not match\njson: {:?}\ncbor: {:?}",
                            self.arg,
                            s
                        )
                    }
                }
            } else {
                bail!("Invalid cbor content");
            }
        } else {
            bail!("Invalid cbor content");
        }
        Ok(())
    }
}

mod date_time_utc {
    // https://serde.rs/custom-date-format.html
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S UTC";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
