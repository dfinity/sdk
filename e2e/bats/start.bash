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

    # ps "isn't a robust solution": https://dfinity.atlassian.net/browse/OPS-166
    #
    # I guess we could make dfx write the replica pid somewhere.
    #
    # Anyway, if this test ends up sometimes killing a replica other than
    # the one created by dfx for this test, then some other test might fail.
    #
    # Also, this does not work on linux:
    #   ps -o "ppid, pid, comm"
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
