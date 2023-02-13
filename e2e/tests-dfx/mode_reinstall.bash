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

@test "install --mode=reinstall --all fails" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx canister install --mode=reinstall --all

    assert_match "The --mode=reinstall is only valid when specifying a single canister, because reinstallation destroys all data in the canister."
}

@test "install --mode=reinstall fails if no canister is provided" {
    # This fails because clap protects against it.
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx canister install --mode=reinstall
    assert_match "required arguments were not provided"
    assert_match "--all"
}

@test "reinstall succeeds when a canister name is provided" {
    dfx_start
    dfx deploy

    # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
    # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
    echo yes | (
        assert_command dfx canister install --mode=reinstall hello_backend

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"
        assert_match "Reinstalling code for canister hello_backend"
    )
}

@test "install --mode=reinstall refused if not approved" {
    dfx_start
    dfx deploy

    echo no | (
        assert_command_fail dfx canister install --mode=reinstall hello_backend

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"

        assert_not_match "Installing code for canister"
        assert_match "Refusing to install canister without approval"
    )
}

@test "deploy --mode=reinstall fails if no canister name specified" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx deploy --mode=reinstall

    assert_match "The --mode=reinstall is only valid when deploying a single canister, because reinstallation destroys all data in the canister."
}

@test "deploy --mode=reinstall succeeds when a canister name is provided" {
    dfx_start
    dfx deploy

    # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
    # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
    echo yes | (
        assert_command dfx deploy --mode=reinstall hello_backend

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"
        assert_match "Reinstalling code for canister hello_backend"
    )
}

@test "deploy --mode=reinstall refused if not approved" {
    dfx_start
    dfx deploy

    echo no | (
        assert_command_fail dfx deploy --mode=reinstall hello_backend

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"

        assert_not_match "Installing code for canister"
        assert_match "Refusing to install canister without approval"
    )
}

@test "deploy --mode=reinstall does not reinstall dependencies" {
    dfx_start
    install_asset counter
    dfx deploy

    assert_command dfx canister call hello_backend read
    assert_eq "(0 : nat)"

    assert_command dfx canister call hello_backend inc
    assert_eq "()"

    assert_command dfx canister call hello_backend read
    assert_eq "(1 : nat)"

    dfx canister call hello_backend inc
    assert_command dfx canister call hello_backend read
    assert_eq "(2 : nat)"


    # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
    # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
    echo "yes" | (
        assert_command dfx deploy --mode=reinstall hello_frontend

        assert_match "You are about to reinstall the hello_frontend canister."
        assert_not_match "You are about to reinstall the hello_backend canister."
        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"
        assert_match "Reinstalling code for canister hello_frontend,"
    )

    # the hello_backend canister should not have been upgraded (which would reset the non-stable var)
    assert_command dfx canister call hello_backend read
    assert_eq "(2 : nat)"
}
