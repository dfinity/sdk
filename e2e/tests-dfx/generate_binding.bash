#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
}

@test "generate-binding succeeds on a simple remote example" {
    install_asset generate_binding/basic

    assert_command dfx generate-binding --all

    if [[ ! -f remote.mo ]]; then
        echo "remote.mo not created"
        exit 1
    fi
}

@test "generate-binding --overwrite behaves as expected" {
    install_asset generate_binding/basic

    echo "wrong" > remote.mo
    assert_command dfx generate-binding --all
    assert_match "already exists"

    assert_command dfx generate-binding --all --overwrite
    assert_neq "wrong" "$(cat remote.mo)"

}