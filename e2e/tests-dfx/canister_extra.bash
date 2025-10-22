#!/usr/bin/env bats

load ../utils/_
load ../utils/toxiproxy

setup() {
    standard_setup
    dfx_new hello
}

teardown() {
    dfx_stop
    standard_teardown
}

@test "canister snapshots" {
    dfx_start
    install_asset counter
    dfx deploy

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(1 : nat)'

    dfx canister stop hello_backend
    assert_command dfx canister snapshot create hello_backend
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}
    dfx canister start hello_backend

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(2 : nat)'

    dfx canister stop hello_backend
    assert_command dfx canister snapshot load hello_backend "$snapshot"
    dfx canister start hello_backend
    assert_command dfx canister call hello_backend read
    assert_contains '(1 : nat)'

    assert_command dfx canister snapshot list hello_backend
    assert_match "^${snapshot}:"
    assert_command dfx canister snapshot delete hello_backend "$snapshot"
    assert_command dfx canister snapshot list hello_backend
    assert_contains 'No snapshots found in canister hello_backend'

    assert_command_fail dfx canister snapshot create hello_backend
    assert_contains 'Canister hello_backend is running and snapshots should not be taken of running canisters'
}

@test "canister snapshots download and upload" {
    dfx_start
    install_asset counter
    dfx deploy

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(1 : nat)'

    # Create the first snapshot.
    dfx canister stop hello_backend
    assert_command dfx canister snapshot create hello_backend
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}
    dfx canister start hello_backend

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(2 : nat)'

    # Download the first snapshot.
    OUTPUT_DIR="output"
    mkdir -p "$OUTPUT_DIR"
    assert_command dfx canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR"
    assert_contains "saved to '$OUTPUT_DIR'"

    # Replace the first snapshot.
    dfx canister stop hello_backend
    assert_command dfx canister snapshot create hello_backend --replace "$snapshot"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}

    dfx canister start hello_backend

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(3 : nat)'

    # Load and verify the replaced snapshot.
    dfx canister stop hello_backend
    assert_command dfx canister snapshot load hello_backend "$snapshot"
    dfx canister start hello_backend

    assert_command dfx canister call hello_backend read
    assert_contains '(2 : nat)'

    # Upload to create a new snapshot.
    assert_command dfx canister snapshot upload hello_backend --dir "$OUTPUT_DIR"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot_1=${BASH_REMATCH[1]}

    assert_command dfx canister snapshot list hello_backend
    assert_contains "${snapshot_1}"

    # Load and verify the uploaded snapshot.
    dfx canister stop hello_backend
    assert_command dfx canister snapshot load hello_backend "$snapshot_1"

    dfx canister start hello_backend
    assert_command dfx canister call hello_backend read
    assert_contains '(1 : nat)'
}

@test "canister snapshots download and upload via toxiproxy with high latency" {
    # Start the dfx server on a random port.
    dfx_port=$(get_ephemeral_port)
    echo "dfx_port: $dfx_port"
    dfx_start --host "127.0.0.1:$dfx_port"

    # Start toxiproxy and create a proxy.
    toxiproxy_start
    proxy_port=$(get_ephemeral_port)
    echo "proxy_port: $proxy_port"
    toxiproxy-cli create --listen "127.0.0.1:$proxy_port" --upstream "127.0.0.1:$dfx_port" dfx_proxy

    install_asset counter
    dfx deploy --no-wallet --network "http://127.0.0.1:$proxy_port"

    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(1 : nat)'

    # Create a snapshot to download.
    dfx canister stop hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister snapshot create hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}

    # Add latency to the proxy.
    toxiproxy_add_latency dfx_proxy 1500 300

    # Download through the proxy with latency.
    OUTPUT_DIR="output"
    mkdir -p "$OUTPUT_DIR"
    assert_command dfx canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"
    assert_contains "saved to '$OUTPUT_DIR'"

    # Start the canister again.
    dfx canister start hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(2 : nat)'

    # Upload the snapshot to create a new snapshot.
    assert_command dfx canister snapshot upload hello_backend --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot_1=${BASH_REMATCH[1]}

    # Stop the canister and load the new snapshot.
    dfx canister stop hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister snapshot load hello_backend "$snapshot_1" --network "http://127.0.0.1:$proxy_port"

    # Start the canister again and verify the loaded snapshot.
    dfx canister start hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister call hello_backend read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(1 : nat)'

    toxiproxy_stop || true
}

@test "canister snapshots download and upload via toxiproxy with network drop" {
    # Start the dfx server on a random port.
    dfx_port=$(get_ephemeral_port)
    dfx_start --host "127.0.0.1:$dfx_port"

    # Start toxiproxy and create a proxy.
    toxiproxy_start
    proxy_port=$(get_ephemeral_port)
    toxiproxy_create_proxy dfx_proxy "127.0.0.1:$proxy_port" "127.0.0.1:$dfx_port"

    install_asset counter
    dfx deploy --no-wallet --network "http://127.0.0.1:$proxy_port"

    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(1 : nat)'

    # Create a snapshot to download.
    dfx canister stop hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister snapshot create hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}

    (
        # Toxiproxy doesn't support disabling the proxy with certain amount of data being transferred.
        # So we roughly wait for 0.5 seconds to disable the proxy and fail the snapshot download.
        sleep 0.5
        toxiproxy_toggle_proxy dfx_proxy || true
    ) &

    # Download through the proxy.
    OUTPUT_DIR="output"
    mkdir -p "$OUTPUT_DIR"
    assert_command_fail timeout 10s dfx canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"

    # For debugging.
    echo "OUTPUT_DIR contents:" >&2
    find "$OUTPUT_DIR" -maxdepth 1 -mindepth 1 -type f -printf '%f\t%s\n' >&2

    # Enable the proxy again.
    toxiproxy_toggle_proxy dfx_proxy || true

    # Resume the download through the proxy.
    assert_command dfx canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR" -r --network "http://127.0.0.1:$proxy_port"
    assert_contains "saved to '$OUTPUT_DIR'"

    # For debugging.
    echo "OUTPUT_DIR contents:" >&2
    find "$OUTPUT_DIR" -maxdepth 1 -mindepth 1 -type f -printf '%f\t%s\n' >&2

    # Start the canister again.
    dfx canister start hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(2 : nat)'

    (
        # Toxiproxy doesn't support disabling the proxy with certain amount of data being transferred.
        # So we roughly wait for 0.5 seconds to disable the proxy and fail the snapshot upload.
        sleep 0.5
        toxiproxy_toggle_proxy dfx_proxy || true
    ) &

    # Upload the snapshot to create a new snapshot.
    assert_command_fail timeout 10s dfx canister snapshot upload hello_backend --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"
    assert_match 'snapshot ([0-9a-f]+)'
    snapshot_1=${BASH_REMATCH[1]}

    # For debugging.
    echo "OUTPUT_DIR contents:" >&2
    find "$OUTPUT_DIR" -maxdepth 1 -mindepth 1 -type f -printf '%f\t%s\n' >&2

    # Enable the proxy again.
    toxiproxy_toggle_proxy dfx_proxy || true

    # Resume the upload through the proxy.
    assert_command dfx canister snapshot upload hello_backend --dir "$OUTPUT_DIR" -r "$snapshot_1" --network "http://127.0.0.1:$proxy_port"
    assert_contains "$snapshot_1"

    # Stop the canister and load the new snapshot.
    dfx canister stop hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister snapshot load hello_backend "$snapshot_1" --network "http://127.0.0.1:$proxy_port"

    # Start the canister again and verify the loaded snapshot.
    dfx canister start hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister call hello_backend read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(1 : nat)'

    toxiproxy_stop || true
}

@test "can query a website" {
  dfx_start

  dfx_new
  install_asset canister_http

  dfx deploy

  assert_command dfx canister call e2e_project_backend get_url '("www.githubstatus.com:443","https://www.githubstatus.com:443")'
  assert_contains "Git Operations"
  assert_contains "API Requests"
}
