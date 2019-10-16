use std::fmt;

pub enum UserMessage {
	CallCanister,
	CanisterId,
	MethodName,
	WaitForResult,
	ArgumentType,
	ArgumentValue,
}

pub fn user_message_str(message: &UserMessage) -> &'static str {
    match message {
        UserMessage::CallCanister => "Call a canister",
        UserMessage::CanisterId => "The canister ID (a number) to call.",
        UserMessage::MethodName => "The method name file to use.",
        UserMessage::WaitForResult => "Wait for the result of the call, by polling the client.",
        UserMessage::ArgumentType => "The type of the argument. Required when using an argument.",
        UserMessage::ArgumentValue => "Argument to pass to the method.",
    }
}

impl fmt::Display for UserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", user_message_str(&self))
    }
}