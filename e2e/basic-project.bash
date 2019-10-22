#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    # Kill the node manager, the dfx and the client. Ignore errors (ie. if processes aren't
    # running).
    killall dfx nodemanager client |& sed 's/^/killall: /' || true
}

@test "build + install + call + request-status -- greet_as" {
    install_asset greet_as
    dfx_start
    dfx build
    INSTALL_REQUEST_ID=$(dfx canister install 1 canisters/greet.wasm)
    dfx canister request-status $INSTALL_REQUEST_ID

    assert_command dfx canister query 1 greet --type=string Banzai
    assert_eq "Hello, Banzai!"

    # Using call --wait.
    assert_command dfx canister call --wait 1 greet --type=string Bongalo
    assert_eq "Hello, Bongalo!"

    # Using call and request-status.
    assert_command dfx canister call 1 greet --type=string Blueberry
    # At this point $output is the request ID.
    assert_command dfx canister request-status $output
    assert_eq "Hello, Blueberry!"
}

@test "build + install + call + request-status -- counter_wat" {
    skip "WAT not supporting IDL"
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
    install_asset counter_as
    dfx_start
    dfx build
    dfx canister install 1 canisters/counter.wasm --wait

    assert_command dfx canister call 1 read --wait
    assert_eq "0"

    assert_command dfx canister call 1 inc --wait
    assert_eq ""

    assert_command dfx canister query 1 read
    assert_eq "1"

    dfx canister call --wait 1 inc
    assert_command dfx canister query 1 read
    assert_eq "2"

    dfx canister call --wait 1 inc
    assert_command dfx canister query 1 read
    assert_eq "3"

    assert_command dfx canister call 1 inc
    assert_command dfx canister request-status $output

    # Call write.
    assert_command dfx canister call 1 write --type=number 1337 --wait
    assert_eq ""

    # Write has no return value. But we can _call_ read too.
    assert_command dfx canister call 1 read
    assert_command dfx canister request-status $output
    assert_eq "1337"
}
