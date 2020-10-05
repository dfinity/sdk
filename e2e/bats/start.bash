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
    skip "this test is not ready, nor is dfx start ready for it"

    dfx_start

    DFX_PID=$(cat .dfx/pid)
    echo "dfx is process $DFX_PID"

    # find the replica that is the child of dfx.  we do not have awk.
    REPLICA_PID=$(ps -o "ppid, pid, comm" | grep ^$DFX_PID.*replica$ | cut -d ' ' -f 2)
    echo "replica is process $REPLICA_PID"

    # not nice
    kill -KILL $REPLICA_PID

    assert_process_exits $REPLICA_PID 15s

    timeout $timeout sh -c \
      "while ps -o "ppid, pid, comm" | grep ^$DFX_PID.*replica$; do echo waiting for replica to restart; sleep 1; done" \
      || (echo "replica did not restart" && ps aux && exit 1)

    ps aux | grep -e dfx -e replica

    echo "forcing failure"
    exit 2

}

