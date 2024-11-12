#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "network-id contains settings digest" {
  dfx_start

  NETWORK_ID_PATH="$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/network-id"
  SETTINGS_DIGEST=$(jq -r '.settings_digest' "$NETWORK_ID_PATH")
  NETWORK_ID_BY_SETTINGS_DIGEST_PATH="$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/$SETTINGS_DIGEST/network-id"
  assert_command diff "$NETWORK_ID_PATH" "$NETWORK_ID_BY_SETTINGS_DIGEST_PATH"
}

@test "start and stop with different options" {
  dfx_start --artificial-delay 101
  dfx_stop

  # notice: no need to --clean
  dfx_start --artificial-delay 102
  dfx_stop
}

@test "project networks still need --clean" {
  dfx_new hello
  define_project_network

  dfx_start --artificial-delay 101
  dfx stop

  assert_command_fail dfx_start --artificial-delay 102
}

@test "stop and start with other options does not disrupt projects" {
  dfx_start --artificial-delay 101

  dfx_new p1
  assert_command dfx deploy
  CANISTER_ID="$(dfx canister id p1_backend)"

  assert_command dfx stop
  dfx_start --artificial-delay 102
  assert_command dfx stop

  dfx_start --artificial-delay 101

  assert_command dfx canister id p1_backend
  assert_eq "$CANISTER_ID"
}

@test "start and stop outside project" {
  dfx_start

  mkdir subdir
  cd subdir || exit 1
  dfx_new
  assert_command dfx deploy
  CANISTER_ID="$(dfx canister id e2e_project_backend)"
  cd ..
  assert_command dfx canister status "$CANISTER_ID"
  assert_contains "Status: Running"
  assert_command dfx canister stop "$CANISTER_ID"
  assert_command dfx canister status "$CANISTER_ID"
  assert_contains "Status: Stopped"
  assert_command dfx canister start "$CANISTER_ID"
  assert_command dfx canister status "$CANISTER_ID"
  assert_contains "Status: Running"
}

@test "uninstall-code outside of a project" {
  dfx_start

  mkdir subdir
  cd subdir || exit 1
  dfx_new
  assert_command dfx deploy
  CANISTER_ID="$(dfx canister id e2e_project_backend)"
  cd ..
  assert_command dfx canister status "$CANISTER_ID"
  assert_contains "Module hash: 0x"
  assert_command dfx canister uninstall-code "$CANISTER_ID"
  assert_contains "Uninstalling code for canister $CANISTER_ID"
  assert_command dfx canister status "$CANISTER_ID"
  assert_contains "Module hash: None"
}

@test "pocket-ic proxy domain configuration in string form" {
  create_networks_json
  jq '.local.proxy.domain="xyz.domain"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  dfx_start

  domains="$(curl "http://localhost:$(get_pocketic_proxy_config_port)/http_gateway" \
    | jq -c ".[] | select(.port == $(get_webserver_port)) | .domains | sort")"

  assert_eq '["xyz.domain"]' "$domains"
}

@test "pocket-ic proxy domain configuration in vector form" {
  create_networks_json
  jq '.local.proxy.domain=["xyz.domain", "abc.something"]' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  dfx_start

  domains="$(curl "http://localhost:$(get_pocketic_proxy_config_port)/http_gateway" \
    | jq -c ".[] | select(.port == $(get_webserver_port)) | .domains | sort")"

  assert_eq '["abc.something","xyz.domain"]' "$domains"
}

@test "pocket-ic proxy domain configuration from project defaults" {
  dfx_new
  define_project_network

  jq '.defaults.proxy.domain=["xyz.domain", "abc.something"]' dfx.json | sponge dfx.json

  dfx_start

  domains="$(curl "http://localhost:$(get_pocketic_proxy_config_port)/http_gateway" \
    | jq -c ".[] | select(.port == $(get_webserver_port)) | .domains | sort")"

  assert_eq '["abc.something","xyz.domain"]' "$domains"
}

@test "pocket-ic proxy domain configuration from command-line" {
  dfx_start --domain xyz.domain --domain def.somewhere

  domains="$(curl "http://localhost:$(get_pocketic_proxy_config_port)/http_gateway" \
    | jq -c ".[] | select(.port == $(get_webserver_port)) | .domains | sort")"

  assert_eq '["def.somewhere","xyz.domain"]' "$domains"
}

@test "dfx restarts the replica" {
  dfx_new hello
  dfx_start

  install_asset greet
  assert_command dfx deploy
  assert_command dfx canister call hello_backend greet '("Alpha")'
  assert_eq '("Hello, Alpha!")'

  REPLICA_PID=$([[ "$USE_POCKETIC" ]] && get_pocketic_pid || get_replica_pid)

  echo "replica pid is $REPLICA_PID"

  [[ "$USE_POCKETIC" ]] && curl -X DELETE "http://localhost:$(get_pocketic_port)/instances/0"
  kill -KILL "$REPLICA_PID"
  assert_process_exits "$REPLICA_PID" 15s

  timeout 15s sh -c \
    'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
    || (echo "replica did not restart" && ps aux && exit 1)
  wait_until_replica_healthy

  # Sometimes initially get an error like:
  #     IC0537: Attempt to execute a message on canister <>> which contains no Wasm module
  # but the condition clears.
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting
  # even after the above, still sometimes fails with
  #     IC0208: Certified state is not available yet. Please try again...
  sleep 10
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting

  assert_command dfx canister call hello_backend greet '("Omega")'
  assert_eq '("Hello, Omega!")'
}

@test "dfx restarts pocketic proxy" {
  dfx_new_assets hello
  dfx_start

  install_asset greet
  assert_command dfx deploy
  assert_command dfx canister call hello_backend greet '("Alpha")'
  assert_eq '("Hello, Alpha!")'

  POCKETIC_PROXY_PID=$(get_pocketic_proxy_pid)

  echo "pocket-ic proxy pid is $POCKETIC_PROXY_PID"

  kill -KILL "$POCKETIC_PROXY_PID"
  assert_process_exits "$POCKETIC_PROXY_PID" 15s

  ID=$(dfx canister id hello_frontend)

  timeout 15s sh -c \
    "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY\"/webserver-port)/sample-asset.txt?canisterId=$ID; do echo waiting for pocket-ic proxy to restart; sleep 1; done" \
    || (echo "pocket-ic proxy did not restart" && ps aux && exit 1)

  assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx restarts pocketic proxy when the replica restarts" {
  dfx_new_assets hello
  dfx_start

  install_asset greet
  assert_command dfx deploy
  assert_command dfx canister call hello_backend greet '("Alpha")'
  assert_eq '("Hello, Alpha!")'

  REPLICA_PID=$([[ "$USE_POCKETIC" ]] && get_pocketic_pid || get_replica_pid)
  POCKETIC_PROXY_PID=$(get_pocketic_proxy_pid)

  echo "replica pid is $REPLICA_PID"
  echo "pocket-ic proxy pid is $POCKETIC_PROXY_PID"

  [[ "$USE_POCKETIC" ]] && curl -X DELETE "http://localhost:$(get_pocketic_port)/instances/0"
  kill -KILL "$REPLICA_PID"
  assert_process_exits "$REPLICA_PID" 15s
  assert_process_exits "$POCKETIC_PROXY_PID" 15s

  timeout 15s sh -c \
    'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
    || (echo "replica did not restart" && ps aux && exit 1)
  wait_until_replica_healthy

  # Sometimes initially get an error like:
  #     IC0537: Attempt to execute a message on canister <>> which contains no Wasm module
  # but the condition clears.
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting
  # even after the above, still sometimes fails with
  #     IC0208: Certified state is not available yet. Please try again...
  sleep 10
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting

  assert_command dfx canister call hello_backend greet '("Omega")'
  assert_eq '("Hello, Omega!")'

  ID=$(dfx canister id hello_frontend)

  timeout 15s sh -c \
    "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/webserver-port\")/sample-asset.txt?canisterId=$ID; do echo waiting for pocket-ic proxy to restart; sleep 1; done" \
    || (echo "pocket-ic proxy did not restart" && ps aux && exit 1)

  assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx start honors replica port configuration" {
  create_networks_json
  replica_port=$(get_ephemeral_port)
  jq ".local.replica.port=$replica_port" "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  dfx_start

  assert_command dfx info replica-port
  assert_eq "$replica_port"
}

@test "dfx starts replica with subnet_type application - project defaults" {
  install_asset subnet_type/project_defaults/application
  define_project_network
  jq '.defaults.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: Application"
}

@test "dfx starts replica with subnet_type verifiedapplication - project defaults" {
  install_asset subnet_type/project_defaults/verified_application
  define_project_network
  jq '.defaults.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: VerifiedApplication"
}

@test "dfx starts replica with subnet_type system - project defaults" {
  install_asset subnet_type/project_defaults/system
  define_project_network
  jq '.defaults.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: System"
}

@test "dfx starts replica with subnet_type application - local network" {
  install_asset subnet_type/project_network_settings/application
  define_project_network
  jq '.networks.local.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: Application"
}

@test "dfx starts replica with subnet_type verifiedapplication - local network" {
  install_asset subnet_type/project_network_settings/verified_application
  define_project_network
  jq '.networks.local.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: VerifiedApplication"
}

@test "dfx starts replica with subnet_type system - local network" {
  install_asset subnet_type/project_network_settings/system
  define_project_network
  jq '.networks.local.replica.log_level="info"' dfx.json | sponge dfx.json

  assert_command dfx start --background
  assert_match "subnet_type: System"
}


@test "dfx starts replica with subnet_type application - shared network" {
  install_shared_asset subnet_type/shared_network_settings/application
  jq '.local.replica.log_level="info"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx start --background
  assert_match "subnet_type: Application"
}

@test "dfx starts replica with subnet_type verifiedapplication - shared network" {
  install_shared_asset subnet_type/shared_network_settings/verified_application
  jq '.local.replica.log_level="info"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx start --background
  assert_match "subnet_type: VerifiedApplication"
}

@test "dfx starts replica with subnet_type system - shared network" {
  install_shared_asset subnet_type/shared_network_settings/system
  jq '.local.replica.log_level="info"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx start --background
  assert_match "subnet_type: System"
}

@test "dfx start detects if dfx is already running - shared network" {
  dfx_new hello
  dfx_start

  assert_command_fail dfx start
  assert_match "dfx is already running"
}

@test "dfx start for shared network warns about default settings specified in dfx.json that do not apply" {
  dfx_new hello

  IGNORED_MESSAGE="Ignoring the 'defaults' field in dfx.json because project settings never apply to the shared network."
  APPLY_SETTINGS_MESSAGE="To apply these settings to the shared network, define them in .*/config-root/.config/dfx/networks.json like so"

  jq 'del(.defaults)' dfx.json | sponge dfx.json
  jq '.defaults.bitcoin.enabled=true' dfx.json | sponge dfx.json
  assert_command dfx start --background
  assert_contains "$IGNORED_MESSAGE"
  assert_match "$APPLY_SETTINGS_MESSAGE"
  assert_contains '"bitcoin": {'
  assert_not_contains '"replica"'
  assert_not_contains '"canister_http"'
  assert_command dfx stop

  jq 'del(.defaults)' dfx.json | sponge dfx.json
  jq '.defaults.replica.log_level="info"' dfx.json | sponge dfx.json
  assert_command dfx start --background
  assert_contains "$IGNORED_MESSAGE"
  assert_match "$APPLY_SETTINGS_MESSAGE"
  assert_not_contains '"bitcoin"'
  assert_contains '"replica": {'
  assert_not_contains '"canister_http"'
  assert_command dfx stop

  jq 'del(.defaults)' dfx.json | sponge dfx.json
  jq '.defaults.canister_http.enabled=false' dfx.json | sponge dfx.json
  assert_command dfx start --background
  assert_contains "$IGNORED_MESSAGE"
  assert_match "$APPLY_SETTINGS_MESSAGE"
  assert_not_contains '"bitcoin"'
  assert_not_contains '"replica"'
  assert_contains '"canister_http": {'
  assert_command dfx stop

  jq 'del(.defaults)' dfx.json | sponge dfx.json
  jq '.defaults.bitcoin.enabled=true' dfx.json | sponge dfx.json
  jq '.defaults.replica.log_level="info"' dfx.json | sponge dfx.json
  jq '.defaults.canister_http.enabled=false' dfx.json | sponge dfx.json
  assert_command dfx start --background
  assert_contains "$IGNORED_MESSAGE"
  assert_match "$APPLY_SETTINGS_MESSAGE"
  assert_contains '"bitcoin": {'
  assert_contains '"replica": {'
  assert_contains '"canister_http": {'
  assert_command dfx stop
}

@test "dfx starts replica with correct log level - project defaults" {
  dfx_new
  jq '.defaults.replica.log_level="warning"' dfx.json | sponge dfx.json
  define_project_network

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.defaults.replica.log_level="critical"' dfx.json | sponge dfx.json
  assert_command dfx start --background --verbose --clean
  assert_match "log level: Critical"
}

@test "dfx starts replica with correct log level - local network" {
  dfx_new
  jq '.networks.local.replica.log_level="warning"' dfx.json | sponge dfx.json
  define_project_network

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.networks.local.replica.log_level="critical"' dfx.json | sponge dfx.json
  assert_command dfx start --background --verbose --clean
  assert_match "log level: Critical"
}

@test "dfx starts replica with correct log level - shared network" {
  dfx_new
  create_networks_json
  jq '.local.replica.log_level="warning"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.local.replica.log_level="critical"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx start --background --verbose --clean
  assert_match "log level: Critical"
}

@test "debug print statements work with default log level" {
  dfx_new
  install_asset print
  dfx_start 2>stderr.txt
  assert_command dfx deploy
  assert_command dfx canister call e2e_project hello
  sleep 2
  run tail -2 stderr.txt
  assert_match "Hello, World! from DFINITY"
}

@test "modifying networks.json does not require --clean on restart" {
  dfx_start
  dfx stop
  assert_command dfx_start
  dfx stop
  jq -n '.local.replica.log_level="warning"' > "$E2E_NETWORKS_JSON"
  assert_command dfx_start
}

@test "project-local networks require --clean if dfx.json was updated" {
  dfx_new
  define_project_network
  dfx_start
  dfx stop
  assert_command dfx_start
  dfx stop
  jq -n '.local.replica.log_level="warning"' > "$E2E_NETWORKS_JSON"
  assert_command dfx_start
  dfx stop
  jq '.networks.local.replica.log_level="warning"' dfx.json | sponge dfx.json
  assert_command_fail dfx_start
  assert_contains  "The network state can't be reused with this configuration. Rerun with \`--clean\`."
  assert_command dfx_start --force
  dfx stop
  assert_command dfx_start --clean
}

@test "flags count as configuration modification and require --clean for a project network" {
  dfx_new
  define_project_network

  dfx start --background
  dfx stop
  assert_command_fail dfx start --artificial-delay 100 --background
  assert_contains "The network state can't be reused with this configuration. Rerun with \`--clean\`."
  assert_command dfx start --artificial-delay 100 --clean --background
  dfx stop
  assert_command dfx start --artificial-delay 100 --background
  dfx stop
  assert_command_fail dfx start --background
  assert_contains "The network state can't be reused with this configuration. Rerun with \`--clean\`."
  assert_command dfx start --force --background
}

@test "dfx start then ctrl-c won't hang and panic but stop actors quickly" {
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/ctrl_c_right_after_dfx_start.exp"
}

@test "dfx-started processes can be killed with dfx killall" {
    dfx_start
    dfx killall
    assert_command_fail pgrep dfx replica pocket-ic
}
