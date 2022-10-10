#!/usr/bin/env bats

load ../utils/_

# All tests in this file are skipped for ic-ref.  See scripts/workflows/e2e-matrix.py


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

    dfx_patchelf
    dfx replica --port 0 &
    DFX_PID=$!

    # wait for replica to start
    assert_file_eventually_exists "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port" 15s

    kill -"$signal" "$DFX_PID"

    assert_process_exits $DFX_PID 15s
    assert_no_dfx_start_or_replica_processes
}
