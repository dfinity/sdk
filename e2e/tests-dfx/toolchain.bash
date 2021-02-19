#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    export RUST_BACKTRACE=1

    # Each test gets a fresh cache directory
    mkdir -p "$(pwd)"/cache-roots
    x=$(mktemp -d "$(pwd)"/cache-roots/cache-XXXXXXXX)
    export HOME="$x"
}

teardown() {
    dfx_stop
}

@test "dfx toolchain : 0.6.23" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test -z "$(ls -A "$HOME")"

    assert_command dfx toolchain install 0.6.23
    test -d $HOME/.cache/dfinity/versions
    test -d $HOME/.dfinity/toolchains
    test -L $HOME/.dfinity/toolchains/0.6.23
    assert_command dfx toolchain install 0.6.23

    assert_command dfx toolchain list
    assert_eq 0.6.23

    assert_command dfx toolchain default 0.6.23
    test -L $HOME/.dfinity/default
    test -e $HOME/.dfinity/default

    assert_command dfx toolchain uninstall 0.6.23
    test ! -e $HOME/.dfinity/toolchains/0.6.23
}

@test "dfx toolchain : 0.6" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test -z "$(ls -A "$HOME")"

    assert_command dfx toolchain install 0.6
    test -d $HOME/.cache/dfinity/versions
    test -d $HOME/.dfinity/toolchains
    test -L $HOME/.dfinity/toolchains/0.6
    assert_command dfx toolchain install 0.6

    assert_command dfx toolchain list
    assert_eq 0.6

    assert_command dfx toolchain default 0.6
    test -L $HOME/.dfinity/default
    test -e $HOME/.dfinity/default

    assert_command dfx toolchain uninstall 0.6
    test ! -e $HOME/.dfinity/toolchains/0.6
}

@test "dfx toolchain : latest" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test -z "$(ls -A "$HOME")"

    assert_command dfx toolchain install latest
    test -d $HOME/.cache/dfinity/versions
    test -d $HOME/.dfinity/toolchains
    test -L $HOME/.dfinity/toolchains/latest
    assert_command dfx toolchain install latest

    assert_command dfx toolchain list
    assert_eq latest

    assert_command dfx toolchain default latest
    test -L $HOME/.dfinity/default
    test -e $HOME/.dfinity/default

    assert_command dfx toolchain uninstall latest
    test ! -e $HOME/.dfinity/toolchains/latest
}