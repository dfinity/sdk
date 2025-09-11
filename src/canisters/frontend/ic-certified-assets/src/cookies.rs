use std::collections::HashMap;

use crate::{canister_env::CanisterEnv, url_encode::url_encode};

const SET_COOKIE_HEADER_NAME: &str = "Set-Cookie";

const IC_ENV_COOKIE_NAME: &str = "ic_env";

const IC_ROOT_KEY_VALUE_KEY: &str = "ic_root_key";
const VALUES_SEPARATOR: &str = "&";

pub fn add_ic_env_cookie(headers: &mut HashMap<String, String>) {
    let canister_env = get_canister_env();
    // TODO: add Secure attribute, see https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie#secure
    let cookie_value = format!("{IC_ENV_COOKIE_NAME}={canister_env}; SameSite=Lax");

    headers.insert(SET_COOKIE_HEADER_NAME.to_string(), cookie_value);
}

fn get_canister_env() -> String {
    let canister_env = CanisterEnv::new();
    to_cookie_value(&canister_env)
}

fn to_cookie_value(canister_env: &CanisterEnv) -> String {
    let hex_root_key = hex::encode(&canister_env.ic_root_key);
    let root_key_value = format!("{IC_ROOT_KEY_VALUE_KEY}={hex_root_key}");

    let mut values = vec![root_key_value];

    let icp_canister_ids = canister_env
        .icp_canister_ids
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>();
    values.extend(icp_canister_ids);

    let cookie_value = values.join(VALUES_SEPARATOR);

    url_encode(&cookie_value)
}
