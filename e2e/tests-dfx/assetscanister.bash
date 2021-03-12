#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

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

    assert_command dfx canister call --query e2e_project_assets retrieve '("/binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01 \80\0aw12 \00xy\0aKL\0b\0ajk")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("/text-with-newlines.txt")' --output idl
    assert_eq '(blob "cherries\0ait'\''s cherry season\0aCHERRIES")'

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

@test "asset canister supports http requests" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    ID=$(dfx canister id e2e_project_assets)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/text-with-newlines.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_eq "cherries
it's cherry season
CHERRIES" "$stdout"
}

@test 'can store arbitrarily large files' {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref" # this takes too long for ic-ref's wasm interpreter

    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    dd if=/dev/urandom of=src/e2e_project_assets/assets/large-asset.bin bs=1000000 count=6

    dfx deploy

    assert_command dfx canister call --query e2e_project_assets get '(record{key="/large-asset.bin";accept_encodings=vec{"identity"}})'
    assert_match 'total_length = 6_000_000'
    assert_match 'content_type = "application/octet-stream"'
    assert_match 'content_encoding = "identity"'

    assert_command dfx canister call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=2})'

    assert_command dfx canister call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=3})'
    assert_command_fail dfx canister call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=4})'
}

@test "list() and keys() return asset keys" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets list
    assert_match '"/binary/noise.txt"'
    assert_match '"/text-with-newlines.txt"'
    assert_match '"/sample-asset.txt"'

    assert_command dfx canister call --query e2e_project_assets keys
    assert_match '"/binary/noise.txt"'
    assert_match '"/text-with-newlines.txt"'
    assert_match '"/sample-asset.txt"'
}
