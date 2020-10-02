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

@test "dfx replica kills the replica upon SIGINT" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx replica --port 0 &
    DFX_PID=$!

    echo "dfx pid is $DFX_PID"
    ps

    #assert_file_eventually_exists .dfx/pid 15s
    assert_file_eventually_exists .dfx/config/port.txt 15s

    ps

    kill -SIGINT $DFX_PID

    assert_process_exits $DFX_PID 15s

    assert_no_dfx_start_or_replica_processes
}

@test "dfx replica kills the replica upon SIGTERM" {
  skip "not yet"
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx replica --port 0 &

    DFX_PID=$(cat .dfx/pid)

    kill -SIGTERM $DFX_PID

    assert_process_exits $DFX_PID 15s

    assert_no_dfx_start_or_replica_processes
}
