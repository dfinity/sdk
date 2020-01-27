#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
  [ -f client.pid ] && kill -9 "$(cat client.pid)"
  dfx_stop
}

# This test is around 15 seconds to run. I don't think it should be faster without raising the
# flakiness (client start time).
@test "client flag can be passed in" {
  dfx build

  # Start a client manually on a specific port.
  $(dfx cache show)/client --config '
    [http_handler]
    write_port_to="port"
  ' &
  echo $! > client.pid # Use a local file for the client.
  sleep 5 # Wait for client to be available.

  export PORT=$(cat port)
  dfx canister --client http://localhost:$PORT install --all
  dfx canister --client http://localhost:$PORT call e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call --client http://localhost:$PORT e2e_project greet '("Blueberry")'
  assert_command_fail dfx canister call e2e_project greet '("Blueberry")'
}
