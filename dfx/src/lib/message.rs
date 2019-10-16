pub fn usr_msg(msg_key: &str) -> &str {
    match msg_key {
        "CALL_CANISTER" => "Call a canister",
        "CANISTER_ID" => "The canister ID (a number) to call.",
        "METHOD_NAME" => "The method name file to use.",
        "WAIT_FOR_RESULT" => "Wait for the result of the call, by polling the client.",
        "ARG_TYPE" => "The type of the argument. Required when using an argument.",
        "ARG_VALUE" => "Argument to pass to the method.",
        _ => ""
    }
}