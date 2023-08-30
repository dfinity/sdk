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

@test "cycles ledger balance" {
    ALICE=$(dfx identity get-principal --identity alice)
    ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
    ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
    BOB=$(dfx identity get-principal --identity bob)

    assert_command dfx deploy cycles-ledger
    assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
    assert_eq "0 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice
    assert_eq "0.000 TC (trillion cycles)."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob --precise
    assert_eq "0 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
    assert_eq "0.000 TC (trillion cycles)."


    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 1_700_400_200_150;})" --identity cycle-giver
    assert_eq "(record { balance = 1_700_400_200_150 : nat; txid = 0 : nat })"

    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 3_750_000_000_000;})" --identity cycle-giver
    assert_eq "(record { balance = 3_750_000_000_000 : nat; txid = 1 : nat })"

    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 760_500_000_000;})" --identity cycle-giver
    assert_eq "(record { balance = 760_500_000_000 : nat; txid = 2 : nat })"

    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_900_000_000_000;})" --identity cycle-giver
    assert_eq "(record { balance = 2_900_000_000_000 : nat; txid = 3 : nat })"


    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
    assert_eq "1700400200150 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --precise --identity alice --subaccount "$ALICE_SUBACCT1"
    assert_eq "3750000000000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --precise --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "760500000000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --precise --identity bob
    assert_eq "2900000000000 cycles."


    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice
    assert_eq "1.700 TC (trillion cycles)."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --identity alice --subaccount "$ALICE_SUBACCT1"
    assert_eq "3.750 TC (trillion cycles)."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "0.760 TC (trillion cycles)."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
    assert_eq "2.900 TC (trillion cycles)."


    # can see cycles balance of other accounts
    assert_command dfx cycles balance --owner "$ALICE" --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
    assert_eq "1.700 TC (trillion cycles)."

    assert_command dfx cycles balance --owner "$ALICE" --subaccount "$ALICE_SUBACCT1" --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
    assert_eq "3.750 TC (trillion cycles)."

    assert_command dfx cycles balance --owner "$BOB" --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"  --identity anonymous
    assert_eq "2.900 TC (trillion cycles)."
}

@test "cycles ledger transfer" {
    copy_cycles_ledger

    ALICE=$(dfx identity get-principal --identity alice)
    ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
    ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
    BOB=$(dfx identity get-principal --identity bob)
    BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

    assert_command dfx deploy cycles-ledger
    assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity icp-giver
    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 2_000_000_000_000;})" --identity icp-giver
    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 1_000_000_000_000;})" --identity icp-giver

    # account to account
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
    assert_eq "3000000000000 cycles."
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
    assert_eq "0 cycles."

    # assert_command dfx canister call cycles-ledger icrc1_transfer "(record {to = record{owner = principal \"$BOB\"}; amount = 100_000;})" --identity alice
    assert_command dfx cycles transfer 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-owner "$BOB"
    # assert_eq "(variant { Ok = 3 : nat })"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
    assert_eq "2999899900000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
    assert_eq "100000 cycles."

    # account to subaccount
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
    assert_eq "2999899900000 cycles."
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
    assert_eq "0 cycles."

    # assert_command dfx canister call cycles-ledger icrc1_transfer "(record {to = record{owner = principal \"$BOB\"}; amount = 100_000;})" --identity alice
    assert_command dfx cycles transfer 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-owner "$BOB" --to-subaccount "$BOB_SUBACCT1"
    # assert_eq "(variant { Ok = 3 : nat })"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
    assert_eq "2999799800000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
    assert_eq "100000 cycles."


    # subaccount to account
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "1000000000000 cycles."
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
    assert_eq "100000 cycles."

    # assert_command dfx canister call cycles-ledger icrc1_transfer "(record {to = record{owner = principal \"$BOB\"}; amount = 100_000;})" --identity alice
    assert_command dfx cycles transfer 700000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-owner "$BOB" --from-subaccount "$ALICE_SUBACCT2"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "999899300000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
    assert_eq "800000 cycles."


    # subaccount to subaccount
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "999899300000 cycles."
    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
    assert_eq "100000 cycles."

    # assert_command dfx canister call cycles-ledger icrc1_transfer "(record {to = record{owner = principal \"$BOB\"}; amount = 100_000;})" --identity alice
    assert_command dfx cycles transfer 400000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-owner "$BOB" --to-subaccount "$BOB_SUBACCT1" --from-subaccount "$ALICE_SUBACCT2"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
    assert_eq "999798900000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
    assert_eq "500000 cycles."
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

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
    assert_eq "0 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob --precise
    assert_eq "0 cycles."


    assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 500_000_000;})" --identity cycle-giver
    assert_eq "(record { balance = 500_000_000 : nat; txid = 0 : nat })"

    assert_command dfx canister status cycles-depositor
    assert_contains "Balance: 9_999_500_000_000 Cycles"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
    assert_eq "500000000 cycles."

    assert_command dfx cycles transfer 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-owner "$BOB"
    assert_eq "Transfer sent at block index 1"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
    assert_eq "399900000 cycles."

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob --precise
    assert_eq "100000 cycles."

    assert_command dfx canister call cycles-ledger send "(record {amount = 100_000;to = principal \"$(dfx canister id cycles-depositor)\"})" --identity alice
    assert_eq "(variant { Ok = 2 : nat })"

    assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
    assert_eq "299800000 cycles."

    assert_command dfx canister status cycles-depositor
    assert_contains "Balance: 9_999_500_100_000 Cycles"
}
