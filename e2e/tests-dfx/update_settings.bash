#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
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
    echo "yes" | assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    echo "yes" | assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --controller "${ALICE_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    echo "yes" | assert_command_fail dfx canister install hello -m reinstall

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
    echo "yes" | assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    echo "yes" | assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --controller "${ALICE_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    echo "yes" | assert_command_fail dfx canister install hello -m reinstall

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
    echo "yes" | assert_command dfx --identity alice canister install hello -m reinstall
    echo "yes" | assert_command dfx --identity bob canister install hello -m reinstall

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
    echo "yes" | assert_command dfx --identity alice canister install hello -m reinstall
    echo "yes" | assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx canister info hello
    assert_match "Controllers: ${WALLETS_SORTED}"
}
