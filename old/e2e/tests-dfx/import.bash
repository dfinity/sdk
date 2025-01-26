#!/usr/bin/env bats

load ../utils/_

assets="$(dirname "$BATS_TEST_FILENAME")"/../assets

setup() {
    standard_setup

    dfx_new
}

teardown() {
    stop_webserver

    dfx_stop

    standard_teardown
}

test_project_import() {
    DFX_JSON_LOCATION="$1"

    # this test is meant to demonstrate that the various
    assert_command dfx beta project import "$DFX_JSON_LOCATION" --prefix "pfx-" --network-mapping ic=mainnet --all

    jq . dfx.json

    assert_command jq -r '.canisters."pfx-normal-canister".candid' dfx.json
    assert_eq "candid/pfx-normal-canister.did"
    # shellcheck disable=SC2154
    assert_files_eq \
      "${assets}/project-import/project-directory/normal-canister-directory/some-subdirectory/the-candid-filename.did" \
      "candid/pfx-normal-canister.did"

    assert_command jq -r '.canisters."pfx-normal-canister".remote.id.ic' dfx.json
    assert_eq "rrkah-fqaaa-aaaaa-aaaaq-cai"

    assert_command jq -r '.canisters."pfx-sibling".candid' dfx.json
    assert_eq "candid/pfx-sibling.did"
    assert_files_eq \
      "${assets}/project-import/sibling-project/canister/canister/the-sibling-candid-definition.did" \
      "candid/pfx-sibling.did"
}

@test "dfx project import from filesystem" {
    test_project_import "${assets}/project-import/project-directory/dfx.json"
}

@test "dfx project import from url" {
    start_webserver --directory "${assets}/project-import"

    test_project_import "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json"
}

test_project_import_specific_canister() {
    LOCATION="$1"

    # this test is meant to demonstrate that the various
    dfx beta project import "$LOCATION" normal-canister

    jq . dfx.json

    assert_command jq -r '.canisters."normal-canister".candid' dfx.json
    assert_eq "candid/normal-canister.did"
    assert_files_eq \
      "${assets}/project-import/project-directory/normal-canister-directory/some-subdirectory/the-candid-filename.did" \
      "candid/normal-canister.did"

    assert_command jq -r '.canisters.sibling.candid' dfx.json
    assert_eq "null"
}

@test "dfx project import specific canister" {
    test_project_import_specific_canister "${assets}/project-import/project-directory/dfx.json"
}

@test "import from url" {
    start_webserver --directory "${assets}/project-import"

    test_project_import_specific_canister "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json"
}

@test "project import from filesystem with no canister_ids.json" {
    mkdir www
    cp -R "${assets}/project-import" www/
    rm www/project-import/project-directory/canister_ids.json

    start_webserver --directory "www/project-import"

    assert_command dfx beta project import www/project-import/project-directory/dfx.json --all
}

@test "project import from url with no canister_ids.json" {
    mkdir www
    cp -R "${assets}/project-import" www/
    rm www/project-import/project-directory/canister_ids.json

    start_webserver --directory "www/project-import"

    assert_command dfx beta project import "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json" --all
}

