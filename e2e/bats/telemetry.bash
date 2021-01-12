#!/usr/bin/env bats

load utils/_

setup() {
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
}

teardown() {
    :
}

@test 'dfx renders telemetry consent message' {
    export DFX_CONFIG_ROOT=$(pwd)
    test ! -f .config/dfx/telemetry/witness.blank
    assert_command dfx identity whoami
    test -f .config/dfx/telemetry/witness.blank
    assert_match 'SDK sends anonymous usage data'
    assert_command dfx identity whoami
    assert_eq 'default'
}
