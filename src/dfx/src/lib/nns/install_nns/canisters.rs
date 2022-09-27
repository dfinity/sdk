//! Information about canisters deployed via `ic nns install`.

/// Configuration for an NNS canister installation as performed by `ic-nns-init`.
///
/// Note: Other deployment methods may well use different settings.
pub struct IcNnsInitCanister {
    /// The name of the canister as typically entered in dfx.json or used in `dfx canister id NAME`.
    pub canister_name: &'static str,
    /// The basename of the wasm file.
    pub wasm_name: &'static str,
    /// ic-nns-init uses test wasms for some canisters but still requires the standard wasm to be present.  For unknown reasons.
    pub test_wasm_name: Option<&'static str>,
    /// The id of the canister when installed by `dfx nns install`.
    pub canister_id: &'static str,
}
/// Canister to keep a record of hardware and resource providers.
pub const NNS_REGISTRY: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-registry",
    wasm_name: "registry-canister.wasm",
    test_wasm_name: None,
    canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai",
};
/// Canister used to hold referanda and execute NNS proposals.
pub const NNS_GOVERNANCE: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-governance",
    test_wasm_name: Some("governance-canister_test.wasm"),
    wasm_name: "governance-canister.wasm",
    canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai",
};
/// Canister that holds ICP account balances.
pub const NNS_LEDGER: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-ledger",
    wasm_name: "ledger-canister_notify-method.wasm",
    test_wasm_name: None,
    canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai",
};
/// Canister that controls all NNS canisters.
pub const NNS_ROOT: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-root",
    wasm_name: "root-canister.wasm",
    test_wasm_name: None,
    canister_id: "r7inp-6aaaa-aaaaa-aaabq-cai",
};
/// Canister that exchanges ICP for cycles.
pub const NNS_CYCLES_MINTING: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-cycles-minting",
    wasm_name: "cycles-minting-canister.wasm",
    test_wasm_name: None,
    canister_id: "rkp4c-7iaaa-aaaaa-aaaca-cai",
};
/// Canister used to restore functionality in an emergency.
pub const NNS_LIFELINE: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-lifeline",
    wasm_name: "lifeline.wasm",
    test_wasm_name: None,
    canister_id: "rno2w-sqaaa-aaaaa-aaacq-cai",
};
/// Canister used to store genesis tokens.
pub const NNS_GENESIS_TOKENS: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-genesis-token",
    wasm_name: "genesis-token-canister.wasm",
    test_wasm_name: None,
    canister_id: "renrk-eyaaa-aaaaa-aaada-cai",
};
/// Placeholder for the Internet Identity.  Not used.
pub const NNS_IDENTITY: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-identity",
    wasm_name: "identity-canister.wasm",
    test_wasm_name: None,
    canister_id: "rdmx6-jaaaa-aaaaa-aaadq-cai",
};
/// Placeholder for the nns-dapp.  Not used.
pub const NNS_UI: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-ui",
    wasm_name: "nns-ui-canister.wasm",
    test_wasm_name: None,
    canister_id: "qoctq-giaaa-aaaaa-aaaea-cai",
};
/// Canister that spawns SNS canister groups.
pub const NNS_SNS_WASM: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-sns-wasm",
    wasm_name: "sns-wasm-canister.wasm",
    test_wasm_name: None,
    canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai",
};
/// Canister that converts between bitcoin and ckBTC.
pub const NNS_IC_CKBTC_MINTER: IcNnsInitCanister = IcNnsInitCanister {
    canister_name: "nns-ic-ckbtc-minter",
    wasm_name: "ic-ckbtc-minter.wasm",
    test_wasm_name: None,
    canister_id: "qjdve-lqaaa-aaaaa-aaaeq-cai",
};
/// Minimum data needed to download and deploy a standard canister via dfx deploy NAME.
pub struct StandardCanister {
    /// The typical name of the canister, as seen in dfx.json or used in `dfx canister id NAME`.
    pub canister_name: &'static str,
    /// The canister ID when the canister is installed locally by dfx-nns-install.
    pub canister_id: &'static str,
    /// The basename of the wasm file.
    pub wasm_name: &'static str,
    /// The URL from which to download the wasm file.
    pub wasm_url: &'static str,
}
/// A canister that provides login as a service for other dapps.
pub const INTERNET_IDENTITY: StandardCanister = StandardCanister {
    canister_name: "internet_identity",
    canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai",
    wasm_name: "internet_identity_dev.wasm",
    wasm_url: "https://github.com/dfinity/internet-identity/releases/download/release-2022-07-11/internet_identity_dev.wasm"
};
/// Frontend dapp for voting and managing neurons.
pub const NNS_DAPP: StandardCanister = StandardCanister {
    canister_name: "nns-dapp",
    canister_id: "qsgjb-riaaa-aaaaa-aaaga-cai",
    wasm_name: "nns-dapp_local.wasm",
    wasm_url: "https://github.com/dfinity/nns-dapp/releases/download/tip/nns-dapp_t2.wasm",
};
/// Backend canisters deployed by `ic nns init`.
pub const NNS_CORE: &[&IcNnsInitCanister; 11] = &[
    &NNS_REGISTRY,       // 0
    &NNS_GOVERNANCE,     // 1
    &NNS_LEDGER,         // 2
    &NNS_ROOT,           // 3
    &NNS_CYCLES_MINTING, // 4
    &NNS_LIFELINE,       // 5
    &NNS_GENESIS_TOKENS, // 6
    &NNS_IDENTITY,       // 7
    &NNS_UI,             // 8
    // 9 - unused
    &NNS_SNS_WASM, // 10
    // ic-icrc1-ledger is expected to go in place 11.
    &NNS_IC_CKBTC_MINTER, // Index not defined here: https://github.com/dfinity/ic/blob/master/rs/nns/constants/src/lib.rs
];
/// Frontend canisters deployed by `ic nns init`.  The deployment is normal, like any other canister.
pub const NNS_FRONTEND: [&StandardCanister; 2] = [&INTERNET_IDENTITY, &NNS_DAPP];

/// Information required for WASMs uploaded to the nns-sns-wasm canister.
///
/// Note:  These wasms are not deployed by `ic nns install` but later by developers
pub struct SnsCanisterInstallation {
    /// The name of the canister as typically added to dfx.json or used in `dfx cansiter id NAME`
    pub canister_name: &'static str,
    /// The name used when uploading to the nns-sns-wasm canister.
    pub upload_name: &'static str,
    /// The basename of the wasm file.
    pub wasm_name: &'static str,
}
/// The controller of all the canisters in a given SNS project.
pub const SNS_ROOT: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-root",
    upload_name: "root",
    wasm_name: "sns-root-canister.wasm",
};
/// The governance canister for an SNS project.
pub const SNS_GOVERNANCE: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-governance",
    upload_name: "governance",
    wasm_name: "sns-governance-canister.wasm",
};
/// Manages the decentralisation of an SNS project, exchanging stake in the mainnet for stake in the project.
pub const SNS_SWAP: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-swap",
    upload_name: "swap",
    wasm_name: "sns-swap-canister.wasm",
};
/// Stores account balances for an SNS project.
pub const SNS_LEDGER: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-ledger",
    upload_name: "ledger",
    wasm_name: "ic-icrc1-ledger.wasm",
};
/// Stores ledger data; needed to preserve data if an SNS ledger canister is upgraded.
pub const SNS_LEDGER_ARCHIVE: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-ledger-archive",
    upload_name: "archive",
    wasm_name: "ic-icrc1-archive.wasm",
};
/// SNS wasm files hosted by the nns-sns-wasms canister.
///
/// Note:  Sets of these canisters are deployed on request, so one network will
/// typically have many sets of these canisters, one per project decentralized
/// with the SNS toolchain.
pub const SNS_CANISTERS: [&SnsCanisterInstallation; 5] = [
    &SNS_ROOT,
    &SNS_GOVERNANCE,
    &SNS_SWAP,
    &SNS_LEDGER,
    &SNS_LEDGER_ARCHIVE,
];

/// Test account with well known public & private keys, used in NNS_LEDGER, NNS_DAPP and third party projects.
/// The keys use the ED25519 curve, used for BasicIdentity on th eInternet Computer.
/// The keys are as follows, in the tweetnacl format used by agent-js:
/// ```
/// const publicKey = "Uu8wv55BKmk9ZErr6OIt5XR1kpEGXcOSOC1OYzrAwuk=";
/// const privateKey =
///    "N3HB8Hh2PrWqhWH2Qqgr1vbU9T3gb1zgdBD8ZOdlQnVS7zC/nkEqaT1kSuvo4i3ldHWSkQZdw5I4LU5jOsDC6Q==";
/// const identity = Ed25519KeyIdentity.fromKeyPair(
///  base64ToUInt8Array(publicKey),
///  base64ToUInt8Array(privateKey)
/// );
/// ```
pub const ED25519_TEST_ACCOUNT: &str =
    "5b315d2f6702cb3a27d826161797d7b2c2e131cd312aece51d4d5574d1247087";

/// Test account for command line usage.  The test account is typically called ident-1
/// The keys use the secp256k1 curve.  To use:
/// ```
/// $ cat ~/.config/dfx/identity/ident-1/identity.pem
/// -----BEGIN EC PRIVATE KEY-----
/// MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
/// oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
/// 74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
/// -----END EC PRIVATE KEY-----
/// ```
/// The test account should match the output of:
/// ```
/// $ dfx --identity ident-1 ledger account-id
/// ```
pub const SECP256K1_TEST_ACCOUNT: &str =
    "2b8fbde99de881f695f279d2a892b1137bfe81a42d7694e064b1be58701e1138";
