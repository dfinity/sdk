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

@test "dfx generate requires canister IDs for Motoko canisters" {
    dfx_new hello
    assert_command_fail dfx generate
    assert_contains "Please create canister 'hello_backend' before generating."
}

@test "dfx generate requires canister IDs for dependees of Motoko canister" {
    dfx_new hello
    dfx_start
    jq '.canisters.hello_backend.dependencies[0]="dependee"' dfx.json | sponge dfx.json
    jq '.canisters.dependee.type="assets"' dfx.json | sponge dfx.json
    jq '.canisters.dependee.source=[]' dfx.json | sponge dfx.json
    cat dfx.json
    dfx canister create --all

    # generate fails if Motoko canister itself is not created
    dfx canister stop hello_backend
    dfx canister delete hello_backend --no-withdrawal
    assert_command_fail dfx generate
    assert_contains "Please create canister 'hello_backend' before generating."
    assert_command dfx canister create hello_backend

    # generate fails if a dependee is not created
    dfx canister stop dependee
    dfx canister delete dependee --no-withdrawal
    assert_command_fail dfx generate
    assert_contains "Please create canister 'dependee' before generating."
    assert_command dfx canister create dependee

    assert_command dfx generate
}
