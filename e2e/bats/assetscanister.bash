#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

teardown() {
    dfx_stop
}

@test "can store and retrieve assets by key" {
    install_asset assetscanister

    dfx_start
    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets retrieve '("binary/noise.txt")'
    assert_eq '("'$(base64 src/e2e_project_assets/assets/binary/noise.txt)'")'
    assert_eq '("uAEggAp3MTIgAHh5CktMCwpqaw==")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")'
    assert_eq '("'$(base64 src/e2e_project_assets/assets/text-with-newlines.txt)'")'
    assert_eq '("Y2hlcnJpZXMKaXQncyBjaGVycnkgc2Vhc29uCkNIRVJSSUVT")'

    assert_command dfx canister call --update e2e_project_assets store '("AA", "xxx")'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_assets store '("B", "yyyy")'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")'
    assert_eq '("yyyy")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("AA")'
    assert_eq '("xxx")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")'
    assert_eq '("yyyy")'

    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("C")'
}

