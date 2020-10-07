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

    echo "xx 1"
    ps | grep replica
    echo "xx 2"
    ps | grep "[/\s]replica"
    echo "xx 3"
    ps | grep "[/\s]replica" | cut -d ' ' -f 1
    echo "xx 4"

    # find the replica that is the child of dfx.  we do not have awk.
    REPLICA_PID=$(ps | grep "[/\s]replica" | cut -d ' ' -f 1)

    echo "replica pid is $REPLICA_PID"

    kill -KILL $REPLICA_PID
    assert_process_exits $REPLICA_PID 15s

    timeout $timeout sh -c \
      'until ps | grep "[/\s]replica"; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)

    echo
    ps | grep "[/\s]replica"
    echo

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'

    #assert_match "what"
}

