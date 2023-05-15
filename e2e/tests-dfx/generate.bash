#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx generate creates files" {
    dfx_new hello
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
    dfx_new hello
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
    dfx_new hello
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
    dfx_new hello
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
    dfx_new hello
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
    dfx_new hello
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

@test "dfx generate succeeds with an encrypted identity without input" {
    dfx_new hello
    dfx_start
    dfx canister create --all

    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
    assert_command dfx identity use alice
    
    assert_command timeout 30s dfx generate
}

@test "dfx generate does not require canister IDs for non-Motoko canisters" {
    dfx_new_rust hello
    assert_command dfx generate
}

@test "dfx generate does not require canister IDs for Motoko canisters" {
    dfx_new hello
    assert_command dfx generate
}

@test "dfx generate --network is still valid" {
    # The option has no effect, but is still accepted to not break existing scripts
    dfx_new hello
    assert_command dfx generate --network local

    # Option is not advertised anymore
    assert_command dfx generate --help
    assert_not_contains "--network"
}
