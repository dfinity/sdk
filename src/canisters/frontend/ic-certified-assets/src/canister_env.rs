use std::collections::HashMap;

use ic_cdk::api::{env_var_count, env_var_name, env_var_value, root_key};

const ICP_CANISTER_IDS_PREFIX: &str = "ICP_CANISTER_ID:";

pub struct CanisterEnv {
    pub ic_root_key: Vec<u8>,
    pub icp_canister_ids: HashMap<String, String>,
}

impl CanisterEnv {
    pub fn new() -> Self {
        Self {
            ic_root_key: root_key(),
            icp_canister_ids: load_icp_canister_ids(),
        }
    }
}

fn load_icp_canister_ids() -> HashMap<String, String> {
    let mut icp_canister_ids = HashMap::new();
    let env_var_count = env_var_count();

    for i in 0..env_var_count {
        let name = env_var_name(i);
        if !name.starts_with(ICP_CANISTER_IDS_PREFIX) {
            continue;
        }
        let value = env_var_value(&name);
        icp_canister_ids.insert(name, value);
    }
    icp_canister_ids
}
