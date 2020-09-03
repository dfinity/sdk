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
    // dfx bootstrap
    BootstrapCommand => "Starts the bootstrap server.",
    BootstrapIP => "IP address that the bootstrap server listens on. Defaults to 127.0.0.1.",
    BootstrapPort => "Port number that the bootstrap server listens on. Defaults to 8081.",
    BootstrapRoot => "Directory containing static assets served by the bootstrap server. Defaults to $HOME/.cache/dfinity/versions/$DFX_VERSION/js-user-library/dist/bootstrap.",
    BootstrapTimeout => "Maximum amount of time, in seconds, the bootstrap server will wait for upstream requests to complete. Defaults to 30.",

    // dfx cache
    ManageCache => "Manages the dfx version cache.",
    CacheDelete => "Delete a specific versioned cache of dfx.",
    CacheUnpack => "Force unpacking the cache from this dfx version.",
    CacheList => "List installed and used version.",
    CacheShow => "Show the path of the cache used by this version.",

    // dfx canister id
    IdCanister => "Prints the identifier of a canister.",

    // dfx canister call
    CallCanister => "Calls a method on a deployed canister.",
    MethodName => "Specifies the method name to call on the canister.",
    AsyncResult => "Do not wait for the result of the call to be returned by polling the replica. Instead return a response ID.",
    ArgumentType => "Specifies the data type for the argument when making the call using an argument.",
    OutputType => "Specifies the format for displaying the method's return result.",
    ArgumentValue => "Specifies the argument to pass to the method.",


    // dfx canister create
    CreateCanister => "Creates an empty canister on the Internet Computer and associates the Internet Computer assigned Canister ID to the canister name.",
    CreateCanisterName => "Specifies the canister name. Either this or the --all flag are required.",
    CreateAll => "Creates all canisters configured in dfx.json.",

    // dfx canister install
    InstallCanister => "Installs compiled code as a canister on the replica.",
    InstallAll => "Install all canisters configured in dfx.json.",
    InstallCanisterName => "Specifies the canister name. Either this or the --all flag are required.",
    InstallComputeAllocation => "Specifies the canister's compute allocation. This should be a percent in the range [0..100]",
    InstallMode => "Install, Reinstall, or Upgrade the canister.",

    // dfx canister mod
    ManageCanister => "Manages canisters deployed on a network replica.",

    // dfx canister delete
    DeleteCanister => "Deletes a canister on the Internet Computer network.",
    DeleteCanisterName => "Specifies the name of the canister to delete. You must specify either a canister name or the --all flag.",
    DeleteAll => "Deletes all of the canisters configured in the dfx.json file.",

    // dfx canister status
    CanisterStatus => "Returns the current status of the canister on the Internet Computer network: Running, Stopping, or Stopped.",
    StatusCanisterName => "Specifies the name of the canister to return information for. You must specify either a canister name or the --all flag.",
    StatusAll => "Returns status information for all of the canisters configured in the dfx.json file.",

    // dfx canister start
    StartCanister => "Starts a canister on the Internet Computer network.",
    StartCanisterName => "Specifies the name of the canister to start. You must specify either a canister name or the --all flag.",
    StartAll => "Starts all of the canisters configured in the dfx.json file.",

    // dfx canister stop
    StopCanister => "Stops a canister that is currently running on the Internet Computer network.",
    StopCanisterName => "Specifies the name of the canister to stop. You must specify either a canister name or the --all flag.",
    StopAll => "Stops all of the canisters configured in the dfx.json file.",

    // dfx canister query
    QueryCanister => "Sends a query request to a canister.",
    UpdateCanisterArg => "Sends an update request to a canister. This is the default if the method is not a query method.",

    // dfx canister request_status
    RequestCallStatus => "Requests the status of a specified call from a canister.",
    RequestId => "Specifies the request identifier. The request identifier is an hexadecimal string starting with 0x.",

    // dfx build
    BuildAll => "Builds all canisters configured in dfx.json.",
    BuildCanisterName => "Specifies the canister name. Either this or the --all flag are required.",
    BuildCanister => "Builds all or specific canisters from the code in your project. By default, all canisters are built.",
    BuildCheck => "Build canisters without creating them. This can be used to check that canisters build ok.",
    CanisterComputeNetwork => "Override the compute network to connect to.  By default uses the local network.",

    // dfx config
    ConfigureOptions => "Configures project options for your currently-selected project.",
    OptionName => "Specifies the name of the configuration option to set or read. Use the period delineated path to specify the option to set or read. If this is not mentioned, outputs the whole configuration.",
    OptionValue => "Specifies the new value to set. If you don't specify a value, the command displays the current value of the option from the configuration file.",
    OptionFormat => "Specifies the format of the output. By default, it uses JSON.",

    // dfx identity mod
    ManageIdentity => "Manages identities used to communicate with the Internet Computer network. Setting an identity enables to test user-based access controls",

    // dfx identity new
    NewIdentity => "Create a new identity.",

    // dfx identity list
    ListIdentities => "List identities.",

    // dfx identity remove
    RemoveIdentity => "Remove an identity.",

    // dfx identity rename
    RenameIdentity => "Rename an identity.",

    // dfx identity use
    UseIdentity => "Specify the identity to use.",

    // dfx identity whoami
    ShowIdentity => "Show the name of the current identity.",

    // dfx new
    CreateProject => "Creates a new project.",
    ProjectName => "Specifies the name of the project to create.",
    DryRun => "Provides a preview the directories and files to be created without adding them to the file system.",
    NewFrontend => "Install the frontend code example for the default canister. This defaults to true if Node is installed, or false if it isn't.",

    // dfx ping
    Ping => "Ping an Internet Computer and returns its status.",

    // dfx replica
    Replica => "Start a local replica.",
    ReplicaMessageGasLimit => "Maximum amount of gas a single message can consume.",
    ReplicaPort => "The port the local replica should listen to.",
    ReplicaRoundGasLimit => "Maximum amount of gas a single round can consume.",

    // dfx start
    CleanState => "Cleans state of current project.",
    StartNode => "Starts the local replica and a web server for the current project.",
    NodeAddress => "Specifies the host name and port number to bind the frontend to.",
    StartBackground => "Exits the dfx leaving the replica running. Will wait until the replica replies before exiting.",

    // misc
    CanisterName => "Specifies the canister name. If you don't specify this argument, all canisters are processed.",

    // dfx stop
    StopNode => "Stops the local network replica.",
    // dfx ide
    StartLanguageService => "Starts the Motoko IDE Language Server. This is meant to be run by editor plugins not the end-user.",
    ForceTTY => "Forces the language server to start even when run from a terminal",
);

impl fmt::Display for UserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_str())
    }
}
