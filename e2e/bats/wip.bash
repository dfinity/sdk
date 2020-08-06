#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    # Each test gets its own home directory in order to have its own identities.
    mkdir $(pwd)/home-for-test
    export HOME=$(pwd)/home-for-test

    dfx_new
}

teardown() {
    dfx_stop
    rm -rf $(pwd)/home-for-test
}


