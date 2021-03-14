use ic_agent::RequestId;
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};

use super::error::DfxResult;
use anyhow::{anyhow, bail};
use serde_cbor::Value;
use std::convert::TryFrom;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessageV1 {
    version: usize,
    pub network: String, // url of the network
    pub call_type: String,
    pub sender: String,
    pub canister_id: String,
    pub method_name: String,
    pub arg: Vec<u8>,
    pub request_id: String, // only useful for update call
    pub content: String,    // hex::encode the Vec<u8>
}

impl SignedMessageV1 {
    pub fn new(
        network: String,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        arg: Vec<u8>,
    ) -> Self {
        Self {
            version: 1,
            network,
            call_type: String::new(),
            sender: sender.to_string(),
            canister_id: canister_id.to_string(),
            method_name,
            arg,
            request_id: String::new(),
            content: String::new(),
        }
    }

    pub fn with_call_type(mut self, request_type: String) -> Self {
        self.call_type = request_type;
        self
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = String::from(request_id);
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn validate(&self) -> DfxResult {
        let content = hex::decode(&self.content)?;

        let cbor: serde_cbor::Value = serde_cbor::from_slice(&content)
            .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;

        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
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
