#!/usr/bin/env bats

load ../utils/_

# All tests in this file are skipped for ic-ref.  See scripts/workflows/e2e-matrix.py

setup() {
    standard_setup

    bitcoind -regtest -daemonwait
}

teardown() {
    bitcoin-cli -regtest stop

    dfx_stop
    stop_dfx_replica
    stop_dfx_bootstrap
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
    assert_command bitcoin-cli -regtest createwallet "test"
    ADDRESS="$(bitcoin-cli -regtest getnewaddress)"
    assert_command bitcoin-cli -regtest generatetoaddress 101 "$ADDRESS"
}

@test "dfx restarts replica when ic-btc-adapter restarts" {
    dfx_new hello
    dfx_start --enable-bitcoin

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    BTC_ADAPTER_PID=$(get_btc_adapter_pid)

    echo "replica pid is $REPLICA_PID"
    echo "ic-btc-adapter pid is $BTC_ADAPTER_PID"

    kill -KILL "$BTC_ADAPTER_PID"
    assert_process_exits "$BTC_ADAPTER_PID" 15s
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait 1\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting
    # even after the above, still sometimes fails with
    #     IC0515: Certified state is not available yet. Please try again...
    sleep 10
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait 2\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'

    ID=$(dfx canister id hello_frontend)

    timeout 15s sh -c \
      "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/webserver-port\")/sample-asset.txt?canisterId=$ID; do echo waiting for icx-proxy to restart; sleep 1; done" \
      || (echo "icx-proxy did not restart" && ps aux && exit 1)

    assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx restarts replica when ic-btc-adapter restarts - replica and bootstrap" {
    dfx_new hello
    dfx_replica --enable-bitcoin
    dfx_bootstrap

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    BTC_ADAPTER_PID=$(get_btc_adapter_pid)

    echo "replica pid is $REPLICA_PID"
    echo "replica port is $(get_replica_port)"
    echo "ic-btc-adapter pid is $BTC_ADAPTER_PID"

    echo "killing btc adapter"

    kill -KILL "$BTC_ADAPTER_PID"
    assert_process_exits "$BTC_ADAPTER_PID" 15s
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -x -c \
      "until curl --fail --verbose -o /dev/null http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port\")/api/v2/status; do echo \"waiting for replica to restart on port \$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port\")\"; sleep 1; done" \
      || (echo "replica did not restart" && echo "last replica port was $(get_replica_port)" && ps aux && exit 1)

    # bootstrap doesn't detect the new replica port, so we have to restart it
    stop_dfx_bootstrap
    dfx_bootstrap

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait 1\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting
    # even after the above, still sometimes fails with
    #     IC0515: Certified state is not available yet. Please try again...
    sleep 10
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait 2\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx start --bitcoin-node <node> implies --enable-bitcoin" {
    dfx_new hello
    dfx_start "--bitcoin-node" "127.0.0.1:18444"

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

@test "dfx replica --bitcoin-node <node> implies --enable-bitcoin" {
    dfx_new hello
    dfx_replica "--bitcoin-node" "127.0.0.1:18444"
    dfx_bootstrap

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}


@test "dfx start --enable-bitcoin with no other configuration succeeds" {
    dfx_new hello

    dfx_start --enable-bitcoin

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

@test "dfx replica --enable-bitcoin with no other configuration succeeds" {
    dfx_new hello

    dfx_replica --enable-bitcoin

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

@test "can enable bitcoin through default configuration - dfx start" {
    dfx_new hello
    define_project_network
    set_project_default_bitcoin_enabled

    dfx_start

    assert_file_not_empty .dfx/network/local/ic-btc-adapter-pid
}

@test "can enable bitcoin through shared local network - dfx start" {
    dfx_new hello
    set_shared_local_network_bitcoin_enabled

    dfx_start

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

@test "can enable bitcoin through local network configuration - dfx start" {
    dfx_new hello
    set_local_network_bitcoin_enabled

    dfx_start

    assert_file_not_empty .dfx/network/local/ic-btc-adapter-pid
}

@test "can enable bitcoin through default configuration - dfx replica" {
    dfx_new hello
    define_project_network
    set_project_default_bitcoin_enabled

    dfx_replica

    assert_file_not_empty .dfx/network/local/ic-btc-adapter-pid
}

@test "can enable bitcoin through shared local network - dfx replica" {
    dfx_new hello
    set_shared_local_network_bitcoin_enabled

    dfx_replica

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

@test "dfx start with both bitcoin and canister http enabled" {
    dfx_new hello

    dfx_start --enable-bitcoin --enable-canister-http

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-canister-http-adapter-pid"

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'
}

@test "dfx replica+bootstrap with both bitcoin and canister http enabled" {
    dfx_new hello

    dfx_replica --enable-bitcoin --enable-canister-http
    dfx_bootstrap

    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
    assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-canister-http-adapter-pid"

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'
}

