use crate::lib::error::DfxResult;

use candid::Principal;
use fn_error_context::context;
use ic_agent::RequestId;

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessageV1 {
    version: usize,
    #[serde(with = "date_time_utc")]
    pub creation: OffsetDateTime,
    #[serde(with = "date_time_utc")]
    pub expiration: OffsetDateTime,
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
        creation: OffsetDateTime,
        expiration: OffsetDateTime,
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

    #[context("Failed to validate signed message.")]
    pub fn validate(&self) -> DfxResult {
        if self.version != 1 {
            bail!("Invalid message: version must be 1");
        }

        if !["query", "update"].contains(&self.call_type.as_str()) {
            bail!("Invalid message: call_type must be `query` or `update`");
        }

        let content = hex::decode(&self.content).context("Failed to decode content.")?;

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
                    let expiration_from_cbor = OffsetDateTime::from_unix_timestamp_nanos(*s)?;
                    let diff = self.expiration - expiration_from_cbor;
                    if diff > Duration::seconds(5) || diff < Duration::seconds(-5) {
                        bail!(
                            "Invalid message: expiration not match\njson: {}\ncbor: {}",
                            self.expiration,
                            expiration_from_cbor
                        )
                    }
                    if OffsetDateTime::now_utc() > expiration_from_cbor {
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
    time::serde::format_description!(date_time, PrimitiveDateTime, "[year repr:full padding:zero]-[month repr:numerical padding:zero]-[day padding:zero] [hour repr:24 padding:zero]:[minute padding:zero]:[second padding:zero] UTC");

    use serde::{Deserializer, Serializer};
    use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

    pub fn serialize<S: Serializer>(datetime: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error> {
        let utc = datetime.to_offset(UtcOffset::UTC);
        let date = utc.date();
        let time = utc.time();
        date_time::serialize(&PrimitiveDateTime::new(date, time), s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<OffsetDateTime, D::Error> {
        let primitive = date_time::deserialize(d)?;
        Ok(primitive.assume_utc())
    }
}
