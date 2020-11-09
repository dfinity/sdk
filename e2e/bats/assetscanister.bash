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
    assert_eq '(blob "\b8\01\20\80\0a\77\31\32\20\00\78\79\0a\4b\4c\0b\0a\6a\6b")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")' --output idl
    assert_eq '(blob "\63\68\65\72\72\69\65\73\0a\69\74\27\73\20\63\68\65\72\72\79\20\73\65\61\73\6f\6e\0a\43\48\45\52\52\49\45\53")'

    assert_command dfx canister call --update e2e_project_assets store '("AA", vec { 100; 107; 62; 9; })'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_assets store '("B", vec { 88; 87; 86; })'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "\58\57\56")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("AA")' --output idl
    assert_eq '(blob "\64\6b\3e\09")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "\58\57\56")'

    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("C")'

    HOME=. assert_command_fail dfx canister call --update e2e_project_assets store '("index.js", vec { 1; 2; 3; })'
}
