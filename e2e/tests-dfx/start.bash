#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "candid interface available through dfx start" {
    dfx_new hello
    dfx_start

    assert_command dfx deploy

    ID=$(dfx canister id hello_backend)
    PORT=$(get_webserver_port)
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID" -o ./web.txt --max-time 60
    assert_command diff .dfx/local/canisters/hello_backend/hello_backend.did ./web.txt
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID"\&format=js -o ./web.txt --max-time 60
    # Relax diff as it's produced by two different compilers.
    assert_command diff --ignore-all-space --ignore-blank-lines .dfx/local/canisters/hello_backend/hello_backend.did.js ./web.txt
}

@test "dfx restarts the replica" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)

    echo "replica pid is $REPLICA_PID"

    kill -KILL "$REPLICA_PID"
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

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx restarts icx-proxy" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    ICX_PROXY_PID=$(get_icx_proxy_pid)

    echo "icx-proxy pid is $ICX_PROXY_PID"

    kill -KILL "$ICX_PROXY_PID"
    assert_process_exits "$ICX_PROXY_PID" 15s
}

@test "dfx restarts icx-proxy when the replica restarts" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    ICX_PROXY_PID=$(get_icx_proxy_pid)

    echo "replica pid is $REPLICA_PID"
    echo "icx-proxy pid is $ICX_PROXY_PID"

    kill -KILL "$REPLICA_PID"
    assert_process_exits "$REPLICA_PID" 15s
    assert_process_exits "$ICX_PROXY_PID" 15s

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

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx starts replica with subnet_type application" {
    install_asset subnet_type/application

    assert_command dfx start --background
    assert_match "subnet_type: Application"

}

@test "dfx starts replica with subnet_type verifiedapplication" {
    install_asset subnet_type/verified_application

    assert_command dfx start --background
    assert_match "subnet_type: VerifiedApplication"

}

@test "dfx starts replica with subnet_type system" {
    install_asset subnet_type/system

    assert_command dfx start --background
    assert_match "subnet_type: System"

}

@test "dfx start detects if dfx is already running" {
    dfx_new hello
    dfx_start

    assert_command_fail dfx start
    assert_match "dfx is already running"
}
