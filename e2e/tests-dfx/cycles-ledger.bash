#!/usr/bin/env bats

load ../utils/_
load ../utils/cycles-ledger

setup() {
    standard_setup
    install_asset cycles-ledger
    install_shared_asset subnet_type/shared_network_settings/system
    install_cycles_ledger_canisters

    dfx identity new --storage-mode plaintext cycle-giver
    dfx identity new --storage-mode plaintext alice
    dfx identity new --storage-mode plaintext bob

    dfx_start_for_nns_install

    dfx extension install nns --version 0.2.1 || true
    dfx nns install --ledger-accounts "$(dfx ledger account-id --identity cycle-giver)"
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "cycles ledger howto" {
    # This is the equivalent of https://www.notion.so/dfinityorg/How-to-install-and-test-the-cycles-ledger-521c9f3c410f4a438514a03e35464299
    ALICE=$(dfx identity get-principal --identity alice)
    BOB=$(dfx identity get-principal --identity bob)

    assert_command dfx deploy cycles-ledger
    assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

    assert_command dfx ledger balance --identity cycle-giver
    assert_eq "1000000000.00000000 ICP"

    assert_command dfx canister status cycles-depositor
    assert_contains "Balance: 10_000_000_000_000 Cycles"

    dfx canister status cycles-depositor

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$ALICE\"})"
    assert_eq "(0 : nat)"

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$BOB\"})"
    assert_eq "(0 : nat)"

    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 500_000_000;})" --identity cycle-giver
    assert_eq "(record { balance = 500_000_000 : nat; txid = 0 : nat })"

    assert_command dfx canister status cycles-depositor
    assert_contains "Balance: 9_999_500_000_000 Cycles"

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$ALICE\"})"
    assert_eq "(500_000_000 : nat)"

    assert_command dfx canister call cycles-ledger icrc1_transfer "(record {to = record{owner = principal \"$BOB\"}; amount = 100_000;})" --identity alice
    assert_eq "(variant { Ok = 1 : nat })"

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$ALICE\"})"
    assert_eq "(399_900_000 : nat)"

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$BOB\"})"
    assert_eq "(100_000 : nat)"

    assert_command dfx canister call cycles-ledger send "(record {amount = 100_000;to = principal \"$(dfx canister id cycles-depositor)\"})" --identity alice
    assert_eq "(variant { Ok = 2 : nat })"

    assert_command dfx canister call cycles-ledger icrc1_balance_of "(record {owner = principal \"$ALICE\"})"
    assert_eq "(299_800_000 : nat)"

    assert_command dfx canister status cycles-depositor
    assert_contains "Balance: 9_999_500_100_000 Cycles"
}
