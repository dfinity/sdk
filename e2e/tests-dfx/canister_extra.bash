#!/usr/bin/env bats

load ../utils/_
load ../utils/toxiproxy

setup() {
    standard_setup
    dfx_new hello
    toxiproxy_start
}

teardown() {
    dfx_stop
    toxiproxy_stop || true
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
    dfx_start --host "127.0.0.1:$dfx_port"

    # Start toxiproxy and create a proxy.
    proxy_port=$(get_ephemeral_port)
    toxiproxy_create_proxy "127.0.0.1:$proxy_port" "127.0.0.1:$dfx_port" proxy_high_latency

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
    toxiproxy_add_latency 1500 300 proxy_high_latency

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

    toxiproxy_delete_proxy proxy_high_latency
}
# bats test_tags=bats:focus
@test "canister snapshots download and upload via toxiproxy with network drop" {
    # Start the dfx server on a random port.
    dfx_port=$(get_ephemeral_port)
    dfx_start --host "127.0.0.1:$dfx_port"

    # Start toxiproxy and create a proxy.
    proxy_port=$(get_ephemeral_port)
    toxiproxy_create_proxy "127.0.0.1:$proxy_port" "127.0.0.1:$dfx_port" proxy_network_drop

    install_asset counter
    dfx deploy --no-wallet --network "http://127.0.0.1:$proxy_port"

    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(1 : nat)'

    # Create a snapshot to download.
    dfx canister stop hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister snapshot create hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}

    # Add a 1MB limit_data toxic to force the snapshot download to fail.
    toxiproxy_add_limit_data limit_download 1000000 proxy_network_drop

    # Download the snapshot should fail.
    OUTPUT_DIR="output"
    mkdir -p "$OUTPUT_DIR"
    assert_command_fail timeout -s9 10s dfx canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"

    # For debugging.
    echo "OUTPUT_DIR contents:" >&2
    find "$OUTPUT_DIR" -maxdepth 1 -mindepth 1 -type f -exec du -h {} \+ >&2

    # Remove the toxic.
    toxiproxy_remove_toxic limit_download proxy_network_drop

    # Resume the download through the proxy.
    assert_command dfx -v canister snapshot download hello_backend "$snapshot" --dir "$OUTPUT_DIR" -r --network "http://127.0.0.1:$proxy_port"
    assert_contains "saved to '$OUTPUT_DIR'"

    # Start the canister again.
    dfx canister start hello_backend --network "http://127.0.0.1:$proxy_port"
    assert_command dfx canister call hello_backend inc_read --network "http://127.0.0.1:$proxy_port"
    assert_contains '(2 : nat)'

    # Add a 1MB limit_data toxic to force the snapshot upload to fail.
    toxiproxy_add_limit_data limit_upload 1000000 proxy_network_drop -u

    # Upload the snapshot should fail.
    assert_command_fail timeout -s9 10s dfx canister snapshot upload hello_backend --dir "$OUTPUT_DIR" --network "http://127.0.0.1:$proxy_port"

    # Loop to get the snapshot id.
    snapshot_1=""
    while IFS= read -r json_file; do
        [ -z "$json_file" ] && continue
        if [[ "$json_file" =~ ^[0-9a-f]+\.json$ ]]; then
            snapshot_1="${json_file%.json}"
            break
        fi
    done < <(find "$OUTPUT_DIR" -maxdepth 1 -type f -name '*.json' -exec basename {} \;)
    if [ -z "$snapshot_1" ]; then
        echo "No matching .json filename ([0-9a-f]+.json) found in $OUTPUT_DIR" >&2
        false
    fi

    # Remove the toxic.
    toxiproxy_remove_toxic limit_upload proxy_network_drop

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

    toxiproxy_delete_proxy proxy_network_drop
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
