#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
}

@test "install fails if no argument is provided" {
    skip "This is known to fail by leaving replica and dfx start processes running"
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx canister install
    assert_match "required arguments were not provided"
    assert_match "--all"

    # Verify that processes are not running (yet).
    ! ( ps | grep -e replica -e "[d]fx start" )
}
