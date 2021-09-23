#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    [ -f replica.pid ] && kill -9 "$(cat replica.pid)"
    dfx_stop

    standard_teardown
}

# This test is around 15 seconds to run. I don't think it should be faster without raising the
# flakiness (replica start time).
@test "provider flag can be passed in" {
  skip "refactor this test to replace client.bash"
  dfx build

  # Start a replica manually on a specific port.
  "$(dfx cache show)/replica" --config '
    [http_handler]
    write_port_to="port"
  ' &
  echo $! > replica.pid # Use a local file for the replica.
  sleep 5 # Wait for replica to be available.

  x=$(cat port)
  export PORT="$x"
  dfx canister --provider "http://localhost:$PORT" install --all
  dfx canister --provider "http://localhost:$PORT" call e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call --provider "http://localhost:$PORT" e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call e2e_project greet '("Blueberry")'
}

@test "uses local bind address if there is no local network" {
  [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
  # shellcheck disable=SC2094
  cat <<<"$(jq 'del(.networks.local)' dfx.json)" >dfx.json
  dfx_start
}

@test "uses local bind address if there are no networks" {
  [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
  # shellcheck disable=SC2094
  cat <<<"$(jq 'del(.networks)' dfx.json)" >dfx.json
  dfx_start
}

@test "network as URL creates the expected name" {
    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    dfx canister --network "http://127.0.0.1:$webserver_port" create --all
    [ -d ".dfx/http___127_0_0_1_$webserver_port" ]
}
