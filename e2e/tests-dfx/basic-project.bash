#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    export RUST_BACKTRACE=1

    dfx_new hello
}

teardown() {
  dfx_stop
}

@test "build + install + call + request-status -- greet_mo" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    # INSTALL_REQUEST_ID=$(dfx canister install hello --async)
    # dfx canister request-status $INSTALL_REQUEST_ID
    dfx canister install hello

    assert_command dfx canister call hello greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'

    assert_command dfx canister call hello greet --type raw '4449444c00017103e29883'
    assert_eq '("Hello, ☃!")'

    assert_command dfx canister call --query hello greet '("Bongalo")'
    assert_eq '("Hello, Bongalo!")'

    # Using call --async and request-status.
    # Call with user Identity as Sender
    assert_command dfx canister --no-wallet call --async hello greet Blueberry
    # At this point $output is the request ID.
    # shellcheck disable=SC2154
    assert_command dfx canister request-status "$stdout" "$(dfx canister id hello)"
    assert_eq '("Hello, Blueberry!")'

    # Call using the wallet's call forwarding
    assert_command dfx canister call --async hello greet Blueberry
    # At this point $output is the request ID.
    # shellcheck disable=SC2154
    assert_command dfx canister request-status "$stdout" "$(dfx canister id hello)"
    assert_eq '( variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01q\11Hello, Blueberry!" } }, )'
}

@test "build + install + call + request-status -- counter_mo" {
    install_asset counter
    dfx_start
    dfx canister create --all
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

    assert_command dfx canister call hello read --output raw
    assert_eq "4449444c00017d02"

    assert_command_fail dfx canister call --query hello inc
    assert_match "inc is not a query method"


    dfx canister call hello inc
    assert_command dfx canister call --query hello read
    assert_eq "(3)"

    assert_command dfx canister call hello inc --async
    assert_command dfx canister request-status "$stdout" "$(dfx canister id hello)"

    # Call write.
    assert_command dfx canister call hello write 1337
    assert_eq "()"

    # Write has no return value. But we can _call_ read too.
    # Call with user Identity as Sender
    assert_command dfx canister --no-wallet call hello read --async
    assert_command dfx canister request-status "$stdout" "$(dfx canister id hello)"
    assert_eq "(1_337)"

    # Call using the wallet's call forwarding
    assert_command dfx canister call hello read --async
    assert_command dfx canister request-status "$stdout" "$(dfx canister id hello)"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01}\b9\0a" } })'

}

@test "build + install + call -- counter_idl_mo" {
    install_asset counter_idl
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    assert_command dfx canister call hello inc '(42,false,"testzZ",vec{1;2;3},opt record{head=42; tail=opt record{head=+43; tail=null}}, variant { cons=record{ 42; variant { cons=record{43; variant { nil }} } } })'  --output idl
    assert_eq "(43, true, \"uftu{[\", vec { 2; 3; 4;}, opt record { head = 43; tail = opt record { head = 44; tail = null;};}, variant { cons = record { 43; variant { cons = record { 44; variant { nil };} };} })"
}

@test "build + install + call -- matrix_multiply_mo" {
    install_asset matrix_multiply
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    assert_command dfx canister call hello multiply '(vec{vec{1;2};vec{3;4};vec{5;6}},vec{vec{1;2;3};vec{4;5;6}})'
    assert_eq "(vec { vec { 9; 12; 15 }; vec { 19; 26; 33 }; vec { 29; 40; 51 } })"
}
