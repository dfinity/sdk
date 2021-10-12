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

@test "dfx replica kills the replica upon SIGINT" {
  dfx_replica_kills_replica SIGINT
}

@test "dfx replica kills the replica upon SIGTERM" {
  dfx_replica_kills_replica SIGTERM
}

dfx_replica_kills_replica() {
    signal=$1

    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_patchelf
    dfx replica --port 0 &
    DFX_PID=$!

    # wait for replica to start
    assert_file_eventually_exists .dfx/replica-configuration/replica-1.port 15s

    kill -"$signal" "$DFX_PID"

    assert_process_exits $DFX_PID 15s
    assert_no_dfx_start_or_replica_processes
}
