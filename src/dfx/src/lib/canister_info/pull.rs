use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::bail;
use candid::Principal;
use dfx_core::config::model::dfinity::CanisterTypeProperties;

pub struct PullCanisterInfo {
    name: String,
    canister_id: Principal,
}

impl PullCanisterInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_canister_id(&self) -> &Principal {
        &self.canister_id
    }
}

impl CanisterInfoFactory for PullCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let name = info.get_name().to_string();
        let canister_id = {
            if let CanisterTypeProperties::Pull { id } = info.type_specific.clone() {
                id
            } else {
                bail!(
                    "Attempted to construct a pull canister from a type:{} canister config",
                    info.type_specific.name()
                );
            }
        };

        Ok(Self { name, canister_id })
    }
}
