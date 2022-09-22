#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx start with disabled canister http" {
    create_networks_json
    echo "{}" | jq '.local.canister_http.enabled=false' >"$E2E_NETWORKS_JSON"
    assert_command dfx start --host 127.0.0.1:0 --background --verbose

    assert_match "canister http: disabled \(default: enabled\)"
}

@test "dfx start with a nonstandard subnet type" {
    create_networks_json
    echo "{}" | jq '.local.replica.subnet_type="verifiedapplication"' >"$E2E_NETWORKS_JSON"

    assert_command dfx start --host 127.0.0.1:0 --background --verbose

    assert_match "subnet type: VerifiedApplication \(default: Application\)"
}

@test "dfx start with nonstandard bitcoin node" {
    assert_command dfx start --host 127.0.0.1:0 --background --bitcoin-node 192.168.0.1:18000 --verbose

    assert_match "bitcoin: enabled \(default: disabled\)"
    assert_match "nodes: \[192.168.0.1:18000\] \(default: \[127.0.0.1:18444\]\)"
}

@test "dfx start enabling bitcoin" {
    assert_command dfx start --host 127.0.0.1:0 --background --enable-bitcoin --verbose

    assert_match "bitcoin: enabled \(default: disabled\)"
}

@test "dfx start in a project without a network definition" {
    mkdir some-project
    cd some-project
    echo "{}" >dfx.json

    # we have to pass 0 for port to avoid conflicts
    assert_command dfx start --host 127.0.0.1:0 --background --verbose

    assert_match "There is no project-specific network 'local' defined in .*/some-project/dfx.json."
    assert_match "Using the default definition for the 'local' shared network because $DFX_CONFIG_ROOT/.config/dfx/networks.json does not exist."

    assert_match "Local server configuration:"
    assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:4943\)"
    assert_match "bitcoin: disabled"
    assert_match "canister http: enabled"
    assert_match "subnet type: Application"
    assert_match "scope: shared"
}

@test "dfx start outside of a project with default configuration" {
    assert_command dfx start --host 127.0.0.1:0 --background --verbose

    assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)."
    assert_match "Using the default definition for the 'local' shared network because $DFX_CONFIG_ROOT/.config/dfx/networks.json does not exist."
}

@test "dfx start outside of a project with a shared configuration file" {
    create_networks_json

    assert_command dfx start --background --verbose

    assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)."
    assert_match "Using the default definition for the 'local' shared network because $DFX_CONFIG_ROOT/.config/dfx/networks.json does not define it."
}


@test "dfx start outside of a project with a shared configuration file that defines the local network" {
    create_networks_json
    echo "{}" | jq '.local.bind="127.0.0.1:0"' >"$E2E_NETWORKS_JSON"

    assert_command dfx start --background --verbose

    assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)."
    assert_match "Using shared network 'local' defined in $DFX_CONFIG_ROOT/.config/dfx/networks.json"
}

@test "dfx start describes default project-specific network" {
    # almost default: use a dynamic port
    echo "{}" | jq '.networks.local.bind="127.0.0.1:0"' > dfx.json

    assert_command dfx start --background --verbose

    assert_match "Local server configuration:"
    assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:8000\)"
    assert_match "bitcoin: disabled"
    assert_match "canister http: enabled"
    assert_match "subnet type: Application"
    assert_match "data directory: .*/working-dir/.dfx/network/local"
    assert_match "scope: project"
}

@test "dfx start describes default shared network" {
    # almost default: use a dynamic port
    create_networks_json
    echo "{}" | jq '.local.bind="127.0.0.1:0"' >"$E2E_NETWORKS_JSON"

    assert_command dfx start --background --verbose

    assert_match "Local server configuration:"
    assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:4943\)"
    assert_match "bitcoin: disabled"
    assert_match "canister http: enabled"
    assert_match "subnet type: Application"

    if [ "$(uname)" == "Darwin" ]; then
        assert_match "data directory: .*/home-dir/Library/Application Support/org.dfinity.dfx/network/local"
    elif [ "$(uname)" == "Linux" ]; then
        assert_match "data directory: .*/home-dir/.local/share/dfx/network/local"
    fi

    assert_match "scope: shared"
}