#!/usr/bin/env bats

load ../utils/_

# All tests in this file are skipped for ic-ref.  See scripts/workflows/e2e-matrix.py

setup() {
    standard_setup
    install_asset ledger
    install_shared_asset subnet_type/shared_network_settings/system

    dfx identity import --storage-mode plaintext alice alice.pem
    dfx identity import --storage-mode plaintext bob bob.pem

    dfx_start_for_nns_install

    dfx nns install --ledger-accounts 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc
}

teardown() {
    dfx_stop

    standard_teardown
}

current_time_nanoseconds() {
    echo "$(date +%s)"000000000
}

@test "ledger account-id" {
    dfx identity use alice
    assert_command dfx ledger account-id
    assert_match 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752

    assert_command dfx ledger account-id --of-principal fg7gi-vyaaa-aaaal-qadca-cai
    assert_match a014842f64a22e59887162a79c7ca7eb02553250704780ec4d954f12d0ea0b18

    # --of-canister accepts both canister alias and canister principal
    assert_command dfx canister create dummy_canister
    assert_command dfx ledger account-id --of-canister "$(dfx canister id dummy_canister)"
    assert_eq "$(dfx ledger account-id --of-canister dummy_canister)"
}

@test "ledger balance & transfer" {
    dfx identity use alice
    assert_command dfx ledger account-id
    assert_match 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752

    assert_command dfx ledger balance
    assert_match "1000000000.00000000 ICP"

    assert_command dfx ledger transfer --amount 100 --memo 1 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 # to bob
    assert_match "Transfer sent at block height"

    # The sender(alice) paid transaction fee which is 0.0001 ICP
    assert_command dfx ledger balance
    assert_match "999999899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger account-id
    assert_match 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89

    assert_command dfx ledger balance
    assert_match "1000000100.00000000 ICP"

    assert_command dfx ledger transfer --icp 100 --e8s 1 --memo 2 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 # to alice
    assert_match "Transfer sent at block height"

    # The sender(bob) paid transaction fee which is 0.0001 ICP
    # 10100 - 100 - 0.0001 - 0.00000001 = 9999.99989999
    assert_command dfx ledger balance
    assert_match "999999999.99989999 ICP"

    # Transaction Deduplication
    t=$(current_time_nanoseconds)

    assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time "$t" 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
    # shellcheck disable=SC2154
    block_height=$(echo "$stdout" | sed '1q' | sed 's/Transfer sent at block height //')
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"

    assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time $((t+1)) 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Transfer sent at block height $block_height" "$stdout"

    assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time "$t" 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
    # shellcheck disable=SC2154
    assert_match "transaction is a duplicate of another transaction in block $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"

    assert_command dfx ledger transfer --icp 1 --memo 2 --created-at-time "$t" 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Transfer sent at block height $block_height" "$stdout"

}

@test "ledger subaccounts" {
    subacct=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    assert_command dfx ledger account-id --identity bob --subaccount "$subacct"
    assert_match 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc

    dfx identity use alice
    assert_command dfx ledger balance
    assert_match "1000000000.00000000 ICP"
    assert_command dfx ledger transfer --amount 100 --memo 1 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc # to bob+subacct 
    assert_match "Transfer sent at block height"
    assert_command dfx ledger balance
    assert_match "999999899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger balance
    assert_match "1000000000.00000000 ICP"
    assert_command dfx ledger balance --subaccount "$subacct"
    assert_match "1000000100.00000000 ICP"
    
    assert_command dfx ledger transfer --amount 100 --memo 2 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --from-subaccount "$subacct" # to alice
    assert_match "Transfer sent at block height"
    assert_command dfx ledger balance
    assert_match "1000000000.00000000 ICP"
    assert_command dfx ledger balance --subaccount "$subacct"
    assert_match "999999999.99990000 ICP"
    assert_command dfx ledger balance --identity alice
    assert_match "999999999.99990000 ICP"
}
tc_to_num() {
    if [[ $1 =~ T ]]; then
        echo "${1%%[^0-9]*}000000000000"
    else
        echo "${1%%[^0-9]*}"
    fi
}

@test "ledger top-up" {
    dfx identity use alice
    assert_command dfx ledger balance
    assert_match "1000000000.00000000 ICP"

    wallet=$(dfx identity get-wallet)
    balance=$(tc_to_num "$(dfx wallet balance)")

    assert_command dfx ledger top-up "$wallet" --icp 5
    assert_match "Canister was topped up with 617283500000000 cycles"
    balance_now=$(tc_to_num "$(dfx wallet balance)")
    
    (( balance_now - balance > 600000000000000 ))
    
    # Transaction Deduplication
    t=$(current_time_nanoseconds)

    assert_command dfx ledger top-up "$wallet" --icp 5 --created-at-time "$t"
    
    # shellcheck disable=SC2154
    block_height=$(echo "$stdout" | sed '1q' | sed 's/Transfer sent at block height //')
    
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister was topped up with" "$stdout"

    assert_command dfx ledger top-up "$wallet" --icp 5 --created-at-time $((t+1))
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister was topped up with" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Using transfer at block height $block_height" "$stdout"
    
    assert_command dfx ledger top-up "$wallet" --icp 5 --created-at-time "$t"
    # shellcheck disable=SC2154
    assert_match "transaction is a duplicate of another transaction in block $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister was topped up with" "$stdout"
}

@test "ledger create-canister" {
    dfx identity use alice
    assert_command dfx ledger create-canister --amount=100 --subnet-type "type1" "$(dfx identity get-principal)"
    assert_match "Transfer sent at block height"
    assert_match "Refunded at block height"
    assert_match "with message: Provided subnet type type1 does not exist"
    
    # Transaction Deduplication
    t=$(current_time_nanoseconds)

    assert_command dfx ledger create-canister --amount=100 --created-at-time "$t" "$(dfx identity get-principal)"
    # shellcheck disable=SC2154
    block_height=$(echo "$stdout" | sed '1q' | sed 's/Transfer sent at block height //')
    # shellcheck disable=SC2154
    created_canister_id=$(echo "$stdout" | sed '3q;d' | sed 's/Canister created with id: //')
    
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister created with id: $created_canister_id" "$stdout"
    
    assert_command dfx ledger create-canister --amount=100 --created-at-time $((t+1)) "$(dfx identity get-principal)"
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister created with id:" "$stdout" 
    # shellcheck disable=SC2154
    assert_not_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Using transfer at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_not_match "Canister created with id: $created_canister_id" "$stdout"
    
    assert_command dfx ledger create-canister --amount=100 --created-at-time "$t" "$(dfx identity get-principal)"
    # shellcheck disable=SC2154
    assert_match "transaction is a duplicate of another transaction in block $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Transfer sent at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Using transfer at block height $block_height" "$stdout"
    # shellcheck disable=SC2154
    assert_match "Canister created with id: $created_canister_id" "$stdout"
    
}

@test "ledger show-subnet-types" {
    install_asset cmc

    dfx deploy cmc

    CANISTER_ID=$(dfx canister id cmc)

    assert_command dfx ledger show-subnet-types --cycles-minting-canister-id "$CANISTER_ID"
    assert_eq '["type1", "type2"]'
}
