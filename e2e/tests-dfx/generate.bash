#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx generate creates files" {
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_exists "src/declarations/hello_backend/index.js"
    assert_file_exists "src/declarations/hello_backend/index.d.ts"
}

@test "dfx generate creates only JS files" {
    jq '.canisters.hello_backend.declarations.bindings=["js"]' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_exists "src/declarations/hello_backend/index.js"
    assert_file_not_exists "src/declarations/hello_backend/index.d.ts"
}

@test "dfx generate creates only TS files" {
    jq '.canisters.hello_backend.declarations.bindings=["ts"]' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_not_exists "src/declarations/hello_backend/index.js"
    assert_file_exists "src/declarations/hello_backend/index.d.ts"
}

@test "dfx generate creates only JS & TS files" {
    jq '.canisters.hello_backend.declarations.bindings=["js", "ts"]' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_exists "src/declarations/hello_backend/index.js"
    assert_file_exists "src/declarations/hello_backend/index.d.ts"
}

@test "dfx generate creates only DID files" {
    jq '.canisters.hello_backend.declarations.bindings=["did"]' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_not_exists "src/declarations/hello_backend/index.js"
    assert_file_not_exists "src/declarations/hello_backend/index.d.ts"
}

@test "dfx generate does not create any files" {
    jq '.canisters.hello_backend.declarations.bindings=[]' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx generate

    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.js"
    assert_file_not_exists "src/declarations/hello_backend/hello_backend.did.d.ts"
    assert_file_not_exists "src/declarations/hello_backend/index.js"
    assert_file_not_exists "src/declarations/hello_backend/index.d.ts"
}
