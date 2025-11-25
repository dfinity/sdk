#!/usr/bin/env bats

load ../utils/_

DOGECOIN_CANISTER_ID="gordg-fyaaa-aaaan-aaadq-cai"

setup() {
  standard_setup

  # Create a unique dogecoin data directory for this test
  export DOGECOIN_DATADIR="$E2E_TEMP_DIR/dogecoin-datadir"
  mkdir -p "$DOGECOIN_DATADIR"

  # Kill any stray dogecoind processes that might be running
  pkill -9 dogecoind || true
  
  # Clean up any stale lock files
  rm -f "$DOGECOIN_DATADIR/.lock" || true
  
  # Wait a moment for processes to fully terminate
  sleep 1

  # Configured per
  # https://github.com/dfinity/dogecoin-canister/blob/3bb69b2dab53e07cf6a3d867a08b76a2d0cfe6b0/docs/src/environment.md?plain=1#L41
  cat > "$DOGECOIN_DATADIR/dogecoin.conf" <<EOF
regtest=1
txindex=1
rpcuser=ic-doge-integration
rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E=
rpcauth=ic-doge-integration:cdf2741387f3a12438f69092f0fdad8e\$62081498c98bee09a0dce2b30671123fa561932992ce377585e8e08bb0c11dfa
EOF
  
  # Start dogecoind with explicit datadir and timeout
  dogecoind -datadir="$DOGECOIN_DATADIR" -daemon -timeout=30 --port=18444
}

teardown() {
  # Stop dogecoin with the correct datadir
  dogecoin-cli -datadir="$DOGECOIN_DATADIR" stop 2>/dev/null || true
  
  # Give it time to shut down gracefully
  sleep 2
  
  # Force kill any remaining dogecoind processes
  pkill -9 dogecoind || true

  dfx_stop
  standard_teardown
}

set_project_default_dogecoin_enabled() {
  jq '.defaults.dogecoin.enabled=true' dfx.json | sponge dfx.json
}

set_shared_local_network_dogecoin_enabled() {
  create_networks_json
  jq '.local.dogecoin.enabled=true' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

set_local_network_dogecoin_enabled() {
  jq '.networks.local.dogecoin.enabled=true' dfx.json | sponge dfx.json
}

@test "noop" {
  assert_command dogecoin-cli -datadir="$DOGECOIN_DATADIR" getinfo
}

@test "dfx start --enable-dogecoin --background waits until dogecoin canister is installed" {
  dfx_new hello

  dfx_start --enable-dogecoin

  assert_command dfx canister info "$DOGECOIN_CANISTER_ID"
  assert_contains "Module hash: 0x"
}

@test "can call dogecoin API of the management canister" {
  install_asset dogecoin
  dfx_start --enable-dogecoin
  dfx identity get-wallet 

  # the non-query dogecoin API can only be called by a canister not an agent
  # we need to proxy the call through the wallet canister

  # Make a call to the dogecoin canister and check that it succeeds
  assert_command dfx canister call --with-cycles 5000000000 --wallet default $DOGECOIN_CANISTER_ID dogecoin_get_block_headers '(
  record {
    start_height = 0 : nat32;
    end_height = null;
    network = variant { regtest };
  },
)'
}
