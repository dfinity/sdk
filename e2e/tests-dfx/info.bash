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

@test "displays the telemetry log path" {
  assert_command dfx info telemetry-log-path

  if [ "$(uname)" == "Darwin" ]; then
    assert_eq "$HOME/Library/Caches/org.dfinity.dfx/telemetry/telemetry.log"
  elif [ "$(uname)" == "Linux" ]; then
    assert_eq "$HOME/.cache/dfx/telemetry/telemetry.log"
  else
     echo "Unsupported OS" | fail
  fi
}

@test "displays the replica port" {
  if [[ ! "$USE_REPLICA" ]]
  then
    assert_command_fail dfx info pocketic-config-port
    assert_contains "No PocketIC port found"
    dfx_start
    assert_command_fail dfx info replica-port
    assert_contains "The running server is PocketIC"
    assert_command dfx info pocketic-config-port
    assert_eq "$(get_pocketic_port)"
  else
    assert_command_fail dfx info replica-port
    assert_contains "No replica port found"
    dfx_start
    assert_command_fail dfx info pocketic-config-port
    assert_contains "The running server is a native replica"
    assert_command dfx info replica-port
    assert_eq "$(get_replica_port)"
  fi
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

@test "displays Candid UI URL" {
  assert_command dfx info candid-ui-url --ic
  # shellcheck disable=SC2154
  assert_eq "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/" "$stdout"

  # Before deployment the UI canister does not exist yet
  assert_command_fail dfx info candid-ui-url
  assert_contains "Candid UI not installed on network local."

  dfx_start
  assert_command dfx deploy e2e_project_backend
  assert_command dfx info candid-ui-url  
  assert_eq "http://127.0.0.1:$(dfx info webserver-port)/?canisterId=$(dfx canister id __Candid_UI)"
}

@test "security-policy shows standard/hardened headers with comments" {
  assert_command dfx info security-policy
  assert_contains "Content-Security-Policy" # One of the headers in the standard policy
  assert_contains "X-XSS-Protection" # One of the headers in the standard policy
  assert_contains '"nosniff"' # One of the values in the standard policy
  assert_contains '"same-origin"' # One of the values in the standard policy
  assert_contains "// Notes about the CSP below:" # One of the comment lines in the standard policy, together with `//` to mark it as a comment in json5
  assert_contains "// - We added img-src data: because data: images are used often." # One of the comment lines in the standard policy, together with `//` to mark it as a comment in json5
}

@test "security-policy produces valid json5 headers" {
  dfx_new_frontend
  install_asset assetscanister
  touch src/e2e_project_frontend/assets/thing.json

  dfx_start
  dfx canister create --all

  echo "[
    {
      \"match\": \"**/*\",
      \"headers\": {
        $(dfx info security-policy)
      }
    }
  ]" > src/e2e_project_frontend/assets/.ic-assets.json5
  cat src/e2e_project_frontend/assets/.ic-assets.json5

  # fails if the the above produced invalid json5
  assert_command dfx deploy
}

@test "prints the path to the config file" {
  cfg=$(dfx info config-json-path)
  assert_command test -f "$cfg"
  assert_command jq . "$cfg"
}

@test "prints the pocket-ic default effective canister id" {
  dfx_start
  if [[ $USE_REPLICA ]]; then
    assert_command dfx info default-effective-canister-id
    assert_eq "$stdout" rwlgt-iiaaa-aaaaa-aaaaa-cai
  else
    local topology expected_id64 expected_id
    topology=$(curl "http://localhost:$(get_webserver_port)/_/topology")
    expected_id64=$(jq -r .default_effective_canister_id.canister_id <<<"$topology")
    expected_id=$(cat <(crc32 <(base64 -d <<<"$expected_id64") | xxd -r -p) <(base64 -d <<<"$expected_id64") | base32 \
      | tr -d = | tr '[:upper:]' '[:lower:]' | fold -w5 | paste -sd- -)
    assert_command dfx info default-effective-canister-id
    assert_eq "$stdout" "$expected_id"
  fi
}
