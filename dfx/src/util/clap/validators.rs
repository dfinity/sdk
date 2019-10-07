use ic_http_agent::CanisterId;

pub fn is_canister_id(v: String) -> Result<(), String> {
    v.parse::<CanisterId>()
        .map_err(|_| format!(r#"Value "{}" is not a valid canister ID"#, &v))
        .map(|_| ())
}

pub fn is_request_id(v: String) -> Result<(), String> {
    // A valid Request Id starts with `0x` and is a series of 64 hexadecimals.
    if !v.starts_with("0x") {
        Err(String::from("A Request ID needs to start with 0x."))
    } else if v.len() != 66 {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x.",
        ))
    } else if v.as_str()[2..].contains(|c: char| !c.is_ascii_hexdigit()) {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x. An invalid character was found.",
        ))
    } else {
        Ok(())
    }
}
