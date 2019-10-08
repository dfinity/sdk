#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    # Kill the node manager, the dfx and the client. Ignore errors (ie. if processes aren't
    # running).
    killall dfx nodemanager client || true
}

@test "build fails on invalid actorscript" {
    install_asset invalid_as
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build succeeds on default project" {
    assert_command dfx build
    assert_match "Building hello..."
}
