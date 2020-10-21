#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "dfx restarts the replica" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    DFX_PID=$(cat .dfx/pid)

    echo "export"
    export

    echo "jobs 0"
    jobs -p

    REPLICA_PID=$(ps x | grep [/[:space:]]replica | awk '{print $1}')

    echo "replica pid is $REPLICA_PID"

    kill -KILL $REPLICA_PID
    assert_process_exits $REPLICA_PID 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}
