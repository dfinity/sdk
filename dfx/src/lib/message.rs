use std::fmt;

pub enum UserMessage {
    CallCanister,
    CanisterId,
    MethodName,
    WaitForResult,
    ArgumentType,
    ArgumentValue,
    InstallCanister,
    WasmFile,
    ManageCanister,
    QueryCanister,
    RequestCallStatus,
    RequestId,
    BuildCanister,
    CanisterName,
    ConfigureOptions,
    OptionName,
    OptionValue,
    CreateProject,
    ProjectName,
    DryRun,
    StartNode,
    NodeAddress,
    StartBackground,
}

impl UserMessage {
    pub fn to_str(&self) -> &str {
        match &self {

            // dfx canister call
            UserMessage::CallCanister => "Call a canister",
            UserMessage::CanisterId => "The canister ID (a number).",
            UserMessage::MethodName => "The method name file to use.",
            UserMessage::WaitForResult => "Wait for the result of the call, by polling the client.",
            UserMessage::ArgumentType => "The type of the argument. Required when using an argument.",
            UserMessage::ArgumentValue => "Argument to pass to the method.",

            // dfx canister install
            UserMessage::InstallCanister => "Install a canister.",
            UserMessage::WasmFile => "The WebAssembly (wasm) file to use.",

            // dfx canister mod
            UserMessage::ManageCanister => "Manage canisters from a network.",

            // dfx canister query
            UserMessage::QueryCanister => "Query a canister.",

            // dfx canister request_status
            UserMessage::RequestCallStatus => "Request the status of a call to a canister.",
            UserMessage::RequestId => "The request ID to call. This is an hexadecimal string starting with 0x.",

            // dfx build
            UserMessage::BuildCanister => "Build a canister code, or all canisters if no argument is passed.",
            UserMessage::CanisterName => "The canister name to build.",

            // dfx config
            UserMessage::ConfigureOptions => "Configure options in the current DFINITY project.",
            UserMessage::OptionName => "The name of the configuration option to set or read.",
            UserMessage::OptionValue => "The new value to set. If unspecified will output the current value in the config.",

            // dfx new
            UserMessage::CreateProject => "Create a new project.",
            UserMessage::ProjectName => "The name of the project to create.",
            UserMessage::DryRun => "Do not write anything to the file system.",

            // dfx start
            UserMessage::StartNode => "Start a local network in the background.",
            UserMessage::NodeAddress => "The host (with port) to bind the frontend to.",
            UserMessage::StartBackground => "Exit the dfx leaving the client running. Will wait until the client replies before exiting.",
        }
    }
}

impl fmt::Display for UserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_str())
    }
}
