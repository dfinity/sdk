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

    assert_command dfx canister call --query e2e_project_assets retrieve '("binary/noise.txt")'
    assert_eq '(vec { 184; 1; 32; 128; 10; 119; 49; 50; 32; 0; 120; 121; 10; 75; 76; 11; 10; 106; 107; })'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")'
    assert_eq '(vec { 99; 104; 101; 114; 114; 105; 101; 115; 10; 105; 116; 39; 115; 32; 99; 104; 101; 114; 114; 121; 32; 115; 101; 97; 115; 111; 110; 10; 67; 72; 69; 82; 82; 73; 69; 83; })'

    assert_command dfx canister call --update e2e_project_assets store '("AA", vec { 100; 107; 62; 9; })'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_assets store '("B", vec { 88; 87; 86; })'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")'
    assert_eq '(vec { 88; 87; 86; })'

    assert_command dfx canister call --query e2e_project_assets retrieve '("AA")'
    assert_eq '(vec { 100; 107; 62; 9; })'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")'
    assert_eq '(vec { 88; 87; 86; })'

    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("C")'

    assert_command dfx canister call --update e2e_project_assets store '("AA", vec { 100; 107; 62; 9; })'

    HOME=. assert_command_fail dfx canister call --update e2e_project_assets store '("index.js", vec { 1; 2; 3; })'
}
