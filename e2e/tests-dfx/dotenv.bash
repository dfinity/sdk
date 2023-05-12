#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop
    
    standard_teardown
}

@test "writes environment variables to .env" {
    dfx_start
    dfx canister create --all
    # .env should also include canisters that are not explicit dependencies
    jq 'del(.canisters.e2e_project_frontend.dependencies)' dfx.json  | sponge dfx.json
    backend_canister=$(dfx canister id e2e_project_backend)
    frontend_canister=$(dfx canister id e2e_project_frontend)
    
    assert_command dfx build e2e_project_frontend

    assert_file_exists .env
    env=$(< .env)
    assert_contains "DFX_NETWORK='local'" "$env"
    assert_contains "CANISTER_ID_E2E_PROJECT_BACKEND='$backend_canister'" "$env"
    assert_contains "E2E_PROJECT_BACKEND_CANISTER_ID='$backend_canister'" "$env"
    assert_contains "CANISTER_ID_E2E_PROJECT_FRONTEND='$frontend_canister'" "$env"
    assert_contains "E2E_PROJECT_FRONTEND_CANISTER_ID='$frontend_canister'" "$env"

    setup_actuallylocal_project_network
    dfx canister create --all --network actuallylocal
    assert_command dfx build --network actuallylocal
    assert_contains "DFX_NETWORK='actuallylocal'" "$(< .env)"
}

@test "writes environment variables to selected file" {
    dfx_start
    dfx canister create --all

    assert_command dfx build --output-env-file flag.env
    assert_file_exists flag.env
    assert_contains "DFX_NETWORK='local'" "$(< flag.env)"

    jq '.output_env_file="json.env"' dfx.json | sponge dfx.json
    assert_command dfx build
    assert_file_exists json.env
    assert_contains "DFX_NETWORK='local'" "$(< json.env)"

    jq 'del(.output_env_file)' dfx.json | sponge dfx.json
    assert_command dfx build
    assert_file_not_exists .env
}

@test "does not clobber existing .env content" {
    dfx_start
    dfx canister create --all
    echo 'foo=bar' > .env
    
    assert_command dfx build
    assert_file_exists .env
    env=$(< .env)
    assert_contains "DFX_NETWORK='local'" "$env"
    assert_contains "foo=bar" "$env"
    
    echo 'baz=quux' >> .env
    assert_command dfx build
    env=$(< .env)
    assert_contains "DFX_NETWORK='local'" "$env"
    assert_contains "foo=bar" "$env"
    assert_contains "baz=quux" "$env"

    # deliberately corrupt the file
    head -n 3 .env | sponge .env
    echo 'baz=quux' >> .env
    assert_command dfx build
    env=$(< .env)
    assert_contains "# END DFX CANISTER ENVIRONMENT VARIABLES" "$env"
    assert_contains "DFX_NETWORK='local'" "$env"
    assert_contains "foo=bar" "$env"
    assert_contains "baz=quux" "$env"
}
