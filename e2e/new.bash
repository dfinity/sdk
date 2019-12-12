#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1
}

@test "dfx new - good names" {
    dfx new --skip-frontend a_good_name_
    dfx new --skip-frontend A
    dfx new --skip-frontend b
    dfx new --skip-frontend a_
    dfx new --skip-frontend a_1
    dfx new --skip-frontend a1
    dfx new --skip-frontend a1a
}

@test "dfx new - bad names" {
    assert_command_fail dfx new _a_good_name_
    assert_command_fail dfx new __also_good
    assert_command_fail dfx new _1
    assert_command_fail dfx new _a
    assert_command_fail dfx new 1
    assert_command_fail dfx new 1_
    assert_command_fail dfx new -
    assert_command_fail dfx new _
    assert_command_fail dfx new a-b-c
    assert_command_fail dfx new '🕹'
    assert_command_fail dfx new '不好'
    assert_command_fail dfx new 'a:b'
}
