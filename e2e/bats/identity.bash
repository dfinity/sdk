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

@test "calls and query receive the same principal from dfx" {
    install_asset identity_mo
    assert_command dfx build
    dfx_start
    assert_command dfx canister install --all

    ID_CALL=$(dfx canister call e2e_project hashFromCall)
    ID_QUERY=$(dfx canister call e2e_project hashFromQuery)
    if [ "$ID_CALL" -ne "$ID_QUERY" ]; then
      echo "IDs did not match: call '${ID_CALL}' != query '${ID_QUERY}'..." | fail
    fi
}
