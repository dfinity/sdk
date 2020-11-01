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

    // dfx canister call
    // CallCanister => "Calls a method on a deployed canister.",
    // AsyncResult => "Specifies not to wait for the result of the call to be returned by polling the replica. Instead return a response ID.",
    // ArgumentType => "Specifies the data type for the argument when making the call using an argument.",
    // OutputType => "Specifies the format for displaying the method's return result.",
    // ArgumentValue => "Specifies the argument to pass to the method.",

    // dfx canister mod
    ManageCanister => "Manages canisters deployed on a network replica.",

    // dfx build
    // CanisterComputeNetwork => "Override the compute network to connect to. By default, the local network is used.",

    // dfx identity mod
    ManageIdentity => "Manages identities used to communicate with the Internet Computer network. Setting an identity enables you to test user-based access controls.",

    // dfx replica
    // ReplicaRoundGasLimit => "Specifies the maximum number of cycles a single round can consume.",

    // misc
    // CanisterName => "Specifies the canister name. If you don't specify this argument, all canisters are processed.",

    // dfx ide
    // StartLanguageService => "Starts the Motoko IDE Language Server. This is meant to be run by editor plugins not the end-user.",
    // ForceTTY => "Forces the language server to start even when run from a terminal.",
);

impl fmt::Display for UserMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_str())
    }
}
