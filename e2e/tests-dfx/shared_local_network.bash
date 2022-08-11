#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx start creates no files in the current directory when run from an empty directory" {
    dfx_start
    assert_command find .
    assert_eq "."
}

@test "dfx start creates only an empty .dfx directory when run from a project" {
    dfx_new hello
    START_DIR_CONTENTS="$(find .)"
    dfx_start
    END_DIR_CONTENTS="$(find . | grep -v '^\./\.dfx$')"
    assert_eq "$START_DIR_CONTENTS" "$END_DIR_CONTENTS"
}

@test "project data is cleared after dfx start --clean from outside the project" {
    [ "$USE_IC_REF" ] && skip "start_dfx does not support parameters with emulator"

    mkdir somewhere
    (
        cd somewhere
        dfx_start
    )

    (
        dfx_new hello
        dfx deploy
        dfx canister id hello_backend
    )

    mkdir somewhere_else
    (
        cd somewhere_else
        dfx_stop
        dfx_start --clean
    )

    (
        cd hello
        assert_command_fail dfx canister id hello_backend
    )
}

@test "multiple projects with the same canister names" {
    dfx_start

    mkdir a
    cd a
    dfx_new hello
    install_asset counter
    dfx deploy
    HELLO_BACKEND_A="$(dfx canister id hello_backend)"
    dfx canister call hello_backend inc
    dfx canister call hello_backend inc
    cd ../..

    mkdir b
    cd b
    dfx_new hello
    install_asset counter
    dfx deploy
    HELLO_BACKEND_B="$(dfx canister id hello_backend)"
    dfx canister call hello_backend write '(6: nat)'
    cd ../..

    assert_command dfx canister call "$HELLO_BACKEND_A" read
    assert_eq "(2 : nat)"
    (
        cd a/hello
        assert_command dfx canister call hello_backend read
        assert_eq "(2 : nat)"
    )

    assert_command dfx canister call "$HELLO_BACKEND_B" read
    assert_eq "(6 : nat)"
    (
        cd b/hello
        assert_command dfx canister call hello_backend read
        assert_eq "(6 : nat)"
    )
}


@test "wallet config file is reset after start --clean" {
    [ "$USE_IC_REF" ] && skip "start_dfx does not support parameters with emulator"

    dfx_start

    (
        dfx_new hello
        dfx wallet balance
        dfx identity get-wallet
        assert_command dfx diagnose
        assert_eq "No problems found"
    )

    dfx_stop
    dfx_start --clean

    (
        cd hello
        assert_command_fail dfx diagnose
        assert_match "No wallet found; nothing to do"
    )
}

@test "separate projects use the same wallet id for a given identity" {
    dfx_start

    ( dfx_new a )
    ( dfx_new b )
    WALLET_ID_A="$(cd a ; dfx identity get-wallet)"
    WALLET_ID_B="$(cd b ; dfx identity get-wallet)"

    assert_eq "$WALLET_ID_A" "$WALLET_ID_B"
}

@test "dfx identity rename renames wallet for shared local network" {
     dfx_start

     dfx identity new  alice --disable-encryption
     ALICE_WALLET="$(dfx identity get-wallet --identity alice)"

     dfx identity rename alice bob
     BOB_WALLET="$(dfx identity get-wallet --identity bob)"

     assert_eq "$ALICE_WALLET" "$BOB_WALLET"
}
