use crate::lib::CanisterId;

pub fn is_canister_id(v: String) -> Result<(), String> {
    v.parse::<CanisterId>()
        .map_err(|_| String::from("The value must be a canister ID (number)."))
        .map(|_| ())
}
