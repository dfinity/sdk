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

  assert_match "canister http: disabled \(default: enabled\)" "$output"
}

@test "dfx start with a nonstandard subnet type" {
  create_networks_json
  echo "{}" | jq '.local.replica.subnet_type="verifiedapplication"' >"$E2E_NETWORKS_JSON"

  assert_command dfx start --host 127.0.0.1:0 --background --verbose

  assert_match "subnet type: VerifiedApplication \(default: Application\)" "$output"
}

@test "dfx start with nonstandard bitcoin node" {
  assert_command dfx start --host 127.0.0.1:0 --background --bitcoin-node 192.168.0.1:18000 --verbose

  assert_match "bitcoin: enabled \(default: disabled\)" "$output"
  assert_match "nodes: \[192.168.0.1:18000\] \(default: \[127.0.0.1:18444\]\)" "$output"
}

@test "dfx start enabling bitcoin" {
  assert_command dfx start --host 127.0.0.1:0 --background --enable-bitcoin --verbose

  assert_match "bitcoin: enabled \(default: disabled\)" "$output"
}

@test "dfx start in a project without a network definition" {
  mkdir some-project
  cd some-project
  echo "{}" >dfx.json

  # we have to pass 0 for port to avoid conflicts
  assert_command dfx start --host 127.0.0.1:0 --background --verbose

  assert_match "There is no project-specific network 'local' defined in .*/some-project/dfx.json." "$output"
  assert_match "Using the default configuration for the local shared network" "$output"

  assert_match "Local server configuration:" "$output"
  assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:4943\)" "$output"
  assert_match "bitcoin: disabled" "$output"
  assert_match "canister http: enabled" "$output"
  assert_match "subnet type: Application" "$output"
  assert_match "scope: shared" "$output"
}

@test "dfx start outside of a project with default configuration" {
  assert_command dfx start --host 127.0.0.1:0 --background --verbose

  assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)." "$output"
  assert_match "Using the default configuration for the local shared network" "$output"
}

@test "dfx start outside of a project with a shared configuration file" {
  create_networks_json

  assert_command dfx start --background --verbose

  assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)." "$output"
  assert_match "Using the default configuration for the local shared network" "$output"
}


@test "dfx start outside of a project with a shared configuration file that defines the local network" {
  create_networks_json
  echo "{}" | jq '.local.bind="127.0.0.1:0"' >"$E2E_NETWORKS_JSON"

  assert_command dfx start --background --verbose

  assert_match "There is no project-specific network 'local' because there is no project \(no dfx.json\)." "$output"
  assert_match "Using shared network 'local' defined in $DFX_CONFIG_ROOT/.config/dfx/networks.json" "$output"
}

@test "dfx start describes default project-specific network" {
  # almost default: use a dynamic port
  echo "{}" | jq '.networks.local.bind="127.0.0.1:0"' > dfx.json

  assert_command dfx start --background --verbose

  assert_match "Local server configuration:" "$output"
  assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:8000\)" "$output"
  assert_match "bitcoin: disabled" "$output"
  assert_match "canister http: enabled" "$output"
  assert_match "subnet type: Application" "$output"
  assert_match "data directory: .*/working-dir/.dfx/network/local" "$output"
  assert_match "scope: project" "$output"
}

@test "dfx start describes default shared network" {
  # almost default: use a dynamic port
  create_networks_json
  echo "{}" | jq '.local.bind="127.0.0.1:0"' >"$E2E_NETWORKS_JSON"

  assert_command dfx start --background --verbose

  assert_match "Local server configuration:" "$output"
  assert_match "bind address: 127.0.0.1:0 \(default: 127.0.0.1:4943\)" "$output"
  assert_match "bitcoin: disabled" "$output"
  assert_match "canister http: enabled" "$output"
  assert_match "subnet type: Application" "$output"

  if [ "$(uname)" == "Darwin" ]; then
    assert_match "data directory: .*/home-dir/Library/Application Support/org.dfinity.dfx/network/local" "$output"
  elif [ "$(uname)" == "Linux" ]; then
    assert_match "data directory: .*/home-dir/.local/share/dfx/network/local" "$output"
  fi

  assert_match "scope: shared" "$output"
}