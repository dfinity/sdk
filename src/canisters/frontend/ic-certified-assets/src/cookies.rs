use std::collections::BTreeMap;

const SET_COOKIE_HEADER_NAME: &str = "Set-Cookie";
const IC_ENV_COOKIE_NAME: &str = "ic_env";

pub fn add_ic_env_cookie(headers: &mut BTreeMap<String, String>, encoded_canister_env: &String) {
    let cookie_value = format!("{IC_ENV_COOKIE_NAME}={encoded_canister_env}; SameSite=Lax");

    headers.insert(SET_COOKIE_HEADER_NAME.to_string(), cookie_value);
}
