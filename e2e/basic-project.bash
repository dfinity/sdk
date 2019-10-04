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

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/assets/$1/
    cp -R $ASSET_ROOT/* .
}

@test "dfx new succeeds" {
    dfx new e2e-project

    test -d e2e-project
    test -f e2e-project/dfinity.json
}

@test "build + install + call + request-status -- greet_as" {
    dfx new e2e-project
    cd e2e-project

    install_asset greet_as
    dfx_start
    dfx build
    INSTALL_REQUEST_ID=$(dfx canister install 1 build/greet.wasm)
    dfx canister request-status $INSTALL_REQUEST_ID

    run dfx canister query 1 greet --type=string Banzai
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Banzai!" ]]

    # Using call --wait.
    run dfx canister call --wait 1 greet --type=string Bongalo
    echo $output
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Bongalo!" ]]

    # Using call and request-status.
    run dfx canister call 1 greet --type=string Blueberry
    [[ $status == 0 ]]

    # At this point $output is the request ID.
    run dfx canister request-status $output
    [[ $status == 0 ]]
    [[ "$output" == "Hello, Blueberry!" ]]
}

@test "build + install + call + request-status -- counter_wat" {
    skip "WAT not supporting IDL"
    dfx new e2e-project
    cd e2e-project

    install_asset counter_wat

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
    dfx new e2e-project
    cd e2e-project

    install_asset counter_as
    dfx_start
    dfx build
    dfx canister install 1 build/counter.wasm

    run dfx canister call 1 read --wait
    [[ "$output" == "0" ]]
    run dfx canister call 1 inc --wait
    [[ "$output" == "" ]]
    run dfx canister query 1 read
    [[ "$output" == "1" ]]

    dfx canister call --wait 1 inc
    run dfx canister query 1 read
    [[ "$output" == "2" ]]

    dfx canister call --wait 1 inc
    run dfx canister query 1 read
    [[ "$output" == "3" ]]

    run dfx canister call 1 inc
    [[ $status == 0 ]]
    dfx canister request-status $output
    [[ $status == 0 ]]

    # Call write.
    run dfx canister call 1 write --type=number 1337 --wait
    [[ $status == 0 ]]

    # Write has no return value. But we can _call_ read too.
    run dfx canister call 1 read
    [[ $status == 0 ]]
    run dfx canister request-status $output
    [[ $status == 0 ]]
    [[ "$output" == "1337" ]]
}
