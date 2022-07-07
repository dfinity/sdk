#!/usr/bin/env bats

load ../utils/_

setup() {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    standard_setup
    install_asset ledger

    dfx identity import --disable-encryption alice alice.pem
    dfx identity import --disable-encryption bob bob.pem

    dfx_start

    # local NNS_URL
    NNS_URL="http://localhost:$(cat .dfx/replica-configuration/replica-1.port)"
    local ic_nns_init
    case "$(uname)" in
    Darwin) ic_nns_init="./ic-nns-init_macos" ;;
    Linux) ic_nns_init="./ic-nns-init_linux" ;;
    *) echo "Unsupported platform $(uname)" && return 1 ;;
    esac

    "$ic_nns_init" \
      --url "$NNS_URL" \
      --initialize-ledger-with-test-accounts 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc\
      --wasm-dir .

    # Set the XDR conversion rate. Locally, 1 XDR = 1 ICP = 1 TC.
    dfx canister call rkp4c-7iaaa-aaaaa-aaaca-cai set_icp_xdr_conversion_rate \
        '(record { data_source= "max"; xdr_permyriad_per_icp = 10000 : nat64 ; timestamp_seconds= '"$(date +%s)"' : nat64 })'
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "ledger balance & transfer" {
    dfx identity use alice
    assert_command dfx ledger account-id
    assert_match 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752

    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"

    assert_command dfx ledger transfer --amount 100 --memo 1 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 # to bob
    assert_match "Transfer sent at BlockHeight:"

    # The sender(alice) paid transaction fee which is 0.0001 ICP
    assert_command dfx ledger balance
    assert_match "9899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger account-id
    assert_match 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89

    assert_command dfx ledger balance
    assert_match "10100.00000000 ICP"

    assert_command dfx ledger transfer --icp 100 --e8s 1 --memo 2 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 # to alice
    assert_match "Transfer sent at BlockHeight:"

    # The sender(bob) paid transaction fee which is 0.0001 ICP
    # 10100 - 100 - 0.0001 - 0.00000001 = 9999.99989999
    assert_command dfx ledger balance
    assert_match "9999.99989999 ICP"
}

@test "ledger subaccounts" {
    subacct=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    assert_command dfx --identity bob ledger account-id --subaccount "$subacct"
    assert_match 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc

    dfx identity use alice
    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"
    assert_command dfx ledger transfer --amount 100 --memo 1 5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc # to bob+subacct 
    assert_match "Transfer sent at BlockHeight:"
    assert_command dfx ledger balance
    assert_match "9899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"
    assert_command dfx ledger balance --subaccount "$subacct"
    assert_match "10100.00000000 ICP"
    
    assert_command dfx ledger transfer --amount 100 --memo 2 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --from-subaccount "$subacct" # to alice
    assert_match "Transfer sent at BlockHeight"
    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"
    assert_command dfx ledger balance --subaccount "$subacct"
    assert_match "9999.99990000 ICP"
    assert_command dfx --identity alice ledger balance
    assert_match "9999.99990000 ICP"
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
    assert_match "10000.00000000 ICP"

    wallet=$(dfx identity get-wallet)
    balance=$(tc_to_num "$(dfx wallet balance)")

    assert_command dfx ledger top-up "$wallet" --icp 5
    assert_match "Canister was topped up with 5000000000000 cycles"
    balance_now=$(tc_to_num "$(dfx wallet balance)")
    
    (( balance_now - balance > 4000000000000 ))
}
