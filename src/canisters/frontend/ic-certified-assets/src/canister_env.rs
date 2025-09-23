use std::collections::HashMap;

use ic_cdk::api::{env_var_count, env_var_name, env_var_value, root_key};

use crate::url_encode::url_encode;

const ICP_PUBLIC_ENV_VAR_NAME_PREFIX: &str = "ICP_PUBLIC_";

const IC_ROOT_KEY_VALUE_KEY: &str = "ic_root_key";
const COOKIE_VALUES_SEPARATOR: &str = "&";

pub struct CanisterEnv {
    pub ic_root_key: Vec<u8>,
    /// We can expect a maximum of 20 entries, each with a maximum of 128 characters
    /// for both the key and the value. Total size: 20 * 128 * 2 = 4096 bytes
    ///
    /// Numbers from https://github.com/dfinity/ic/blob/34bd4301f941cdfa1596a0eecf9f58ad6407293c/rs/config/src/execution_environment.rs#L175-L183
    pub icp_public_env_vars: HashMap<String, String>,
}

impl CanisterEnv {
    pub fn load() -> Self {
        Self {
            ic_root_key: root_key(),
            icp_public_env_vars: load_icp_public_env_vars(),
        }
    }

    pub fn to_cookie_value(&self) -> String {
        let hex_root_key = hex::encode(&self.ic_root_key);
        let root_key_value = format!("{IC_ROOT_KEY_VALUE_KEY}={hex_root_key}");

        let mut values = vec![root_key_value];

        let icp_public_env_vars = self
            .icp_public_env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>();
        values.extend(icp_public_env_vars);

        let cookie_value = values.join(COOKIE_VALUES_SEPARATOR);

        url_encode(&cookie_value)
    }
}

fn load_icp_public_env_vars() -> HashMap<String, String> {
    let mut icp_canister_ids = HashMap::new();
    let env_var_count = env_var_count();

    for i in 0..env_var_count {
        let name = env_var_name(i);
        if name.starts_with(ICP_PUBLIC_ENV_VAR_NAME_PREFIX) {
            let value = env_var_value(&name);
            icp_canister_ids.insert(name, value);
        }
    }
    icp_canister_ids
}
