#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
  [ -f replica.pid ] && kill -9 "$(cat replica.pid)"
  dfx_stop
}

# This test is around 15 seconds to run. I don't think it should be faster without raising the
# flakiness (replica start time).
@test "provider flag can be passed in" {
  dfx build

  # Start a replica manually on a specific port.
  $(dfx cache show)/replica --config '
    [http_handler]
    write_port_to="port"
  ' &
  echo $! > replica.pid # Use a local file for the replica.
  sleep 5 # Wait for replica to be available.

  export PORT=$(cat port)
  dfx canister --provider http://localhost:$PORT install --all
  dfx canister --provider http://localhost:$PORT call e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call --provider http://localhost:$PORT e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call e2e_project greet '("Blueberry")'
}
