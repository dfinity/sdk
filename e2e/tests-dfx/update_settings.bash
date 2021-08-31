#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    # Each test gets its own home directory in order to have its own identities.
    x=$(pwd)/home-for-test
    mkdir "$x"
    export HOME="$x"

    dfx_new hello
}

teardown() {
    dfx_stop
    rm -rf "$(pwd)/home-for-test"
}

@test "vs wallet 0.7.2, refuses to try to set multiple controllers" {
    use_wallet_wasm 0.7.2

    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    dfx deploy hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command_fail dfx canister update-settings hello --controller "${ALICE_WALLET}" --controller "${BOB_WALLET}"
    assert_match "The installed wallet does not support multiple controllers.  Please upgrade with 'dfx wallet upgrade'."
}

@test "set controller with wallet" {
    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello --controller "${BOB_WALLET}"
    assert_match "Set controller of \"hello\" to: ${BOB_WALLET}"

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --controller "${ALICE_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity alice canister update-settings hello --controller "${BOB_WALLET}"
    assert_match "Set controller of \"hello\" to: ${BOB_WALLET}"

    assert_command dfx --identity bob canister update-settings "${ID}" --controller alice
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity alice canister --no-wallet update-settings hello --controller charlie
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity alice canister --no-wallet update-settings hello_assets --controller bob
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}

@test "set controller with wallet 0.7.2" {
    use_wallet_wasm 0.7.2

    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello --controller "${BOB_WALLET}"
    assert_match "Set controller of \"hello\" to: ${BOB_WALLET}"

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --controller "${ALICE_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity alice canister update-settings hello --controller "${BOB_WALLET}"
    assert_match "Set controller of \"hello\" to: ${BOB_WALLET}"

    assert_command dfx --identity bob canister update-settings "${ID}" --controller alice
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity alice canister --no-wallet update-settings hello --controller charlie
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity alice canister --no-wallet update-settings hello_assets --controller bob
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}


@test "set multiple controllers" {
    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "$ALICE_WALLET" "$BOB_WALLET" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello --controller "${ALICE_WALLET}" --controller "${BOB_WALLET}"
    assert_match "Set controllers of \"hello\" to: $WALLETS_SORTED"

    # Both can reinstall
    assert_command dfx --identity alice canister install hello -m reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx canister info hello
    assert_match "Controllers: ${WALLETS_SORTED}"
}

@test "set multiple controllers even with wallet 0.7.2" {
    use_wallet_wasm 0.7.2
    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "$ALICE_WALLET" "$BOB_WALLET" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello --controller "${ALICE_WALLET}" --controller "${BOB_WALLET}"
    assert_match "Set controllers of \"hello\" to: $WALLETS_SORTED"

    # Both can reinstall
    assert_command dfx --identity alice canister install hello -m reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx canister info hello
    assert_match "Controllers: ${WALLETS_SORTED}"
}
