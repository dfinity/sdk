#!/usr/bin/env bats

load ../utils/_

export ALICE_ACCOUNT_ID="345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752"
export BOB_ACCOUNT_ID="22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89"
export BOB_SUBACCOUNT_ID="5a94fe181e9d411c58726cb87cbf2d016241b6c350bc3330e4869ca76e54ecbc"

setup() {
  standard_setup
  install_asset ledger

  dfx identity import --storage-mode plaintext alice alice.pem
  dfx identity import --storage-mode plaintext bob bob.pem
  dfx identity import --storage-mode plaintext david david.pem
}

# Top up alice, bob and bob-sub accounts with 1,000,000 ICP each.
# This method must be called after dfx_start
prepare_accounts() {
  # The pocket-ic instance has the ICP ledger canister pre-installed 
  # in which the account of the annonymous identity has 1,000,000,000 ICP.
  dfx ledger --identity anonymous transfer --memo 1 --icp 1000000 "$ALICE_ACCOUNT_ID"
  dfx ledger --identity anonymous transfer --memo 1 --icp 1000000 "$BOB_ACCOUNT_ID"
  dfx ledger --identity anonymous transfer --memo 1 --icp 1000000 "$BOB_SUBACCOUNT_ID" 
}

teardown() {
  dfx_stop

  standard_teardown
}

current_time_nanoseconds() {
  echo "$(date +%s)"000000000
}

@test "ledger account-id" {
  dfx_start --system-canisters

  dfx identity use alice
  assert_command dfx ledger account-id
  assert_match "$ALICE_ACCOUNT_ID"

  assert_command dfx ledger account-id --of-principal fg7gi-vyaaa-aaaal-qadca-cai
  assert_match a014842f64a22e59887162a79c7ca7eb02553250704780ec4d954f12d0ea0b18

  ALICE_PRINCIPAL="$(dfx identity get-principal)"
  assert_command dfx ledger account-id --of-canister qvhpv-4qaaa-aaaaa-aaagq-cai --subaccount-from-principal "${ALICE_PRINCIPAL}"
  # value obtained by running `dfx --identity alice canister call --ic qvhpv-4qaaa-aaaaa-aaagq-cai get_payment_subaccount`
  assert_match 7afe37275178a26c463a6609825748ba3ed3572f7f308917f96f9f7be20e9d01

  # --of-canister accepts both canister alias and canister principal
  assert_command dfx canister create dummy_canister
  assert_command dfx ledger account-id --of-canister "$(dfx canister id dummy_canister)"
  assert_eq "$(dfx ledger account-id --of-canister dummy_canister)"
}

@test "ledger balance & transfer" {
  dfx_start --system-canisters
  prepare_accounts

  dfx identity use alice
  assert_command dfx ledger account-id
  assert_eq "$ALICE_ACCOUNT_ID"

  assert_command dfx ledger balance
  assert_eq "1000000.00000000 ICP"

  assert_command dfx ledger transfer --amount 100 --memo 1 "$BOB_ACCOUNT_ID"
  assert_contains "Transfer sent at block height"

  # The sender(alice) paid transaction fee which is 0.0001 ICP
  assert_command dfx ledger balance
  assert_eq "999899.99990000 ICP"

  dfx identity use bob
  assert_command dfx ledger account-id
  assert_eq "$BOB_ACCOUNT_ID"

  assert_command dfx ledger balance
  assert_eq "1000100.00000000 ICP"

  assert_command dfx ledger transfer --icp 100 --e8s 1 --memo 2 "$ALICE_ACCOUNT_ID"
  assert_contains "Transfer sent at block height"

  # The sender(bob) paid transaction fee which is 0.0001 ICP
  # 10100 - 100 - 0.0001 - 0.00000001 = 9999.99989999
  assert_command dfx ledger balance
  assert_eq "999999.99989999 ICP"

  # Transaction Deduplication
  t=$(current_time_nanoseconds)

  assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time "$t" "$ALICE_ACCOUNT_ID"
  # shellcheck disable=SC2154
  block_height=$(echo "$stdout" | sed '1q' | sed 's/Transfer sent at block height //')
  # shellcheck disable=SC2154
  assert_eq "Transfer sent at block height $block_height" "$stdout"

  assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time $((t+1)) "$ALICE_ACCOUNT_ID"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block height" "$stdout"
  # shellcheck disable=SC2154
  assert_not_contains "Transfer sent at block height $block_height" "$stdout"

  assert_command dfx ledger transfer --icp 1 --memo 1 --created-at-time "$t" "$ALICE_ACCOUNT_ID"
  # shellcheck disable=SC2154
  assert_eq "transaction is a duplicate of another transaction in block $block_height" "$stderr"
  assert_eq "Transfer sent at block height $block_height" "$stdout"

  assert_command dfx ledger transfer --icp 1 --memo 2 --created-at-time "$t" "$ALICE_ACCOUNT_ID"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block height" "$stdout"
  # shellcheck disable=SC2154
  assert_not_contains "Transfer sent at block height $block_height" "$stdout"
}

@test "ledger icrc functions" {
  dfx_start --system-canisters
  prepare_accounts

  ALICE=$(dfx identity get-principal --identity alice)
  BOB=$(dfx identity get-principal --identity bob)
  DAVID=$(dfx identity get-principal --identity david)

  dfx identity use alice

  assert_command dfx ledger balance
  assert_eq "1000000.00000000 ICP"

  # Test transfer and balance.

  assert_command dfx ledger transfer --amount 50 --to-principal "$DAVID" --memo 1 # to david
  assert_contains "Transfer sent at block index"

  # The owner(alice) transferred 50 ICP to david and paid transaction fee which is 0.0001 ICP.
  assert_command dfx ledger balance
  assert_eq "999949.99990000 ICP"

  # The receiver(david) received 50 ICP.
  assert_command dfx ledger balance --of-principal "$DAVID"
  assert_match "50.00000000 ICP"

  # Test approve, transfer-from and allowance.

  assert_command dfx ledger approve "$BOB" --amount 100 # to bob
  assert_contains "Approval sent at block index"

  # The approver(alice) paid approving fee which is 0.0001 ICP.
  assert_command dfx ledger balance
  assert_eq "999949.99980000 ICP"

  # The spender(bob) have 100 ICP allowance from the approver(alice).
  assert_command dfx ledger allowance --spender "$BOB"
  assert_match "Allowance 100.00000000 ICP"

  dfx identity use bob

  assert_command dfx ledger balance
  assert_match "1000000.00000000 ICP"

  assert_command dfx ledger transfer-from --from "$ALICE" --amount 50 "$DAVID" # to david
  assert_contains "Transfer sent at block index"

  # The spender(bob) transferred 50 ICP to david from the approver(alice).
  # And the approver(alice) paid transaction fee which is 0.0001 ICP
  assert_command dfx ledger balance --of-principal "$ALICE"
  assert_eq "999899.99970000 ICP"

  # The spender(bob) remains 49.99990000 ICP allowance from the approver(alice).
  assert_command dfx ledger allowance --owner "$ALICE" --spender "$BOB"
  assert_match "Allowance 49.99990000 ICP"

  # The spender(bob) balance is unchanged.
  assert_command dfx ledger balance --of-principal "$BOB"
  assert_match "1000000.00000000 ICP"

  # The receiver(david) received 50 ICP.
  assert_command dfx ledger balance --of-principal "$DAVID"
  assert_match "100.00000000 ICP"
}

@test "ledger subaccounts" {
  dfx_start --system-canisters
  prepare_accounts

  subacct=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
  assert_command dfx ledger account-id --identity bob --subaccount "$subacct"
  assert_match "$BOB_SUBACCOUNT_ID"

  dfx identity use alice
  assert_command dfx ledger balance
  assert_match "1000000.00000000 ICP"
  assert_command dfx ledger transfer --amount 100 --memo 1 "$BOB_SUBACCOUNT_ID"
  assert_match "Transfer sent at block height"
  assert_command dfx ledger balance
  assert_match "999899.99990000 ICP"

  dfx identity use bob
  assert_command dfx ledger balance
  assert_match "1000000.00000000 ICP"
  assert_command dfx ledger balance --subaccount "$subacct"
  assert_match "1000100.00000000 ICP"

  assert_command dfx ledger transfer --amount 100 --memo 2 "$ALICE_ACCOUNT_ID" --from-subaccount "$subacct"
  assert_match "Transfer sent at block height"
  assert_command dfx ledger balance
  assert_match "1000000.00000000 ICP"
  assert_command dfx ledger balance --subaccount "$subacct"
  assert_match "999999.99990000 ICP"
  assert_command dfx ledger balance --identity alice
  assert_match "999999.99990000 ICP"
}

tc_to_num() {
  if [[ $1 =~ T ]]; then
    echo "${1%%[^0-9]*}000000000000"
  else
    echo "${1%%[^0-9]*}"
  fi
}

@test "ledger top-up" {
  dfx_start --system-canisters
  prepare_accounts

  dfx identity use alice
  assert_command dfx ledger balance
  assert_match "1000000.00000000 ICP"

  wallet=$(dfx identity get-wallet)
  balance=$(tc_to_num "$(dfx wallet balance)")

  assert_command dfx ledger top-up "$wallet" --icp 5
  assert_match "Canister was topped up with 17600000000000 cycles"
  balance_now=$(tc_to_num "$(dfx wallet balance)")

  (( balance_now - balance > 15000000000000 ))

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
  assert_contains "transaction is a duplicate of another transaction in block $block_height" "$stderr"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block height $block_height" "$stdout"
  # shellcheck disable=SC2154
  assert_contains "Using transfer at block height $block_height" "$stdout"
  # shellcheck disable=SC2154
  assert_contains "Canister was topped up with" "$stdout"

  # Top up canister by name instead of principal
  dfx_new
  assert_command dfx canister create e2e_project_backend
  assert_command dfx ledger top-up e2e_project_backend --amount 5
  assert_contains "Canister was topped up with 17600000000000 cycles"
}

@test "ledger create-canister" {
  dfx_start --system-canisters
  prepare_accounts

  dfx identity use alice
  assert_command dfx ledger create-canister --amount=100 --subnet-type "type1" "$(dfx identity get-principal)"
  assert_match "Transfer sent at block height"
  assert_match "Refunded at block height"
  assert_match "with message: Provided subnet type type1 does not exist"

  SUBNET_ID="5kdm2-62fc6-fwnja-hutkz-ycsnm-4z33i-woh43-4cenu-ev7mi-gii6t-4ae" # a random, valid principal
  assert_command dfx ledger create-canister --amount=100 --subnet "$SUBNET_ID" "$(dfx identity get-principal)"
  assert_match "Transfer sent at block height"
  assert_match "Refunded at block height"
  assert_match "with message: Subnet $SUBNET_ID does not exist"

  # Verify that registry is queried before sending any ICP to CMC
  CANISTER_ID="2vxsx-fae" # anonymous principal
  balance=$(dfx ledger balance)
  assert_command_fail dfx ledger create-canister --amount=100 --next-to "$CANISTER_ID" "$(dfx identity get-principal)"
  # TODO: assert error message once registry is fixed
  assert_eq "$balance" "$(dfx ledger balance)"

  # Verify that creating a canister under a different principal's control properly sets ownership
  CONTROLLER_PRINCIPAL="$(dfx --identity default identity get-principal)"
  assert_command dfx ledger create-canister --amount=100 "$CONTROLLER_PRINCIPAL"
  echo "created with: $stdout"
  created_canister_id=$(echo "$stdout" | sed '3q;d' | sed 's/Canister created with id: //;s/"//g')
  assert_command dfx canister info "$created_canister_id"
  assert_contains "Controllers: $CONTROLLER_PRINCIPAL"
  assert_not_contains "$(dfx identity get-principal)"

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
  assert_contains "transaction is a duplicate of another transaction in block $block_height" "$stderr"
  # shellcheck disable=SC2154
  assert_contains "Transfer sent at block height $block_height" "$stdout"
  # shellcheck disable=SC2154
  assert_contains "Using transfer at block height $block_height" "$stdout"
  # shellcheck disable=SC2154
  assert_contains "Canister created with id: $created_canister_id" "$stdout"

}

@test "ledger show-subnet-types" {
  dfx_start --system-canisters
  assert_command dfx ledger show-subnet-types
  assert_eq '["fiduciary"]'
}

@test "balance without ledger fails as expected" {
  dfx_start

  assert_command_fail dfx ledger balance
  assert_contains "ICP Ledger with canister ID 'ryjl3-tyaaa-aaaaa-aaaba-cai' is not installed."
}
