#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx stop

    # Verify that processes are killed.
    ! ( ps | grep \ dfx\ start )
}

@test "build + install + call + request-status -- greet_mo" {
    install_asset greet_mo
    dfx_start
    dfx build
    INSTALL_REQUEST_ID=$(dfx canister install hello --async)
    dfx canister request-status $INSTALL_REQUEST_ID

    assert_command dfx canister call hello greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'

    assert_command dfx canister call --query hello greet '("Bongalo")'
    assert_eq '("Hello, Bongalo!")'

    # Using call --async and request-status.
    assert_command dfx canister call --async hello greet '("Blueberry")'
    # At this point $output is the request ID.
    assert_command dfx canister request-status $stdout
    assert_eq '("Hello, Blueberry!")'
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
        dfx canister call 42 write
    done

    run dfx canister call 42 read
    [[ "$stdout" == "A" ]]
    run dfx canister call 42 read
    [[ "$stdout" == "A" ]]

    dfx canister call 42 write
    run dfx canister call 42 read
    [[ "$stdout" == "B" ]]

    dfx canister call 42 write
    run dfx canister call 42 read
    [[ "$stdout" == "C" ]]

    run dfx canister call 42 write --async
    [[ $status == 0 ]]
    dfx canister request-status $stdout
    [[ $status == 0 ]]

    # Write has no return value. But we can _call_ read too.
    run dfx canister call 42 read --async
    [[ $status == 0 ]]
    run dfx canister request-status $stdout
    [[ $status == 0 ]]
    [[ "$stdout" == "D" ]]
}

@test "build + install + call + request-status -- counter_mo" {
    install_asset counter_mo
    dfx_start
    dfx build
    dfx canister install hello

    assert_command dfx canister call hello read
    assert_eq "(0)"

    assert_command dfx canister call hello inc
    assert_eq "()"

    assert_command dfx canister call hello read
    assert_eq "(1)"

    dfx canister call hello inc
    assert_command dfx canister call hello read
    assert_eq "(2)"

    assert_command_fail dfx canister call --query hello inc
    assert_match "inc is not a query method"
    

    dfx canister call hello inc
    assert_command dfx canister call --query hello read
    assert_eq "(3)"

    assert_command dfx canister call hello inc --async
    assert_command dfx canister request-status $stdout

    # Call write.
    assert_command dfx canister call hello write '(1337)'
    assert_eq "()"

    # Write has no return value. But we can _call_ read too.
    assert_command dfx canister call hello read --async
    assert_command dfx canister request-status $stdout
    assert_eq "(1337)"
}

@test "build + install + call -- counter_idl_mo" {
    install_asset counter_idl_mo
    dfx_start
    dfx build
    dfx canister install --all

    assert_command dfx canister call hello inc '(42,false,"testzZ",vec{1;2;3},opt record{head=42; tail=opt record{head=+43; tail=none}})'
    assert_eq "(+43, true, \"uftu{[\", vec { 2; 3; 4; }, opt record { 1158359328 = +43; 1291237008 = opt record { 1158359328 = +44; 1291237008 = none; }; })"
}
