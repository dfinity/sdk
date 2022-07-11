#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "DFX_WALLET_WASM environment variable overrides wallet module wasm at installation" {
    dfx_new hello
    dfx_start

    dfx identity new --disable-encryption alice
    dfx identity new --disable-encryption bob

    use_wallet_wasm 0.7.0
    assert_command dfx --identity alice identity get-wallet
    assert_match "Using wasm at path: .*/wallet/0.7.0/wallet.wasm"

    use_wallet_wasm 0.7.2
    assert_command dfx --identity bob identity get-wallet
    assert_match "Using wasm at path: .*/wallet/0.7.2/wallet.wasm"

    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    assert_command dfx --identity alice canister info "$ALICE_WALLET"
    assert_match "Module hash: 0xa609400f2576d1d6df72ce868b359fd08e1d68e58454ef17db2361d2f1c242a1"

    assert_command dfx --identity bob canister info "$BOB_WALLET"
    assert_match "Module hash: 0x1404b28b1c66491689b59e184a9de3c2be0dbdd75d952f29113b516742b7f898"
}

@test "DFX_WALLET_WASM environment variable overrides wallet module wasm for upgrade" {
    dfx_new hello
    dfx_start

    use_wallet_wasm 0.7.0-beta.5

    assert_command dfx identity get-wallet
    WALLET_ID=$(dfx identity get-wallet)

    assert_command dfx canister info "$WALLET_ID"
    assert_match "Module hash: 0x3d5b221387875574a9fd75b3165403cf1b301650a602310e9e4229d2f6766dcc"

    use_wallet_wasm 0.7.0
    assert_command dfx wallet upgrade

    assert_command dfx canister info "$WALLET_ID"
    assert_match "Module hash: 0xa609400f2576d1d6df72ce868b359fd08e1d68e58454ef17db2361d2f1c242a1"
}

@test "'dfx identity set-wallet --force' bypasses wallet canister verification" {
    dfx_new hello
    dfx_start
    setup_actuallylocal_network

    # get Canister IDs to install the wasm onto
    dfx canister --network actuallylocal create hello_backend
    ID=$(dfx canister --network actuallylocal id hello_backend)
    dfx canister --network actuallylocal create hello_frontend
    ID_TWO=$(dfx canister --network actuallylocal id hello_frontend)

    # set controller to user
    dfx canister --network actuallylocal update-settings hello_backend --controller "$(dfx identity get-principal)"
    dfx canister --network actuallylocal update-settings hello_frontend --controller "$(dfx identity get-principal)"

    assert_command_fail dfx identity --network actuallylocal set-wallet "${ID}"
    assert_not_match "Setting wallet for identity"
    assert_command dfx identity --network actuallylocal set-wallet --force "${ID}"
    assert_match "Setting wallet for identity 'default' on network 'actuallylocal' to id '$ID'"
    assert_command jq -r .identities.default.actuallylocal <"$DFX_CONFIG_ROOT"/.config/dfx/identity/default/wallets.json
    assert_eq "$ID"
}

@test "deploy wallet" {
    dfx_new hello
    dfx_start
    setup_actuallylocal_network

    # get Canister IDs to install the wasm onto
    dfx canister --network actuallylocal create hello_frontend
    ID=$(dfx canister --network actuallylocal id hello_frontend)
    dfx deploy --network actuallylocal hello_backend
    ID_TWO=$(dfx canister --network actuallylocal id hello_backend)

    # set controller to user
    dfx canister --network actuallylocal update-settings hello_backend --controller "$(dfx identity get-principal)"
    dfx canister --network actuallylocal update-settings hello_frontend --controller "$(dfx identity get-principal)"

    # We're testing on a local network so the create command actually creates a wallet
    # Delete this file to force associate wallet created by deploy-wallet to identity
    rm "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/wallets.json

    assert_command dfx identity --network actuallylocal deploy-wallet "${ID}"
    GET_WALLET_RES=$(dfx identity --network actuallylocal get-wallet)
    assert_eq "$ID" "$GET_WALLET_RES"

    # Command should fail on an already-deployed canister
    assert_command_fail dfx identity --network actuallylocal deploy-wallet "${ID_TWO}"
    assert_match "The wallet canister \"${ID_TWO}\"\ already exists for user \"default\" on \"actuallylocal\" network."
}

@test "wallet create wallet" {
    dfx_new
    dfx_start
    WALLET_ID=$(dfx identity get-wallet)
    CREATE_RES=$(dfx canister call "${WALLET_ID}" wallet_create_wallet "(record { cycles = (2000000000000:nat64); settings = record {controller = opt principal \"$(dfx identity get-principal)\";};})")
    CHILD_ID=$(echo "${CREATE_RES}" | tr '\n' ' ' |  cut -d'"' -f 2)
    assert_command dfx canister call "${CHILD_ID}" wallet_balance '()'
}

@test "forward user call through wallet" {
    dfx_new
    install_asset identity
    dfx_start
    WALLET=$(dfx identity get-wallet)
    assert_command dfx canister --wallet "$WALLET" create --all
    assert_command dfx build
    assert_command dfx canister --wallet "$WALLET" install --all

    CALL_RES=$(dfx canister --wallet "$WALLET" call e2e_project_backend fromCall)
    CALLER=$(echo "${CALL_RES}" | cut -d'"' -f 2)
    assert_eq "$CALLER" "$WALLET"

    assert_command dfx canister call "$WALLET" wallet_call \
        "(record { canister = principal \"$(dfx canister id e2e_project_backend)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01~\01" } })'  # True in DIDL.
}

@test "forward user call through wallet: deploy" {
    dfx_new
    install_asset identity
    dfx_start
    WALLET=$(dfx identity get-wallet)
    assert_command dfx deploy --wallet "$WALLET"
    CALL_RES=$(dfx canister --wallet "$WALLET" call e2e_project_backend fromCall)
    CALLER=$(echo "${CALL_RES}" | cut -d'"' -f 2)
    assert_eq "$CALLER" "$WALLET"

    assert_command dfx canister call e2e_project_backend amInitializer
    assert_command dfx canister call "$WALLET" wallet_call \
        "(record { canister = principal \"$(dfx canister id e2e_project_backend)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01~\01" } })'  # True in DIDL.
}

@test "a 64-bit wallet can still be called in the 128-bit context" {
    use_wallet_wasm 0.8.2
    dfx_new hello
    dfx_start
    WALLET=$(dfx identity get-wallet)
    assert_command dfx wallet balance
    assert_command dfx deploy --wallet "$WALLET"
    assert_command dfx canister --wallet "$WALLET" call hello_backend greet '("")' --with-cycles 1
    dfx identity new alice --disable-encryption
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    dfx wallet send "$ALICE_WALLET" 1
}

@test "dfx canister deposit-cycles succeeds on a canister the caller does not own" {
    dfx_new hello
    dfx_start
    dfx identity new alice --disable-encryption
    dfx --identity alice deploy --no-wallet hello_backend
    assert_command dfx canister --wallet "$(dfx identity get-wallet)" deposit-cycles 1 hello_backend
}
