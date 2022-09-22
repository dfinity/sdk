#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx restarts the replica" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)

    echo "replica pid is $REPLICA_PID"

    kill -KILL "$REPLICA_PID"
    assert_process_exits "$REPLICA_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting
    # even after the above, still sometimes fails with
    #     IC0515: Certified state is not available yet. Please try again...
    sleep 10
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'
}

@test "dfx restarts icx-proxy" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    ICX_PROXY_PID=$(get_icx_proxy_pid)

    echo "icx-proxy pid is $ICX_PROXY_PID"

    kill -KILL "$ICX_PROXY_PID"
    assert_process_exits "$ICX_PROXY_PID" 15s

    ID=$(dfx canister id hello_frontend)

    timeout 15s sh -c \
      "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY\"/webserver-port)/sample-asset.txt?canisterId=$ID; do echo waiting for icx-proxy to restart; sleep 1; done" \
      || (echo "icx-proxy did not restart" && ps aux && exit 1)

    assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx restarts icx-proxy when the replica restarts" {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref"

    dfx_new hello
    dfx_start

    install_asset greet
    assert_command dfx deploy
    assert_command dfx canister call hello_backend greet '("Alpha")'
    assert_eq '("Hello, Alpha!")'

    REPLICA_PID=$(get_replica_pid)
    ICX_PROXY_PID=$(get_icx_proxy_pid)

    echo "replica pid is $REPLICA_PID"
    echo "icx-proxy pid is $ICX_PROXY_PID"

    kill -KILL "$REPLICA_PID"
    assert_process_exits "$REPLICA_PID" 15s
    assert_process_exits "$ICX_PROXY_PID" 15s

    timeout 15s sh -c \
      'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
      || (echo "replica did not restart" && ps aux && exit 1)
    wait_until_replica_healthy

    # Sometimes initially get an error like:
    #     IC0304: Attempt to execute a message on canister <>> which contains no Wasm module
    # but the condition clears.
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting
    # even after the above, still sometimes fails with
    #     IC0515: Certified state is not available yet. Please try again...
    sleep 10
    timeout 30s sh -c \
      "until dfx canister call hello_backend greet '(\"wait\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
      || (echo "canister call did not succeed") # but continue, for better error reporting

    assert_command dfx canister call hello_backend greet '("Omega")'
    assert_eq '("Hello, Omega!")'

    ID=$(dfx canister id hello_frontend)

    timeout 15s sh -c \
      "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/webserver-port\")/sample-asset.txt?canisterId=$ID; do echo waiting for icx-proxy to restart; sleep 1; done" \
      || (echo "icx-proxy did not restart" && ps aux && exit 1)

    assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
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

@test "dfx starts replica with correct log level - project defaults" {
    dfx_new
    jq '.defaults.replica.log_level="warning"' dfx.json | sponge dfx.json
    define_project_network

    assert_command dfx start --background --verbose
    assert_match "log level: Warning"
    assert_command dfx stop

    jq '.defaults.replica.log_level="critical"' dfx.json | sponge dfx.json
    assert_command dfx start --background --verbose
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
    assert_command dfx start --background --verbose
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
    assert_command dfx start --background --verbose
    assert_match "log level: Critical"
}

@test "debug print statements work with default log level" {
    [ "$USE_IC_REF" ] && skip "printing from mo not specified"

    dfx_new
    install_asset print
    dfx_start 2>stderr.txt
    assert_command dfx deploy
    assert_command dfx canister call e2e_project hello
    sleep 2
    run tail -2 stderr.txt
    assert_match "Hello, World! from DFINITY"
}
