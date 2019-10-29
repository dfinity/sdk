#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1
    dfx_new
}

@test "build -- stdlib_usage_as" {
    install_asset stdlib_usage_as
    assert_command dfx build
}
