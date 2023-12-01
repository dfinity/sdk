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

add_cycles_ledger_canisters_to_project() {
  jq -s '.[0] * .[1]' ../dfx.json dfx.json | sponge dfx.json
}

current_time_nanoseconds() {
  echo "$(date +%s)"000000000
}

@test "balance" {
  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB=$(dfx identity get-principal --identity bob)

  assert_command deploy_cycles_ledger
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

@test "transfer" {
  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

  assert_command deploy_cycles_ledger
  assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 2_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 1_000_000_000_000;})" --identity cycle-giver

  # account to account
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
  assert_eq "3000000000000 cycles."
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "0 cycles."

  assert_command dfx cycles transfer "$BOB" 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice
  assert_eq "Transfer sent at block index 3"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "100000 cycles."

  # account to subaccount
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
  assert_eq "2999899900000 cycles."
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "0 cycles."

  assert_command dfx cycles transfer "$BOB" 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-subaccount "$BOB_SUBACCT1"
  assert_eq "Transfer sent at block index 4"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice
  assert_eq "2999799800000 cycles."

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "100000 cycles."


  # subaccount to account
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "1000000000000 cycles."
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "100000 cycles."

  assert_command dfx cycles transfer "$BOB" 700000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --from-subaccount "$ALICE_SUBACCT2"
  assert_eq "Transfer sent at block index 5"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999899300000 cycles."

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "800000 cycles."


  # subaccount to subaccount
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999899300000 cycles."
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "100000 cycles."

  assert_command dfx cycles transfer "$BOB" 400000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --to-subaccount "$BOB_SUBACCT1" --from-subaccount "$ALICE_SUBACCT2"
  assert_eq "Transfer sent at block index 6"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999798900000 cycles."

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "500000 cycles."
}

@test "transfer deduplication" {
  ALICE=$(dfx identity get-principal --identity alice)
  BOB=$(dfx identity get-principal --identity bob)

  assert_command deploy_cycles_ledger
  assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity cycle-giver

  assert_command dfx cycles balance  --precise --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "3000000000000 cycles."

  assert_command dfx cycles balance --precise --identity bob --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "0 cycles."

  t=$(current_time_nanoseconds)

  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 1 --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "Transfer sent at block index 1"

  assert_command dfx cycles balance --precise --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --precise --identity bob --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "100000 cycles."

  # same memo and created-at-time: dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 1 --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  # shellcheck disable=SC2154
  assert_contains "transaction is a duplicate of another transaction in block 1" "$stderr"
  # shellcheck disable=SC2154
  assert_eq "Transfer sent at block index 1" "$stdout"

  assert_command dfx cycles balance --precise --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --precise --identity bob --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "100000 cycles."

  # different memo and same created-at-time same: not dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 2 --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_contains "Transfer sent at block index 2"

  assert_command dfx cycles balance --precise --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "2999799800000 cycles."

  assert_command dfx cycles balance --precise --identity bob --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "200000 cycles."

  # same memo and different created-at-time same: not dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time $((t+1)) --memo 1 --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_contains "Transfer sent at block index 3"

  assert_command dfx cycles balance --precise --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "2999699700000 cycles."

  assert_command dfx cycles balance --precise --identity bob --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_eq "300000 cycles."
}

@test "top up canister principal check" {
  BOB=$(dfx identity get-principal --identity bob)

  assert_command deploy_cycles_ledger

  assert_command_fail dfx cycles top-up "$BOB" 600000 --identity alice --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)"
  assert_contains "Invalid receiver: $BOB.  Make sure the receiver is a canister."
}

@test "top-up" {
  dfx_new
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB_SUBACCT2="6C6B6A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT2_CANDID="\6C\6B\6A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  assert_command deploy_cycles_ledger
  assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

  assert_command dfx deploy

  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT2_CANDID\"};cycles = 2_700_000_000_000;})" --identity cycle-giver

  # account to canister
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2400000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_000_000 Cycles"

  assert_command dfx cycles top-up e2e_project_backend 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2399899900000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_100_000 Cycles"

  # subaccount to canister
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "2600000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_100_000 Cycles"

  assert_command dfx cycles top-up e2e_project_backend 300000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob  --from-subaccount "$BOB_SUBACCT1"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob  --subaccount "$BOB_SUBACCT1"
  assert_eq "2599899700000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_400_000 Cycles"

  # subaccount to canister - by canister id
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob --subaccount "$BOB_SUBACCT2"
  assert_eq "2700000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_400_000 Cycles"

  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" 600000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob  --from-subaccount "$BOB_SUBACCT2"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob  --subaccount "$BOB_SUBACCT2"
  assert_eq "2699899400000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_001_000_000 Cycles"
}

@test "top-up deduplication" {
  dfx_new
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB_SUBACCT2="6C6B6A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT2_CANDID="\6C\6B\6A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  assert_command deploy_cycles_ledger
  assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000

  assert_command dfx deploy

  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT2_CANDID\"};cycles = 2_700_000_000_000;})" --identity cycle-giver

  # account to canister
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2400000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_000_000 Cycles"

  t=$(current_time_nanoseconds)
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time "$t" 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
  assert_eq "Transfer sent at block index 3"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2399899900000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_100_000 Cycles"

  # same created-at-time: dupe
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time "$t" 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
  # shellcheck disable=SC2154
  assert_contains "transaction is a duplicate of another transaction in block 3" "$stderr"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block index 3" "$stdout"
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2399899900000 cycles."

  # different created-at-time: not dupe
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time $((t+1)) 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob
  assert_eq "Transfer sent at block index 4"
  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --precise --identity bob
  assert_eq "2399799800000 cycles."
}

@test "howto" {
  # This is the equivalent of https://www.notion.so/dfinityorg/How-to-install-and-test-the-cycles-ledger-521c9f3c410f4a438514a03e35464299
  ALICE=$(dfx identity get-principal --identity alice)
  BOB=$(dfx identity get-principal --identity bob)

  assert_command deploy_cycles_ledger
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

  assert_command dfx cycles transfer "$BOB" 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice
  assert_eq "Transfer sent at block index 1"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
  assert_eq "399900000 cycles."

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity bob --precise
  assert_eq "100000 cycles."

  assert_command dfx cycles top-up cycles-depositor 100000 --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice
  assert_eq "Transfer sent at block index 2"

  assert_command dfx cycles balance --cycles-ledger-canister-id "$(dfx canister id cycles-ledger)" --identity alice --precise
  assert_eq "299800000 cycles."

  assert_command dfx canister status cycles-depositor
  assert_contains "Balance: 9_999_500_100_000 Cycles"
}

@test "canister creation" {
  # skip "can't be properly tested with feature flag turned off (`CYCLES_LEDGER_ENABLED`). TODO(SDK-1331): re-enable this test"
  dfx_new temporary
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  assert_command deploy_cycles_ledger
  CYCLES_LEDGER_ID=$(dfx canister id cycles-ledger)
  echo "Cycles ledger deployed at id $CYCLES_LEDGER_ID"
  assert_command dfx deploy cycles-depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000
  echo "Cycles depositor deployed at id $(dfx canister id cycles-depositor)"

  assert_command dfx deploy

  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 2_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call cycles-depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver

  cd ..
  dfx_new
  # setup done

  # using dfx canister create
  dfx identity use alice
  export DFX_DISABLE_AUTO_WALLET=1
  assert_command dfx canister create e2e_project_backend --with-cycles 1T --cycles-ledger-canister-id "$CYCLES_LEDGER_ID"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --cycles-ledger-canister-id "$CYCLES_LEDGER_ID" --precise
  assert_eq "1399900000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend

  assert_command dfx canister create e2e_project_backend --with-cycles 0.5T --from-subaccount "$ALICE_SUBACCT1" --cycles-ledger-canister-id "$CYCLES_LEDGER_ID"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --cycles-ledger-canister-id "$CYCLES_LEDGER_ID" --subaccount "$ALICE_SUBACCT1" --precise
  assert_eq "2099900000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend

  # using dfx deploy
  assert_command dfx deploy e2e_project_backend --with-cycles 1T --cycles-ledger-canister-id "$CYCLES_LEDGER_ID"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --cycles-ledger-canister-id "$CYCLES_LEDGER_ID" --precise
  assert_eq "399800000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend
  
  assert_command dfx deploy e2e_project_backend --with-cycles 0.5T --from-subaccount "$ALICE_SUBACCT1" --cycles-ledger-canister-id "$CYCLES_LEDGER_ID"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --cycles-ledger-canister-id "$CYCLES_LEDGER_ID" --subaccount "$ALICE_SUBACCT1" --precise
  assert_eq "1599800000000 cycles."
}