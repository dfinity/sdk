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
    #skip "this test is not ready, nor is dfx start ready for it"

    dfx_start

    install_asset greet
    assert_command dfx deploy

    assert_command dfx canister call hello greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    DFX_PID=$(cat .dfx/pid)
    echo "dfx is process $DFX_PID"

    ps -o "ppid, pid, comm"
    # find the replica that is the child of dfx.  we do not have awk.
    REPLICA_PID=$(ps -o "ppid, pid, comm" | grep ^\\s*$DFX_PID\\s.*replica$ | cut -d ' ' -f 2)
    echo "replica is process $REPLICA_PID"

    ps -o "ppid, pid, comm" | grep -e replica -e dfx

    # not nice
    kill -KILL $REPLICA_PID
    echo "sent kill signal to replica $REPLICA_PID"

    assert_process_exits $REPLICA_PID 15s

    echo "replica exited, waiting for replica to restart"

    timeout $timeout sh -c \
      'until ps -o "ppid, pid, comm" | grep ^\\s*'$DFX_PID'\\s.*replica$; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)

    echo "no longer waiting for replica to restart"
    ps -o "ppid, pid, comm"
    echo
    ps aux | grep -e dfx -e replica

    assert_command dfx canister call hello greet '("Omega")'
    assert_eq '("Hello, Omega!")'

}

