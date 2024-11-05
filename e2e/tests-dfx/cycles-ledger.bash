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
  echo "Alice principal: $(dfx identity get-principal --identity alice)"
  dfx identity new --storage-mode plaintext bob
  echo "Bob principal: $(dfx identity get-principal --identity bob)"
}

teardown() {
  dfx_stop

  standard_teardown
}

start_and_install_nns() {
  dfx_start_for_nns_install

  dfx extension install nns --version 0.4.3
  dfx nns install --ledger-accounts "$(dfx ledger account-id --identity cycle-giver)"
}

add_cycles_ledger_canisters_to_project() {
  jq -s '.[0] * .[1]' ../dfx.json dfx.json | sponge dfx.json
}

deploy_cycles_ledger() {
  assert_command dfx deploy cycles-ledger --specified-id "um5iw-rqaaa-aaaaq-qaaba-cai" --argument '(variant { Init = record { max_blocks_per_request = 100; index_id = null; } })'
  assert_command dfx deploy depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})" --with-cycles 10000000000000 --specified-id "ul4oc-4iaaa-aaaaq-qaabq-cai"
}

current_time_nanoseconds() {
  echo "$(date +%s)"000000000
}

@test "balance" {
  start_and_install_nns

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB=$(dfx identity get-principal --identity bob)

  deploy_cycles_ledger

  assert_command dfx cycles balance --identity alice --precise
  assert_eq "0 cycles."

  assert_command dfx cycles balance --identity alice
  assert_eq "0.000 TC (trillion cycles)."

  assert_command dfx cycles balance --identity bob --precise
  assert_eq "0 cycles."

  assert_command dfx cycles balance --identity bob
  assert_eq "0.000 TC (trillion cycles)."


  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 1_700_400_200_150;})" --identity cycle-giver
  assert_eq "(record { balance = 1_700_400_200_150 : nat; block_index = 0 : nat })"

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 3_750_000_000_000;})" --identity cycle-giver
  assert_eq "(record { balance = 3_750_000_000_000 : nat; block_index = 1 : nat })"

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 760_500_000_000;})" --identity cycle-giver
  assert_eq "(record { balance = 760_500_000_000 : nat; block_index = 2 : nat })"

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_900_000_000_000;})" --identity cycle-giver
  assert_eq "(record { balance = 2_900_000_000_000 : nat; block_index = 3 : nat })"


  assert_command dfx cycles balance --precise --identity alice
  assert_eq "1700400200150 cycles."

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT1"
  assert_eq "3750000000000 cycles."

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "760500000000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2900000000000 cycles."


  assert_command dfx cycles balance --identity alice
  assert_eq "1.700 TC (trillion cycles)."

  assert_command dfx cycles balance --identity alice --subaccount "$ALICE_SUBACCT1"
  assert_eq "3.750 TC (trillion cycles)."

  assert_command dfx cycles balance --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "0.760 TC (trillion cycles)."

  assert_command dfx cycles balance --identity bob
  assert_eq "2.900 TC (trillion cycles)."


  # can see cycles balance of other accounts
  assert_command dfx cycles balance --owner "$ALICE" --identity bob
  assert_eq "1.700 TC (trillion cycles)."

  assert_command dfx cycles balance --owner "$ALICE" --subaccount "$ALICE_SUBACCT1" --identity bob
  assert_eq "3.750 TC (trillion cycles)."

  assert_command dfx cycles balance --owner "$BOB" --identity anonymous
  assert_eq "2.900 TC (trillion cycles)."
}

@test "balance without cycles ledger failed as expected" {
  dfx_start

  assert_command_fail dfx cycles balance
  assert_contains "Cycles ledger with canister ID 'um5iw-rqaaa-aaaaq-qaaba-cai' cannot be found."
}

@test "transfer" {
  start_and_install_nns

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

  deploy_cycles_ledger

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 2_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 1_000_000_000_000;})" --identity cycle-giver

  # account to account
  assert_command dfx cycles balance --precise --identity alice
  assert_eq "3000000000000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "0 cycles."

  assert_command dfx cycles transfer "$BOB" 100000 --identity alice
  assert_eq "Transfer sent at block index 3"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  # account to subaccount
  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999899900000 cycles."
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "0 cycles."

  assert_command dfx cycles transfer "$BOB" 100000 --identity alice --to-subaccount "$BOB_SUBACCT1"
  assert_eq "Transfer sent at block index 4"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999799800000 cycles."

  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "100000 cycles."


  # subaccount to account
  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "1000000000000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  assert_command dfx cycles transfer "$BOB" 700000 --identity alice --from-subaccount "$ALICE_SUBACCT2"
  assert_eq "Transfer sent at block index 5"

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999899300000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "800000 cycles."


  # subaccount to subaccount
  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999899300000 cycles."
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "100000 cycles."

  assert_command dfx cycles transfer "$BOB" 400000 --identity alice --to-subaccount "$BOB_SUBACCT1" --from-subaccount "$ALICE_SUBACCT2"
  assert_eq "Transfer sent at block index 6"

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999798900000 cycles."

  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "500000 cycles."
}

@test "transfer deduplication" {
  start_and_install_nns

  ALICE=$(dfx identity get-principal --identity alice)
  BOB=$(dfx identity get-principal --identity bob)

  deploy_cycles_ledger

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity cycle-giver

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "3000000000000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "0 cycles."

  t=$(current_time_nanoseconds)

  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 1 --identity alice
  assert_eq "Transfer sent at block index 1"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  # same memo and created-at-time: dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 1 --identity alice
  # shellcheck disable=SC2154
  assert_contains "transaction is a duplicate of another transaction in block 1" "$stderr"
  # shellcheck disable=SC2154
  assert_eq "Transfer sent at block index 1" "$stdout"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999899900000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  # different memo and same created-at-time same: not dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time "$t" --memo 2 --identity alice
  assert_contains "Transfer sent at block index 2"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999799800000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "200000 cycles."

  # same memo and different created-at-time same: not dupe
  assert_command dfx cycles transfer "$BOB" 100000 --created-at-time $((t+1)) --memo 1 --identity alice
  assert_contains "Transfer sent at block index 3"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999699700000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "300000 cycles."
}

@test "approve and transfer_from" {
  start_and_install_nns

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  ALICE_SUBACCT2="9C9B9A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2_CANDID="\9C\9B\9A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

  deploy_cycles_ledger

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 3_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 1_000_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT2_CANDID\"};cycles = 1_000_000_000_000;})" --identity cycle-giver

  # account to account
  assert_command dfx cycles balance --precise --identity alice
  assert_eq "3000000000000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "0 cycles."

  t=$(current_time_nanoseconds)
  assert_command dfx cycles approve "$BOB" 2000000000 --created-at-time "$t" --memo 123 --identity alice
  assert_eq "Approval sent at block index 3"
  assert_command dfx cycles approve "$BOB" 2000000000 --created-at-time "$t" --memo 123 --identity alice
  assert_contains "Approval is a duplicate of block 3"
  assert_command dfx cycles transfer "$BOB" 100000 --from "$ALICE" --identity bob
  assert_eq "Transfer sent at block index 4"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999799900000 cycles."

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  # account to subaccount
  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999799900000 cycles."
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "0 cycles."

  t=$(current_time_nanoseconds)
  assert_command dfx cycles transfer "$BOB" 100000 --from "$ALICE" --to-subaccount "$BOB_SUBACCT1" --created-at-time "$t" --identity bob
  assert_eq "Transfer sent at block index 5"
  assert_command dfx cycles transfer "$BOB" 100000 --from "$ALICE" --to-subaccount "$BOB_SUBACCT1" --created-at-time "$t" --identity bob
  assert_contains "Transfer is a duplicate of block index 5"

  assert_command dfx cycles balance --precise --identity alice
  assert_eq "2999699800000 cycles."

  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "100000 cycles."

  # subaccount to account
  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT1"
  assert_eq "1000000000000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "100000 cycles."

  assert_command dfx cycles approve "$BOB" 200000000000 --from-subaccount "$ALICE_SUBACCT1" --identity alice
  assert_eq "Approval sent at block index 6"
  assert_command dfx cycles transfer "$BOB" 700000 --from "$ALICE" --from-subaccount "$ALICE_SUBACCT1" --identity bob
  assert_eq "Transfer sent at block index 7"

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT1"
  assert_eq "999799300000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "800000 cycles."

  # spender subaccount
  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "1000000000000 cycles."

  assert_command dfx cycles approve "$BOB" 200000000000 --spender-subaccount "$BOB_SUBACCT1" --from-subaccount "$ALICE_SUBACCT2" --identity alice
  assert_eq "Approval sent at block index 8"
  assert_command dfx cycles transfer "$BOB" 300000 --from "$ALICE" --from-subaccount "$ALICE_SUBACCT2"  --spender-subaccount "$BOB_SUBACCT1" --identity bob
  assert_eq "Transfer sent at block index 9"

  assert_command dfx cycles balance --precise --identity alice --subaccount "$ALICE_SUBACCT2"
  assert_eq "999799700000 cycles."
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "1100000 cycles."
}

@test "top up canister principal check" {
  start_and_install_nns

  BOB=$(dfx identity get-principal --identity bob)

  deploy_cycles_ledger

  assert_command_fail dfx cycles top-up "$BOB" 600000 --identity alice
  assert_contains "Invalid receiver: $BOB.  Make sure the receiver is a canister."
}

@test "top-up and deposit-cycles" {
  start_and_install_nns

  dfx_new
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB_SUBACCT2="6C6B6A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT2_CANDID="\6C\6B\6A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  deploy_cycles_ledger

  assert_command dfx deploy

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT2_CANDID\"};cycles = 2_700_000_000_000;})" --identity cycle-giver

  # shellcheck disable=SC2030
  export DFX_DISABLE_AUTO_WALLET=1

  # account to canister
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2400000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_000_000 Cycles"

  assert_command dfx cycles top-up e2e_project_backend 100000 --identity bob
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399899900000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_100_000 Cycles"

  assert_command dfx canister deposit-cycles 100000 e2e_project_backend --identity bob
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399799800000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_200_000 Cycles"

  # subaccount to canister
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "2600000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_200_000 Cycles"

  assert_command dfx cycles top-up e2e_project_backend 300000 --identity bob --from-subaccount "$BOB_SUBACCT1"
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "2599899700000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_500_000 Cycles"

  assert_command dfx canister deposit-cycles 300000 e2e_project_backend --identity bob --from-subaccount "$BOB_SUBACCT1"
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT1"
  assert_eq "2599799400000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_800_000 Cycles"

  # subaccount to canister - by canister id
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT2"
  assert_eq "2700000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_800_000 Cycles"

  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" 600000 --identity bob --from-subaccount "$BOB_SUBACCT2"
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT2"
  assert_eq "2699899400000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_001_400_000 Cycles"

  assert_command dfx canister deposit-cycles 600000 "$(dfx canister id e2e_project_backend)" --identity bob --from-subaccount "$BOB_SUBACCT2"
  assert_command dfx cycles balance --precise --identity bob --subaccount "$BOB_SUBACCT2"
  assert_eq "2699798800000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_002_000_000 Cycles"

  # deduplication
  t=$(current_time_nanoseconds)
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399799800000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_002_000_000 Cycles"

  assert_command dfx canister deposit-cycles 100000 e2e_project_backend --identity bob --created-at-time "$t"
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399699700000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_002_100_000 Cycles"

  assert_command dfx canister deposit-cycles 100000 e2e_project_backend --identity bob --created-at-time "$t"
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399699700000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_002_100_000 Cycles"
}

@test "top-up deduplication" {
  start_and_install_nns

  dfx_new
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  BOB=$(dfx identity get-principal --identity bob)
  BOB_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"
  BOB_SUBACCT2="6C6B6A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  BOB_SUBACCT2_CANDID="\6C\6B\6A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  deploy_cycles_ledger

  assert_command dfx deploy

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\";};cycles = 2_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$BOB\"; subaccount = opt blob \"$BOB_SUBACCT2_CANDID\"};cycles = 2_700_000_000_000;})" --identity cycle-giver

  # account to canister
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2400000000000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_000_000 Cycles"

  t=$(current_time_nanoseconds)
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time "$t" 100000 --identity bob
  assert_eq "Transfer sent at block index 3"

  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399899900000 cycles."
  assert_command dfx canister status e2e_project_backend
  assert_contains "Balance: 3_100_000_100_000 Cycles"

  # same created-at-time: dupe
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time "$t" 100000 --identity bob
  # shellcheck disable=SC2154
  assert_contains "transaction is a duplicate of another transaction in block 3" "$stderr"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block index 3" "$stdout"
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399899900000 cycles."

  # different created-at-time: not dupe
  assert_command dfx cycles top-up "$(dfx canister id e2e_project_backend)" --created-at-time $((t+1)) 100000 --identity bob
  assert_eq "Transfer sent at block index 4"
  assert_command dfx cycles balance --precise --identity bob
  assert_eq "2399799800000 cycles."
}

@test "howto" {
  start_and_install_nns

  # This is the equivalent of https://www.notion.so/dfinityorg/How-to-install-and-test-the-cycles-ledger-521c9f3c410f4a438514a03e35464299
  ALICE=$(dfx identity get-principal --identity alice)
  BOB=$(dfx identity get-principal --identity bob)

  deploy_cycles_ledger

  assert_command dfx ledger balance --identity cycle-giver
  assert_eq "1000000000.00000000 ICP"

  assert_command dfx canister status depositor
  assert_contains "Balance: 10_000_000_000_000 Cycles"

  dfx canister status depositor

  assert_command dfx cycles balance --identity alice --precise
  assert_eq "0 cycles."

  assert_command dfx cycles balance --identity bob --precise
  assert_eq "0 cycles."


  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 500_000_000;})" --identity cycle-giver
  assert_eq "(record { balance = 500_000_000 : nat; block_index = 0 : nat })"

  assert_command dfx canister status depositor
  assert_contains "Balance: 9_999_500_000_000 Cycles"

  assert_command dfx cycles balance --identity alice --precise
  assert_eq "500000000 cycles."

  assert_command dfx cycles transfer "$BOB" 100000 --identity alice
  assert_eq "Transfer sent at block index 1"

  assert_command dfx cycles balance --identity alice --precise
  assert_eq "399900000 cycles."

  assert_command dfx cycles balance --identity bob --precise
  assert_eq "100000 cycles."

  assert_command dfx cycles top-up depositor 100000 --identity alice
  assert_eq "Transfer sent at block index 2"

  assert_command dfx cycles balance --identity alice --precise
  assert_eq "299800000 cycles."

  assert_command dfx canister status depositor
  assert_contains "Balance: 9_999_500_100_000 Cycles"
}

@test "canister creation" {
  start_and_install_nns
  
  dfx_new temporary
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT1_CANDID="\7C\7B\7A\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f"

  assert_command deploy_cycles_ledger
  CYCLES_LEDGER_ID=$(dfx canister id cycles-ledger)
  echo "Cycles ledger deployed at id $CYCLES_LEDGER_ID"
  assert_command dfx deploy depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})"
  echo "Cycles depositor deployed at id $(dfx canister id depositor)"
  assert_command dfx ledger fabricate-cycles --canister depositor --t 9999

  assert_command dfx deploy

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 13_400_000_000_000;})" --identity cycle-giver
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\"; subaccount = opt blob \"$ALICE_SUBACCT1_CANDID\"};cycles = 2_600_000_000_000;})" --identity cycle-giver

  cd ..
  dfx_new
  # setup done

  # using dfx canister create
  dfx identity use alice
  # shellcheck disable=SC2030,SC2031
  export DFX_DISABLE_AUTO_WALLET=1
  t=$(current_time_nanoseconds)
  assert_command dfx canister create e2e_project_backend --with-cycles 1T --created-at-time "$t"
  assert_command dfx canister id e2e_project_backend
  E2E_PROJECT_BACKEND_CANISTER_ID=$(dfx canister id e2e_project_backend)
  assert_command dfx cycles balance --precise
  assert_eq "12399900000000 cycles."
  # forget about canister. If --created-at-time is a valid idempotency key we should end up with the same canister id
  rm .dfx/local/canister_ids.json
  assert_command dfx canister create e2e_project_backend --with-cycles 1T --created-at-time "$t"
  assert_command dfx canister id e2e_project_backend
  assert_contains "$E2E_PROJECT_BACKEND_CANISTER_ID"
  assert_command dfx cycles balance --precise
  assert_eq "12399900000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend --no-withdrawal

  assert_command dfx canister create e2e_project_backend --with-cycles 0.5T --from-subaccount "$ALICE_SUBACCT1"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --subaccount "$ALICE_SUBACCT1" --precise
  assert_eq "2099900000000 cycles."
  
  # reset deployment status
  rm -r .dfx

  # using dfx deploy
  t=$(current_time_nanoseconds)
  assert_command dfx deploy e2e_project_backend --with-cycles 1T --created-at-time "$t"
  assert_command dfx canister id e2e_project_backend
  E2E_PROJECT_BACKEND_CANISTER_ID=$(dfx canister id e2e_project_backend)
  assert_command dfx cycles balance --precise
  assert_eq "11399800000000 cycles."
  # reset and forget about canister. If --created-at-time is a valid idempotency key we should end up with the same canister id
  dfx canister uninstall-code e2e_project_backend
  rm .dfx/local/canister_ids.json
  assert_command dfx deploy e2e_project_backend --with-cycles 1T --created-at-time "$t" -vv
  assert_command dfx canister id e2e_project_backend
  assert_contains "$E2E_PROJECT_BACKEND_CANISTER_ID"
  assert_command dfx cycles balance --precise
  assert_eq "11399800000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend --no-withdrawal
  
  assert_command dfx deploy e2e_project_backend --with-cycles 0.5T --from-subaccount "$ALICE_SUBACCT1"
  assert_command dfx canister id e2e_project_backend
  assert_command dfx cycles balance --subaccount "$ALICE_SUBACCT1" --precise
  assert_eq "1599800000000 cycles."
  dfx canister stop e2e_project_backend
  dfx canister delete e2e_project_backend --no-withdrawal
}

@test "canister deletion" {
  start_and_install_nns

  dfx_new temporary
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  ALICE=$(dfx identity get-principal --identity alice)

  assert_command deploy_cycles_ledger
  CYCLES_LEDGER_ID=$(dfx canister id cycles-ledger)
  echo "Cycles ledger deployed at id $CYCLES_LEDGER_ID"
  assert_command dfx deploy depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})"
  echo "Cycles depositor deployed at id $(dfx canister id depositor)"
  assert_command dfx ledger fabricate-cycles --canister depositor --t 9999
  assert_command dfx deploy
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 22_400_000_000_000;})" --identity cycle-giver

  cd ..
  dfx_new
  # setup done

  dfx identity use alice
  # shellcheck disable=SC2030,SC2031
  export DFX_DISABLE_AUTO_WALLET=1
  assert_command dfx canister create --all

  # delete by name
  assert_command dfx canister stop --all
  assert_command dfx canister delete e2e_project_backend
  assert_contains "Successfully withdrew"

  # delete by id
  assert_command dfx canister create --all
  CANISTER_ID=$(dfx canister id e2e_project_backend)
  rm .dfx/local/canister_ids.json
  assert_command dfx canister stop "${CANISTER_ID}"
  assert_command dfx canister delete "${CANISTER_ID}"
  assert_contains "Successfully withdrew"
}

@test "redeem-faucet-coupon redeems into the cycles ledger" {
  start_and_install_nns

  assert_command deploy_cycles_ledger
  dfx_new hello
  install_asset faucet
  dfx deploy
  dfx ledger fabricate-cycles --canister faucet --t 1000

  dfx identity new --storage-mode plaintext no_wallet_identity
  dfx identity use no_wallet_identity
  SUBACCOUNT="7C7B7A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

  assert_command dfx cycles balance
  assert_eq "0.000 TC (trillion cycles)."
  assert_command dfx cycles balance --subaccount "$SUBACCOUNT"
  assert_eq "0.000 TC (trillion cycles)."

  assert_command dfx cycles redeem-faucet-coupon --faucet "$(dfx canister id faucet)" 'valid-coupon'
  assert_match "Redeemed coupon 'valid-coupon'"
  assert_command dfx cycles redeem-faucet-coupon --faucet "$(dfx canister id faucet)" 'another-valid-coupon'
  assert_match "Redeemed coupon 'another-valid-coupon'"
  assert_command dfx cycles balance
  assert_eq "20.000 TC (trillion cycles)."

  # with subaccount
  assert_command dfx cycles redeem-faucet-coupon --faucet "$(dfx canister id faucet)" 'another-valid-coupon' --to-subaccount "$SUBACCOUNT"
  assert_match "Redeemed coupon 'another-valid-coupon'"
  assert_command dfx cycles balance --subaccount "$SUBACCOUNT"
  assert_eq "10.000 TC (trillion cycles)."
}

@test "create canister on specific subnet" {
  start_and_install_nns
  
  dfx_new temporary
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters

  ALICE=$(dfx identity get-principal --identity alice)

  assert_command deploy_cycles_ledger
  CYCLES_LEDGER_ID=$(dfx canister id cycles-ledger)
  echo "Cycles ledger deployed at id $CYCLES_LEDGER_ID"
  assert_command dfx deploy depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})"
  echo "Cycles depositor deployed at id $(dfx canister id depositor)"
  assert_command dfx ledger fabricate-cycles --canister depositor --t 9999

  assert_command dfx deploy

  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 13_400_000_000_000;})" --identity cycle-giver
  cd ..
  dfx_new 

  dfx identity use alice
  # shellcheck disable=SC2030,SC2031
  export DFX_DISABLE_AUTO_WALLET=1
  # setup done

  # use --subnet <principal>
  SUBNET_ID="5kdm2-62fc6-fwnja-hutkz-ycsnm-4z33i-woh43-4cenu-ev7mi-gii6t-4ae" # a random, valid principal
  assert_command_fail dfx canister create e2e_project_backend --subnet "$SUBNET_ID"
  assert_contains "Subnet $SUBNET_ID does not exist"
  
  # use --subnet-type
  assert_command_fail dfx canister create e2e_project_backend --subnet-type custom_subnet_type
  assert_contains "Provided subnet type custom_subnet_type does not exist"
}

@test "automatically choose subnet" {
  [[ "$USE_POCKETIC" ]] && skip "skipped for pocketic: subnet range"
  dfx_start

  REGISTRY="rwlgt-iiaaa-aaaaa-aaaaa-cai"
  CMC="rkp4c-7iaaa-aaaaa-aaaca-cai"
  ALICE=$(dfx identity get-principal --identity alice)
  dfx_new temporary
  install_asset fake_registry
  dfx deploy fake_registry --specified-id "$REGISTRY"
  add_cycles_ledger_canisters_to_project
  install_cycles_ledger_canisters
  assert_command deploy_cycles_ledger
  CYCLES_LEDGER_ID=$(dfx canister id cycles-ledger)
  echo "Cycles ledger deployed at id $CYCLES_LEDGER_ID"
  assert_command dfx deploy depositor --argument "(record {ledger_id = principal \"$(dfx canister id cycles-ledger)\"})"
  echo "Cycles depositor deployed at id $(dfx canister id depositor)"
  assert_command dfx ledger fabricate-cycles --canister depositor --t 9999
  assert_command dfx canister call depositor deposit "(record {to = record{owner = principal \"$ALICE\";};cycles = 99_000_000_000_000;})"
  install_asset fake_cmc
  dfx deploy fake-cmc --specified-id "$CMC"
  cd ..
  # shellcheck disable=SC2030,SC2031
  export DFX_DISABLE_AUTO_WALLET=1
  dfx identity use alice
  dfx_new

  SUBNET1="iqd74-4xnai"
  SUBNET2="2myss-nlbai"

  jq '.canisters.one = { "main": "src/e2e_project_backend/main.mo", "type": "motoko" }' dfx.json | sponge dfx.json
  jq '.canisters.two = { "main": "src/e2e_project_backend/main.mo", "type": "motoko" }' dfx.json | sponge dfx.json
  # setup done


  # no other canisters already exist
  assert_command dfx canister create e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet_selection = null"
  stop_and_delete e2e_project_backend

  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet_selection = null"
  stop_and_delete e2e_project_backend

  # one other canister already exists
  assert_command dfx canister create one --subnet aaaaa-aa
  ONE_ID="$(dfx canister id one)"
  echo "Canister one: $ONE_ID"
  assert_command dfx canister call "$REGISTRY" set_subnet_for_canister "(vec { record {0 = principal \"$ONE_ID\"; 1 = principal \"$SUBNET1\"} })"

  assert_command dfx canister create e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend

  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend

  # multiple other canisters already exist - all on same subnet
  assert_command dfx canister create two --subnet aaaaa-aa
  TWO_ID="$(dfx canister id two)"
  echo "Canister two: $TWO_ID"
  assert_command dfx canister call "$REGISTRY" set_subnet_for_canister "(vec { record {0 = principal \"$ONE_ID\"; 1 = principal \"$SUBNET1\"}; record { 0 = principal \"$TWO_ID\"; 1 = principal \"$SUBNET1\"} })"

  assert_command dfx canister create e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend

  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend

  # multiple other canisters already exist - not all on same subnet
  assert_command dfx canister call "$REGISTRY" set_subnet_for_canister "(vec { record {0 = principal \"$ONE_ID\"; 1 = principal \"$SUBNET1\"}; record { 0 = principal \"$TWO_ID\"; 1 = principal \"$SUBNET2\"} })"

  assert_command_fail dfx canister create e2e_project_backend
  assert_contains "Cannot automatically decide which subnet to target."

  assert_command_fail dfx deploy e2e_project_backend
  assert_contains "Cannot automatically decide which subnet to target."
  
  # still can create if a subnet is specified
  assert_command dfx canister create e2e_project_backend --subnet "$SUBNET2"
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET2\""
  stop_and_delete e2e_project_backend

  assert_command dfx deploy e2e_project_backend --subnet "$SUBNET2"
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET2\""
  stop_and_delete e2e_project_backend

  # remote canister exists on different subnet
  THREE_ID="3333u-aiaaa-aaaar-avzbq-cai"
  jq '.canisters.three = { "main": "src/e2e_project_backend/main.mo", "type": "motoko", "remote" : { "candid": "", "id": { "local": "'$THREE_ID'" } } }' dfx.json | sponge dfx.json
  assert_command dfx canister call "$REGISTRY" set_subnet_for_canister "(vec { record {0 = principal \"$ONE_ID\"; 1 = principal \"$SUBNET1\"}; record { 0 = principal \"$TWO_ID\"; 1 = principal \"$SUBNET1\"}; record { 0 = principal \"$THREE_ID\"; 1 = principal \"$SUBNET2\"} })"

  assert_command dfx canister create e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend

  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister call "$CMC" last_create_canister_args --query
  assert_contains "subnet = principal \"$SUBNET1\""
  stop_and_delete e2e_project_backend
}

@test "convert icp to cycles" {
  start_and_install_nns

  ALICE=$(dfx identity get-principal --identity alice)
  ALICE_SUBACCT1="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
  ALICE_SUBACCT2="6C6B6A030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

  deploy_cycles_ledger

  assert_command dfx --identity cycle-giver ledger transfer --memo 1234 --amount 100 "$(dfx ledger account-id --of-principal "$ALICE")"
  assert_command dfx --identity cycle-giver ledger transfer --memo 1234 --amount 100 "$(dfx ledger account-id --of-principal "$ALICE" --subaccount "$ALICE_SUBACCT1")"

  dfx identity use alice
  assert_command dfx ledger balance
  assert_eq "100.00000000 ICP"
  assert_command dfx ledger balance --subaccount "$ALICE_SUBACCT1"
  assert_eq "100.00000000 ICP"
  assert_command dfx cycles balance --precise
  assert_eq "0 cycles."

  dfx canister call rrkah-fqaaa-aaaaa-aaaaq-cai get_proposal_info '(3 : nat64)'

  # base case
  assert_command dfx cycles convert --amount 12.5
  assert_contains "Account was topped up with 1_543_208_750_000_000 cycles!"
  assert_command dfx ledger balance
  assert_eq "87.49990000 ICP"
  assert_command dfx cycles balance --precise
  assert_eq "1543208750000000 cycles."

  # to-subaccount and from-subaccount
  assert_command dfx cycles convert --amount 10 --from-subaccount "$ALICE_SUBACCT1" --to-subaccount "$ALICE_SUBACCT2"
  assert_contains "Account was topped up with 1_234_567_000_000_000 cycles!"
  assert_command dfx ledger balance --subaccount "$ALICE_SUBACCT1"
  assert_eq "89.99990000 ICP"
  assert_command dfx cycles balance --precise --subaccount "$ALICE_SUBACCT2"
  assert_eq "1234567000000000 cycles."

  # deduplication
  t=$(current_time_nanoseconds)
  assert_command dfx cycles convert --amount 10 --created-at-time "$t"
  assert_contains "Transfer sent at block height 12"
  assert_command dfx cycles balance --precise
  assert_eq "2777775750000000 cycles."
  # same created-at-time: dupe
  assert_command dfx cycles convert --amount 10 --created-at-time "$t"
  # shellcheck disable=SC2154
  assert_contains "transaction is a duplicate of another transaction in block 12" "$stderr"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block height 12"
}
