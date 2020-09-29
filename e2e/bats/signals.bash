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

@test "dfx shuts down (gracefully) due to SIGINT" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_start

    DFX_PID=$(cat .dfx/pid)

    kill -SIGINT $DFX_PID

    assert_process_exits $DFX_PID 15s

    assert_no_dfx_start_or_replica_processes
}

@test "dfx shuts down (gracefully) due to SIGTERM" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_start

    DFX_PID=$(cat .dfx/pid)

    kill -SIGTERM $DFX_PID

    assert_process_exits $DFX_PID 15s

    assert_no_dfx_start_or_replica_processes
}
