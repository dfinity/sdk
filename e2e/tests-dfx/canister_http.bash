#!/usr/bin/env bats

load ../utils/_

# All tests in this file are skipped for ic-ref.  See scripts/workflows/e2e-matrix.py

setup() {
    standard_setup
}

teardown() {
    dfx_stop
    stop_dfx_replica
    stop_dfx_bootstrap
    standard_teardown
}

set_default_canister_http_enabled() {
    # shellcheck disable=SC2094
    cat <<<"$(jq '.defaults.canister_http.enabled=true' dfx.json)" >dfx.json
}

set_local_network_canister_http_enabled() {
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.local.canister_http.enabled=true' dfx.json)" >dfx.json
}

@test "dfx restarts replica when ic-canister-http-adapter restarts" {
    dfx_new hello
    set_default_canister_http_enabled
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    CANISTER_HTTP_ADAPTER_PID=$(get_canister_http_adapter_pid)

    echo "replica pid is $REPLICA_PID"
    echo "ic-canister-http-adapter pid is $CANISTER_HTTP_ADAPTER_PID"

    kill -KILL "$CANISTER_HTTP_ADAPTER_PID"
    assert_process_exits "$CANISTER_HTTP_ADAPTER_PID" 15s
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    # Even so, after that passes, sometimes this happens:
    #     IC0515: Certified state is not available yet. Please try again...

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx restarts replica when ic-canister-http-adapter restarts - replica and bootstrap" {
    dfx_new hello
    set_default_canister_http_enabled
    dfx_replica
    dfx_bootstrap

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    CANISTER_HTTP_ADAPTER_PID=$(get_canister_http_adapter_pid)

    echo "replica pid is $REPLICA_PID"
    echo "replica port is $(get_replica_port)"
    echo "ic-canister-http-adapter pid is $CANISTER_HTTP_ADAPTER_PID"

    kill -KILL "$CANISTER_HTTP_ADAPTER_PID"
    assert_process_exits "$CANISTER_HTTP_ADAPTER_PID" 15s
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -x -c \
      "until curl --fail --verbose -o /dev/null http://localhost:\$(cat .dfx/replica-configuration/replica-1.port)/api/v2/status; do echo \"waiting for replica to restart on port \$(cat .dfx/replica-configuration/replica-1.port)\"; sleep 1; done" \
      || (echo "replica did not restart" && echo "last replica port was $(get_replica_port)" && ps aux && exit 1)

    # bootstrap doesn't detect the new replica port, so we have to restart it
    stop_dfx_bootstrap
    dfx_bootstrap

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx start --enable-canister-http with no other configuration succeeds" {
    dfx_new hello

    dfx_start --enable-canister-http

    assert_file_not_empty .dfx/ic-canister-http-adapter-pid
}

@test "dfx replica --enable-canister-http with no other configuration succeeds" {
    dfx_new hello

    dfx_replica --enable-canister-http

    assert_file_not_empty .dfx/ic-canister-http-adapter-pid
}

@test "can enable http through default configuration (dfx start)" {
    dfx_new hello
    set_default_canister_http_enabled

    dfx_start

    assert_file_not_empty .dfx/ic-canister-http-adapter-pid
}

@test "can enable http through local network configuration (dfx start)" {
    dfx_new hello
    set_local_network_canister_http_enabled

    dfx_start

    assert_file_not_empty .dfx/ic-canister-http-adapter-pid
}

@test "can enable http through default configuration (dfx replica)" {
    dfx_new hello
    set_default_canister_http_enabled

    dfx_replica

    assert_file_not_empty .dfx/ic-canister-http-adapter-pid
}
