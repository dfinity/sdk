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
    assert_command dfx identity get-wallet --identity alice
    assert_match "Using wasm at path: .*/wallet/0.7.0/wallet.wasm"

    use_wallet_wasm 0.7.2
    assert_command dfx identity get-wallet --identity bob
    assert_match "Using wasm at path: .*/wallet/0.7.2/wallet.wasm"

    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB_WALLET=$(dfx identity get-wallet --identity bob)

    assert_command dfx canister info "$ALICE_WALLET" --identity alice
    assert_match "Module hash: 0xa609400f2576d1d6df72ce868b359fd08e1d68e58454ef17db2361d2f1c242a1"

    assert_command dfx canister info "$BOB_WALLET" --identity bob
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
    setup_actuallylocal_shared_network

    # get Canister IDs to install the wasm onto
    dfx canister create hello_backend --network actuallylocal
    ID=$(dfx canister id hello_backend --network actuallylocal)
    dfx canister create hello_frontend --network actuallylocal
    ID_TWO=$(dfx canister id hello_frontend --network actuallylocal)

    # set controller to user
    dfx canister update-settings hello_backend --set-controller "$(dfx identity get-principal)" --network actuallylocal
    dfx canister update-settings hello_frontend --set-controller "$(dfx identity get-principal)" --network actuallylocal

    assert_command_fail dfx identity set-wallet "${ID}" --network actuallylocal
    assert_not_match "Setting wallet for identity"
    assert_command dfx identity set-wallet --force "${ID}" --network actuallylocal
    assert_match "Setting wallet for identity 'default' on network 'actuallylocal' to id '$ID'"
    assert_command jq -r .identities.default.actuallylocal <"$DFX_CONFIG_ROOT"/.config/dfx/identity/default/wallets.json
    assert_eq "$ID"
}

@test "deploy wallet" {
    dfx_new hello
    dfx_start
    setup_actuallylocal_shared_network

    # get Canister IDs to install the wasm onto
    dfx canister create hello_frontend --network actuallylocal
    ID=$(dfx canister id hello_frontend --network actuallylocal)
    dfx deploy hello_backend --network actuallylocal
    ID_TWO=$(dfx canister id hello_backend --network actuallylocal)

    # set controller to user
    dfx canister update-settings hello_backend --set-controller "$(dfx identity get-principal)" --network actuallylocal
    dfx canister update-settings hello_frontend --set-controller "$(dfx identity get-principal)" --network actuallylocal

    # We're testing on a local network so the create command actually creates a wallet
    # Delete this file to force associate wallet created by deploy-wallet to identity
    rm "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/wallets.json

    assert_command dfx identity deploy-wallet "${ID}" --network actuallylocal
    GET_WALLET_RES=$(dfx identity get-wallet --network actuallylocal)
    assert_eq "$ID" "$GET_WALLET_RES"

    # Command should fail on an already-deployed canister
    assert_command_fail dfx identity deploy-wallet "${ID_TWO}" --network actuallylocal
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
    assert_command dfx canister create --all --wallet "$WALLET"
    assert_command dfx build
    assert_command dfx canister install --all --wallet "$WALLET"

    CALL_RES=$(dfx canister call e2e_project_backend fromCall --wallet "$WALLET")
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
    CALL_RES=$(dfx canister call e2e_project_backend fromCall --wallet "$WALLET")
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
    assert_command dfx canister call hello_backend greet '("")' --with-cycles 1 --wallet "$WALLET"
    dfx identity new alice --disable-encryption
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    dfx wallet send "$ALICE_WALLET" 1
}

@test "dfx canister deposit-cycles succeeds on a canister the caller does not own" {
    dfx_new hello
    dfx_start
    dfx identity new alice --disable-encryption
    dfx deploy --no-wallet hello_backend --identity alice
    assert_command dfx canister deposit-cycles 1 hello_backend --wallet "$(dfx identity get-wallet)"
}

@test "dfx canister deposit-cycles uses default wallet if no wallet is specified" {
    dfx_new hello
    dfx_start
    dfx deploy
    assert_command dfx canister deposit-cycles 1 hello_backend
}

@test "detects if there is no wallet to upgrade" {
    dfx_new hello
    assert_command_fail dfx wallet upgrade
    assert_match "There is no wallet defined for identity 'default' on network 'local'.  Nothing to do."
}

@test "redeem-faucet-coupon can set a new wallet and top up an existing one" {
    dfx_new hello
    dfx_start
    install_asset faucet
    dfx deploy
    dfx ledger fabricate-cycles --canister faucet --t 1000

    dfx identity new --disable-encryption faucet_testing
    dfx identity use faucet_testing

    # prepare wallet to hand out
    dfx wallet balance # this creates a new wallet with user faucet_testing as controller
    dfx canister call faucet set_wallet_to_hand_out "(principal \"$(dfx identity get-wallet)\")" # register the wallet as the wallet that the faucet will return
    rm "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json" # forget about the currently configured wallet

    # assert: no wallet configured
    export DFX_DISABLE_AUTO_WALLET=1
    assert_command_fail dfx wallet balance
    assert_match "command requires a configured wallet"

    assert_command dfx wallet redeem-faucet-coupon --faucet "$(dfx canister id faucet)" 'valid-coupon'
    assert_match "Redeemed coupon valid-coupon for a new wallet"
    assert_match "New wallet set."

    # only succeeds if wallet is correctly set
    assert_command dfx wallet balance
    # checking only balance before the dot, rest may fluctuate
    # balance may be 99.??? TC if cycles accounting is done, or 100.000 TC if not
    assert_match "99\.|100\."

    unset DFX_DISABLE_AUTO_WALLET

    assert_command dfx wallet redeem-faucet-coupon --faucet "$(dfx canister id faucet)" 'another-valid-coupon'
    assert_eq "Redeemed coupon code another-valid-coupon for 10.000 TC (trillion cycles)."

    assert_command dfx wallet balance
    # checking only balance before the dot, rest may fluctuate
    # balance may be 109.??? TC if cycles accounting is done, or 110.000 TC if not
    assert_match "109\.|110\."
}
