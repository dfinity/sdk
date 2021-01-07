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
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets retrieve '("binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01 \80\0aw12 \00xy\0aKL\0b\0ajk")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")' --output idl
    assert_eq '(blob "\cherries\0ait'\''s cherry season\0aCHERRIES")'

    assert_command dfx canister call --update e2e_project_assets store '("AA", blob "hello, world!")'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_assets store '("B", vec { 88; 87; 86; })'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("AA")' --output idl
    assert_eq '(blob "hello, world!")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("C")'

    HOME=. assert_command_fail dfx canister call --update e2e_project_assets store '("index.js", vec { 1; 2; 3; })'
}
