use std::fmt;

macro_rules! user_message {
    ( $($name: ident => $msg: literal,)+ ) => {
        #[derive(Debug, Copy, Clone)]
        pub enum UserMessage {
            $($name), +
        }

        impl UserMessage {
            pub fn to_str(&self) -> &str {
                match &self {
                    $(UserMessage::$name => $msg,)+
                }
            }
        }
    };
}

user_message!(
    // dfx cache
    ManageCache => "Manages the dfx version cache.",
    CacheDelete => "Delete a specific versioned cache of dfx.",
    CacheUnpack => "Force unpacking the cache from this dfx version.",
    CacheList => "List installed and used version.",
    CacheShow => "Show the path of the cache used by this version.",

    // dfx canister call
    CallCanister => "Calls a method on a deployed canister.",
    MethodName => "Specifies the method name to call on the canister.",
    AsyncResult => "Do not wait for the result of the call to be returned by polling the client. Instead return a response ID.",
    ArgumentType => "Specifies the data type for the argument when making the call using an argument.",
    ArgumentValue => "Specifies the argument to pass to the method.",

    // dfx canister install
    InstallCanister => "Installs compiled code as a canister on the client.",
    InstallAll => "Install all canisters configured in dfx.json.",
    InstallCanisterName => "Specifies the canister name. Either this or the --all flag are required.",

    // dfx canister mod
    ManageCanister => "Manages canisters deployed on a network client.",

    // dfx canister query
    QueryCanister => "Sends a query request to a canister.",
    CallCanisterArg => "Sends a call request to a canister. This is the default if the method is not a query method.",

    // dfx canister request_status
    RequestCallStatus => "Requests the status of a specified call from a canister.",
    RequestId => "Specifies the request identifier. The request identifier is an hexadecimal string starting with 0x.",

    // dfx build
    BuildCanister => "Builds all or specific canisters from the code in your project. By default, all canisters are built.",

    // dfx config
    ConfigureOptions => "Configures project options for your currently-selected project.",
    OptionName => "Specifies the name of the configuration option to set or read. Use the period delineated path to specify the option to set or read. If this is not mentioned, outputs the whole configuration.",
    OptionValue => "Specifies the new value to set. If you don't specify a value, the command displays the current value of the option from the configuration file.",
    OptionFormat => "Specifies the format of the output. By default, it uses JSON.",

    // dfx new
    CreateProject => "Creates a new project.",
    ProjectName => "Specifies the name of the project to create.",
    DryRun => "Provides a preview the directories and files to be created without adding them to the file system.",
    NewFrontend => "Install the frontend code example for the default canister. This defaults to true if Node is installed, or false if it isn't.",

    // dfx start
    StartNode => "Starts the local network client.",
    NodeAddress => "Specifies the host name and port number to bind the frontend to.",
    StartBackground => "Exits the dfx leaving the client running. Will wait until the client replies before exiting.",

    // misc
    CanisterName => "Specifies the canister name. If you don't specify this argument, all canisters are processed.",

    // dfx stop
    StopNode => "Stops the local network client.",
    // dfx ide
    StartLanguageService => "Starts the Motoko IDE Language Server. This is meant to be run by editor plugins not the end-user.",
    ForceTTY => "Forces the language server to start even when run from a terminal",
);

impl fmt::Display for UserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_str())
    }
}
