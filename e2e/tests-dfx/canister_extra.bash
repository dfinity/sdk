#!/usr/bin/env bats

load ../utils/_

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

@test "can query a website" {
  dfx_start

  dfx_new
  install_asset canister_http

  dfx deploy

  assert_command dfx canister call e2e_project_backend get_url '("www.githubstatus.com:443","https://www.githubstatus.com:443")'
  assert_contains "Git Operations"
  assert_contains "API Requests"
}