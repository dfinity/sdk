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
    dfx_stop_replica_and_bootstrap
    standard_teardown

    # created in bitcoin/patch.bash
    rm -f "/tmp/e2e-ic-btc-adapter.$$.socket"
}

set_default_btc_adapter_config() {
    # shellcheck disable=SC2094
    cat <<<"$(jq '.defaults.bitcoin.btc_adapter_config="'"$(pwd)/testnet.config.json"'"' dfx.json)" >dfx.json
}
set_default_bitcoin_enabled() {
    # shellcheck disable=SC2094
    cat <<<"$(jq '.defaults.bitcoin.enabled=true' dfx.json)" >dfx.json
}

@test "noop" {
    assert_command bitcoin-cli -regtest createwallet "test"
    ADDRESS="$(bitcoin-cli -regtest getnewaddress)"
    assert_command bitcoin-cli -regtest generatetoaddress 101 "$ADDRESS"
}

@test "dfx restarts replica when ic-btc-adapter restarts" {
    dfx_new hello
    install_asset bitcoin
    set_default_btc_adapter_config
    set_default_bitcoin_enabled
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(cat .dfx/replica-configuration/replica-pid)
    BTC_ADAPTER_PID=$(cat .dfx/ic-btc-adapter-pid)

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
      "until dfx canister call hello greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'

    ID=$(dfx canister id hello_assets)

    timeout 15s sh -c \
      "until curl --fail http://localhost:\$(cat .dfx/webserver-port)/sample-asset.txt?canisterId=$ID; do echo waiting for icx-proxy to restart; sleep 1; done" \
      || (echo "icx-proxy did not restart" && ps aux && exit 1)

    assert_command curl --fail http://localhost:"$(cat .dfx/webserver-port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx restarts replica when ic-btc-adapter restarts (replica and bootstrap)" {
    dfx_new hello
    install_asset bitcoin
    set_default_btc_adapter_config
    set_default_bitcoin_enabled
    dfx_start_replica_and_bootstrap

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(cat .dfx/replica-configuration/replica-pid)
    BTC_ADAPTER_PID=$(cat .dfx/ic-btc-adapter-pid)

    echo "replica pid is $REPLICA_PID"
    echo "replica port is $(cat .dfx/replica-configuration/replica-1.port)"
    echo "ic-btc-adapter pid is $BTC_ADAPTER_PID"

    kill -KILL "$BTC_ADAPTER_PID"
    assert_process_exits "$BTC_ADAPTER_PID" 15s
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -x -c \
      "until curl --fail --verbose -o /dev/null http://localhost:\$(cat .dfx/replica-configuration/replica-1.port)/api/v2/status; do echo \"waiting for replica to restart on port \$(cat .dfx/replica-configuration/replica-1.port)\"; sleep 1; done" \
      || (echo "replica did not restart" && echo "last replica port was $(cat .dfx/replica-configuration/replica-1.port)" && ps aux && exit 1)
    # shellcheck disable=SC2094
    cat <<<"$(jq .networks.local.bind=\"127.0.0.1:"$(cat .dfx/replica-configuration/replica-1.port)"\" dfx.json)" >dfx.json
    echo "dfx.json configured for new replica:"
    cat dfx.json

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}


@test "dfx start --btc-adapter-config implies --enable-bitcoin" {
    dfx_new hello
    install_asset bitcoin
    dfx_start "--btc-adapter-config" "$(pwd)/testnet.config.json"

    assert_file_not_empty .dfx/ic-btc-adapter-pid
}

@test "dfx replica --btc-adapter-config implies --enable-bitcoin" {
    dfx_new hello
    install_asset bitcoin
    dfx_start_replica_and_bootstrap "--btc-adapter-config" "$(pwd)/testnet.config.json"

    assert_file_not_empty .dfx/ic-btc-adapter-pid
}


@test "dfx start/replica --enable-bitcoin without a btc adapter config fails" {
    dfx_new hello
    install_asset bitcoin

    assert_command_fail dfx start --enable-bitcoin
    assert_eq "Error: Failed to get BTC adapter config. Error: Bitcoin integration was enabled without either --btc-adapter-config or .defaults.bitcoin.btc_adapter_config in dfx.json"

    assert_command_fail dfx replica --enable-bitcoin
    assert_eq "Error: Failed to get BTC adapter config. Error: Bitcoin integration was enabled without either --btc-adapter-config or .defaults.bitcoin.btc_adapter_config in dfx.json"
}

@test "can enable bitcoin through default configuration (dfx start)" {
    dfx_new hello
    install_asset bitcoin
    set_default_btc_adapter_config
    set_default_bitcoin_enabled

    dfx_start

    assert_file_not_empty .dfx/ic-btc-adapter-pid
}

@test "can enable bitcoin through default configuration (dfx replica)" {
    dfx_new hello
    install_asset bitcoin
    set_default_btc_adapter_config
    set_default_bitcoin_enabled

    dfx_start_replica_and_bootstrap

    assert_file_not_empty .dfx/ic-btc-adapter-pid
}
