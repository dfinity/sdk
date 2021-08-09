#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export DFX_CONFIG_ROOT="$x"
    cd "$x" || exit
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
    rm -rf "$DFX_CONFIG_ROOT"
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
        assert_command dfx canister install --mode=reinstall e2e_project

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"
        assert_match "Reinstalling code for canister e2e_project"
    )
}

@test "install --mode=reinstall refused if not approved" {
    dfx_start
    dfx deploy

    echo no | (
        assert_command_fail dfx canister install --mode=reinstall e2e_project

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"

        assert_not_match "Installing code for canister"
        assert_match "Refusing to reinstall canister without approval"
    )
}

# dfx deploy --mode=reinstall requires canister name
# dfx deploy --mode=reinstall

@test "deploy --mode=reinstall fails" {
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
        assert_command dfx deploy --mode=reinstall e2e_project

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"
        assert_match "Reinstalling code for canister e2e_project"
    )
}

@test "deploy --mode=reinstall refused if not approved" {
    dfx_start
    dfx deploy

    echo no | (
        assert_command_fail dfx deploy --mode=reinstall e2e_project

        assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER"

        assert_not_match "Installing code for canister"
        assert_match "Refusing to reinstall canister without approval"
    )
}
