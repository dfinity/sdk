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

@test "displays the default webserver port for the local shared network" {
    assert_command dfx info webserver-port
    assert_eq "4943"
}

@test "displays the webserver port for a project-specific network" {
    define_project_network
    assert_command dfx info webserver-port
    assert_eq "8000"
}

@test "displays path to networks.json" {
    assert_command dfx info networks-json-path
    assert_eq "$E2E_NETWORKS_JSON"
}

@test "displays the replica revision included in dfx" {
    nix_sources_path="${BATS_TEST_DIRNAME}/../../nix/sources.json"
    expected_rev="$(jq -r '."replica-x86_64-linux".rev' "$nix_sources_path")"

    assert_command dfx info replica-rev
    assert_eq "$expected_rev"
}
