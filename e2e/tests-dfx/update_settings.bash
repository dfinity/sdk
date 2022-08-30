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

@test "set freezing threshold" {
    dfx_start
    assert_command dfx deploy hello_backend

    # trying to set threshold to 1T seconds, which should not work because it's likely a mistake
    assert_command_fail dfx canister update-settings hello_backend --freezing-threshold 100000000000
    assert_match "SECONDS" # error message pointing to the error

    # with manual override it's ok
    assert_command dfx canister update-settings hello_backend --freezing-threshold 100000000000 --confirm-very-long-freezing-threshold

    # to check if threshold is set correctly we have to un-freeze the canister by adding cycles. Fabricating 100T cycles onto it
    assert_command dfx ledger fabricate-cycles --canister hello_backend --t 100
    assert_command dfx canister status hello_backend
    assert_match "Freezing threshold: 100_000_000_000"
}

@test "set controller" {
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice
    
    dfx_start
    ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
    BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

    dfx canister create hello_backend
    dfx build hello_backend
    dfx canister install hello_backend
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller bob
    assert_match "Set controller of \"hello_backend\" to: bob"

    # Bob is controller, Alice cannot reinstall
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall

    # Bob can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "$ID" --set-controller "${ALICE_PRINCIPAL}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_PRINCIPAL}"
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_PRINCIPAL}" --identity alice
    assert_match "Set controller of \"hello_backend\" to: ${BOB_PRINCIPAL}"

    assert_command dfx canister update-settings "${ID}" --set-controller alice --identity bob
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx canister update-settings hello_backend --set-controller charlie --identity alice
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx canister update-settings hello_assets --set-controller bob --identity alice
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}

@test "set controller with wallet" {
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB_WALLET=$(dfx identity get-wallet --identity bob)

    dfx canister create hello_backend --wallet "${ALICE_WALLET}"
    dfx build hello_backend
    dfx canister install hello_backend --wallet "${ALICE_WALLET}"
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_WALLET}" --wallet "${ALICE_WALLET}"
    assert_match "Set controller of \"hello_backend\" to: ${BOB_WALLET}"

    # Bob is controller, Alice cannot reinstall
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall --wallet "${ALICE_WALLET}"

    # Bob can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob --wallet "${BOB_WALLET}"

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --set-controller "${ALICE_WALLET}" --wallet "${BOB_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall --wallet "${BOB_WALLET}"

    # Set controller using combination of name/id and identity/principal
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_WALLET}" --identity alice --wallet "${ALICE_WALLET}"
    assert_match "Set controller of \"hello_backend\" to: ${BOB_WALLET}"

    assert_command dfx canister update-settings "${ID}" --set-controller alice --identity bob --wallet "${BOB_WALLET}"
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx canister update-settings hello_backend --set-controller charlie --identity alice --wallet "${ALICE_WALLET}"
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx canister update-settings hello_assets --set-controller bob --identity alice --wallet "${ALICE_WALLET}"
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}

@test "set controller with wallet 0.7.2" {
    use_wallet_wasm 0.7.2

    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB_WALLET=$(dfx identity get-wallet --identity bob)

    dfx canister create hello_backend --wallet "${ALICE_WALLET}"
    dfx build hello_backend
    dfx canister install hello_backend --wallet "${ALICE_WALLET}"
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_WALLET}" --wallet "${ALICE_WALLET}"
    assert_match "Set controller of \"hello_backend\" to: ${BOB_WALLET}"

    # Bob is controller, Alice cannot reinstall
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall --wallet "${ALICE_WALLET}"

    # Bob can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob --wallet "${BOB_WALLET}"

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "${ID}" --set-controller "${ALICE_WALLET}" --wallet "${BOB_WALLET}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_WALLET}"
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall --wallet "${BOB_WALLET}"

    # Set controller using combination of name/id and identity/principal
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_WALLET}" --identity alice --wallet "${ALICE_WALLET}"
    assert_match "Set controller of \"hello_backend\" to: ${BOB_WALLET}"

    assert_command dfx canister update-settings "${ID}" --set-controller alice --identity bob --wallet "${BOB_WALLET}"
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx canister update-settings hello_backend --set-controller charlie --identity alice
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx canister update-settings hello_assets --set-controller bob --identity alice
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}

@test "set controller without wallet but using wallet 0.7.2" {
    use_wallet_wasm 0.7.2
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice
    
    dfx_start
    ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
    BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

    dfx canister create hello_backend
    dfx canister update-settings hello_backend --add-controller "$ALICE_PRINCIPAL" --wallet "$(dfx identity get-wallet)"
    dfx build hello_backend
    dfx canister install hello_backend
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller bob
    assert_match "Set controller of \"hello_backend\" to: bob"

    # Bob is controller, Alice cannot reinstall
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall

    # Bob can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister update-settings "$ID" --set-controller "${ALICE_PRINCIPAL}"
    assert_match "Set controller of \"${ID}\" to: ${ALICE_PRINCIPAL}"
    echo "yes" | assert_command_fail dfx canister install hello_backend -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx canister update-settings hello_backend --set-controller "${BOB_PRINCIPAL}" --identity alice
    assert_match "Set controller of \"hello_backend\" to: ${BOB_PRINCIPAL}"

    assert_command dfx canister update-settings "${ID}" --set-controller alice --identity bob
    assert_match "Set controller of \"${ID}\" to: alice"

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx canister update-settings hello_backend --set-controller charlie --identity alice
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx canister update-settings hello_assets --set-controller bob --identity alice
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}


@test "set multiple controllers" {
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
    BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)
    # awk step is to avoid trailing space
    PRINCIPALS_SORTED=$(echo "$ALICE_PRINCIPAL" "$BOB_PRINCIPAL" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello_backend
    dfx build hello_backend
    dfx canister install hello_backend
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${ALICE_PRINCIPAL}" --set-controller "${BOB_PRINCIPAL}"
    assert_match "Set controllers of \"hello_backend\" to: $PRINCIPALS_SORTED"

    # Both can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity alice
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob

    assert_command dfx canister info hello_backend
    assert_match "Controllers: ${PRINCIPALS_SORTED}"
}

@test "set multiple controllers with wallet" {
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB_WALLET=$(dfx identity get-wallet --identity bob)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "${ALICE_WALLET}" "${BOB_WALLET}" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello_backend
    dfx build hello_backend
    dfx canister install hello_backend
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${ALICE_WALLET}" --set-controller "${BOB_WALLET}" --wallet "${ALICE_WALLET}"
    assert_match "Set controllers of \"hello_backend\" to: ${WALLETS_SORTED}"

    # Both can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity alice --wallet "${ALICE_WALLET}"
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob --wallet "${BOB_WALLET}"

    assert_command dfx canister info hello_backend
    assert_match "Controllers: ${WALLETS_SORTED}"
}

@test "set multiple controllers even with wallet 0.7.2" {
    use_wallet_wasm 0.7.2
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB_WALLET=$(dfx identity get-wallet --identity bob)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "${ALICE_WALLET}" "${BOB_WALLET}" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello_backend
    dfx build hello_backend
    dfx canister install hello_backend --wallet "${ALICE_WALLET}"
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${ALICE_WALLET}" --set-controller "${BOB_WALLET}" --wallet "${ALICE_WALLET}"
    assert_match "Set controllers of \"hello_backend\" to: $WALLETS_SORTED"

    # Both can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity alice --wallet "${ALICE_WALLET}"
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob --wallet "${BOB_WALLET}"

    assert_command dfx canister info hello_backend
    assert_match "Controllers: ${WALLETS_SORTED}"
}

@test "set multiple controllers without wallet but using wallet 0.7.2" {
    use_wallet_wasm 0.7.2
    # Create two identities
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
    BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)
    # awk step is to avoid trailing space
    PRINCIPALS_SORTED=$(echo "$ALICE_PRINCIPAL" "$BOB_PRINCIPAL" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    dfx canister create hello_backend
    dfx canister update-settings hello_backend --add-controller "$ALICE_PRINCIPAL" --wallet "$(dfx identity get-wallet)"
    dfx build hello_backend
    dfx canister install hello_backend
    ID=$(dfx canister id hello_backend)

    # Set controller using canister name and identity name
    assert_command dfx canister update-settings hello_backend --set-controller "${ALICE_PRINCIPAL}" --set-controller "${BOB_PRINCIPAL}"
    assert_match "Set controllers of \"hello_backend\" to: $PRINCIPALS_SORTED"

    # Both can reinstall
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity alice
    echo "yes" | assert_command dfx canister install hello_backend -m reinstall --identity bob

    assert_command dfx canister info hello_backend
    assert_match "Controllers: ${PRINCIPALS_SORTED}"
}

@test "add controller to existing canister" {
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob
    assert_command dfx identity new --disable-encryption charlie

    dfx identity use alice
    dfx_start

    ALICE=$(dfx identity get-principal --identity alice)
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB=$(dfx identity get-principal --identity bob)
    CHARLIE=$(dfx identity get-principal --identity charlie)
    SORTED=$(echo "$ALICE" "$ALICE_WALLET" "$BOB" "$CHARLIE" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s %s %s",$1,$2,$3,$4}' )
    
    dfx canister create hello_backend
    dfx build hello_backend
    dfx canister install hello_backend
    
    # make bob a controller
    assert_command dfx canister update-settings hello_backend --add-controller bob
    # check that bob has the authority to make someone else a controller
    assert_command dfx canister update-settings hello_backend --add-controller charlie --identity bob
    assert_command dfx canister info hello_backend
    assert_match "Controllers: $SORTED"
}

@test "add controller to all canisters" {
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob 
    assert_command dfx identity new --disable-encryption charlie

    dfx identity use alice
    dfx_start

    ALICE=$(dfx identity get-principal --identity alice)
    ALICE_WALLET=$(dfx identity get-wallet --identity alice)
    BOB=$(dfx identity get-principal --identity bob)
    CHARLIE=$(dfx identity get-principal --identity charlie)
    SORTED=$(echo "$ALICE" "$ALICE_WALLET" "$BOB" "$CHARLIE" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s %s %s",$1,$2,$3,$4}' )
    
    dfx canister create --all
    dfx build --all
    dfx canister install --all

    # make bob a controller
    assert_command dfx canister update-settings --all --add-controller bob
    # check that bob has the authority to make someone else a controller
    assert_command dfx canister update-settings --all --add-controller charlie --identity bob
    assert_command dfx canister info hello_backend
    assert_match "Controllers: $SORTED"
}

@test "update settings by canister id, when canister id is not known to the project" {
    dfx_start
    dfx deploy

    CANISTER_ID=$(dfx canister id hello_backend)

    rm .dfx/local/canister_ids.json
    jq '.canisters={}' dfx.json | sponge dfx.json

    assert_command dfx canister status "$CANISTER_ID"
    assert_match 'Memory allocation: 0'
    assert_match 'Compute allocation: 0'
    assert_match 'Freezing threshold: 2_592_000'

    dfx canister update-settings --memory-allocation 2GB "$CANISTER_ID"
    assert_command dfx canister status "$CANISTER_ID"
    assert_match 'Memory allocation: 2_000_000_000'
    assert_match 'Compute allocation: 0'
    assert_match 'Freezing threshold: 2_592_000'

    # This is just checking that update-settings leaves the previous value
    # (of memory allocation) alone when setting something else

    # Compute allocations are temporarily disabled.
    # See https://dfinity.atlassian.net/browse/RUN-314
    # dfx canister update-settings --compute-allocation 1 "$CANISTER_ID"

    dfx canister update-settings --freezing-threshold 172 "$CANISTER_ID"
    assert_command dfx canister status "$CANISTER_ID"
    assert_match 'Memory allocation: 2_000_000_000'
    # assert_match 'Compute allocation: 4'
    assert_match 'Freezing threshold: 172'
}
