#!/usr/bin/env bats

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
}

teardown() {
    # Kill the node manager, the dfx and the client. Ignore errors (ie. if processes aren't
    # running).
    killall dfx nodemanager client || true
}

# Create a new project and starts its client in the background.
dfx_start() {
    # Bats create a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leave a child process running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.
    dfx start --background 3>&-
}

@test "dfx new succeeds" {
    dfx new e2e-project

    test -d e2e-project
    test -f e2e-project/dfinity.json
}

@test "canister query -- greet" {
    dfx new e2e-project
    cd e2e-project
    dfx_start

    run dfx canister query 42 greet Banzai
    echo $output
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Banzai!" ]]
}

@test "canister call wait -- greet" {
    dfx new e2e-project
    cd e2e-project
    dfx_start

    run dfx canister call --wait 42 greet Bongalo
    echo $output
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Bongalo!" ]]
}

@test "canister call + request-status -- greet" {
    dfx new e2e-project
    cd e2e-project
    dfx_start

    run dfx canister call 42 greet Bongalo
    [[ $status == 0 ]]

    run dfx canister request-status $output
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Bongalo!" ]]
}

@test "build + install + call + request-status -- counter_wat" {
    dfx new e2e-project
    cd e2e-project

    cp ${BATS_TEST_DIRNAME}/assets/counter_wat/* .

    dfx build
    dfx_start
    dfx canister install 42 build/counter.wasm

    # Currently the counter is set to 0. We call write which increments it
    # 64 times. This is important because query returns a byte, and 64 is
    # "A" in UTF8. We then just compare and work around the alphabet.
    for _x in {0..64}; do
        dfx canister call --wait 42 write
    done

    run dfx canister query 42 read
    [[ "$output" == "A" ]]
    run dfx canister query 42 read
    [[ "$output" == "A" ]]

    dfx canister call --wait 42 write
    run dfx canister query 42 read
    [[ "$output" == "B" ]]

    dfx canister call --wait 42 write
    run dfx canister query 42 read
    [[ "$output" == "C" ]]

    run dfx canister call 42 write
    [[ $status == 0 ]]
    dfx canister request-status $output
    [[ $status == 0 ]]

    # Write has no return value. But we can _call_ read too.
    run dfx canister call 42 read
    [[ $status == 0 ]]
    run dfx canister request-status $output
    [[ $status == 0 ]]
    [[ "$output" == "D" ]]
}

@test "build + install + call + request-status -- counter_as" {
    skip "This does not work as the AS tries to deserialize IDL, which we dont support yet."
    dfx new e2e-project
    cd e2e-project

    cp ${BATS_TEST_DIRNAME}/assets/counter_as/* .

    dfx build
    dfx_start
    dfx canister install 42 build/counter.wasm

    # Currently the counter is set to 0. We call write which increments it
    # 64 times. This is important because query returns a byte, and 64 is
    # "A" in UTF8. We then just compare and work around the alphabet.
    for _x in {0..64}; do
        dfx canister call --wait 42 inc
    done

    run dfx canister query 42 read
    [[ "$output" == "A" ]]
    run dfx canister query 42 read
    [[ "$output" == "A" ]]

    dfx canister call --wait 42 inc
    run dfx canister query 42 read
    [[ "$output" == "B" ]]

    dfx canister call --wait 42 inc
    run dfx canister query 42 read
    [[ "$output" == "C" ]]

    run dfx canister call 42 inc
    [[ $status == 0 ]]
    dfx canister request-status $output
    [[ $status == 0 ]]

    # Write has no return value. But we can _call_ read too.
    run dfx canister call 42 read
    [[ $status == 0 ]]
    run dfx canister request-status $output
    [[ $status == 0 ]]
    [[ "$output" == "D" ]]
}
