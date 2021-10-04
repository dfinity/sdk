#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx restarts the replica 20" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'
    dfx canister status hello || (echo "dfx canister status failed...continuing" ; dfx ping)

    REPLICA_PID=$(cat .dfx/replica-configuration/replica-pid)

    echo "replica pid is $REPLICA_PID"

    kill -KILL "$REPLICA_PID"
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    dfx canister status hello || (echo "dfx canister status failed...continuing" ; dfx ping)

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx restarts icx-proxy when the replica restarts 20" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'
    dfx canister status hello || (echo "dfx canister status failed...continuing" ; dfx ping)

    REPLICA_PID=$(cat .dfx/replica-configuration/replica-pid)
    ICX_PROXY_PID=$(cat .dfx/icx-proxy-pid)

    echo "replica pid is $REPLICA_PID"
    echo "icx-proxy pid is $ICX_PROXY_PID"

    kill -KILL "$REPLICA_PID"
    assert_process_exits "$REPLICA_PID" 15s
    assert_process_exits "$ICX_PROXY_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    dfx canister status hello || (echo "dfx canister status failed...continuing" ; dfx ping)

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'

    ID=$(dfx canister id hello_assets)

    timeout 15s sh -c \
      "until curl --fail http://localhost:$(cat .dfx/webserver-port)/sample-asset.txt?canisterId=$ID; do echo waiting for icx-proxy to restart; sleep 1; done" \
      || (echo "icx-proxy did not restart" && ps aux && exit 1)

    assert_command curl --fail http://localhost:"$(cat .dfx/webserver-port)"/sample-asset.txt?canisterId="$ID"
}
