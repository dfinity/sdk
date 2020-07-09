#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "bootstrap fetches candid file" {
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    ID=$(dfx canister id hello)

    assert_command curl http://localhost:8000/_/candid?canisterId="$ID" -o ./web.txt
    assert_command diff canisters/hello/hello.did ./web.txt
    assert_command curl http://localhost:8000/_/candid?canisterId="$ID"\&format=js -o ./web.txt
    # Relax diff as it's produced by two different compilers.
    assert_command diff --ignore-all-space --ignore-blank-lines -I '^}' canisters/hello/hello.did.js ./web.txt
}

@test "forbid starting webserver with a forwarded port" {
    dfx replica &
    echo $! > replica.pid # Use a local file for the replica.
    sleep 5 # Wait for replica to be available.

    assert_command_fail dfx bootstrap --port 8000
    assert_match "Cannot forward API calls to the same bootstrap server"
    kill -TERM `cat replica.pid`
}
