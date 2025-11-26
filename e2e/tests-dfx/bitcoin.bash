#!/usr/bin/env bats

load ../utils/_

BITCOIN_CANISTER_ID="g4xu7-jiaaa-aaaan-aaaaq-cai"

setup() {
  standard_setup

  # Create a unique bitcoin data directory for this test
  export BITCOIN_DATADIR="$E2E_TEMP_DIR/bitcoin-datadir"
  mkdir -p "$BITCOIN_DATADIR"

  # Kill any stray bitcoind processes that might be running
  pkill -9 bitcoind || true
  
  # Clean up any stale lock files
  rm -f "$BITCOIN_DATADIR/.lock" || true
  
  # Wait a moment for processes to fully terminate
  sleep 1
  
  # Start bitcoind with explicit datadir and timeout
  bitcoind -regtest -datadir="$BITCOIN_DATADIR" -daemonwait -timeout=30
}

teardown() {
  # Stop bitcoin with the correct datadir
  bitcoin-cli -regtest -datadir="$BITCOIN_DATADIR" stop 2>/dev/null || true
  
  # Give it time to shut down gracefully
  sleep 2
  
  # Force kill any remaining bitcoind processes
  pkill -9 bitcoind || true

  dfx_stop
  standard_teardown
}

set_project_default_bitcoin_enabled() {
  jq '.defaults.bitcoin.enabled=true' dfx.json | sponge dfx.json
}

set_shared_local_network_bitcoin_enabled() {
  create_networks_json
  jq '.local.bitcoin.enabled=true' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

set_local_network_bitcoin_enabled() {
  jq '.networks.local.bitcoin.enabled=true' dfx.json | sponge dfx.json
}

@test "noop" {
  assert_command bitcoin-cli -regtest -datadir="$BITCOIN_DATADIR" createwallet "test"
  ADDRESS="$(bitcoin-cli -regtest -datadir="$BITCOIN_DATADIR" getnewaddress)"
  assert_command bitcoin-cli -regtest -datadir="$BITCOIN_DATADIR" generatetoaddress 101 "$ADDRESS"
}

@test "dfx start --enable-bitcoin --background waits until bitcoin canister is installed" {
  dfx_new hello

  dfx_start --enable-bitcoin

  assert_command dfx canister info "$BITCOIN_CANISTER_ID"
  assert_contains "Module hash: 0x"
}

@test "can call bitcoin API of the management canister" {
  install_asset bitcoin
  dfx_start --enable-bitcoin
  dfx identity get-wallet

  # the non-query Bitcoin API can only be called by a canister not an agent
  # we need to proxy the call through the wallet canister

  # bitcoin_get_balance
  assert_command dfx canister call --with-cycles 100000000 --wallet default aaaaa-aa --candid bitcoin.did bitcoin_get_balance '(
  record {
    network = variant { regtest };
    address = "bcrt1qu58aj62urda83c00eylc6w34yl2s6e5rkzqet7";
    min_confirmations = opt (1 : nat32);
  }
)'
  assert_eq "(0 : nat64)"

  # bitcoin_get_utxos
  assert_command dfx canister call --with-cycles 10000000000 --wallet default aaaaa-aa --candid bitcoin.did bitcoin_get_utxos '(
  record {
    network = variant { regtest };
    filter = opt variant { min_confirmations = 1 : nat32 };
    address = "bcrt1qu58aj62urda83c00eylc6w34yl2s6e5rkzqet7";
  }
)'
  assert_contains "tip_height = 0 : nat32;"

  # bitcoin_get_current_fee_percentiles
  assert_command dfx canister call --with-cycles 100000000 --wallet default aaaaa-aa --candid bitcoin.did bitcoin_get_current_fee_percentiles '(record { network = variant { regtest } })'

  # bitcoin_send_transaction
  # It's hard to test this without a real transaction, but we can at least check that the call fails.
  # The error message indicates that the argument is in correct format, only the inner transaction is malformed.
  assert_command_fail dfx canister call --with-cycles 5020000000 --wallet default aaaaa-aa --candid bitcoin.did bitcoin_send_transaction '(record { transaction = vec {0:nat8}; network = variant { regtest } })'
  assert_contains "send_transaction failed: MalformedTransaction"
}
