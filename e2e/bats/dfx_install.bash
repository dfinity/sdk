#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    # Each test gets a fresh cache directory
    mkdir -p $(pwd)/cache-roots
    export HOME=$(mktemp -d $(pwd)/cache-roots/cache-XXXXXXXX)
}

teardown() {
    dfx_stop
}

@test "dfx cache show does not install the dfx version into the cache" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test -z "$(ls -A $HOME)"

    assert_command dfx cache show

    # does not populate the cache with this version
    test ! -e $(dfx cache show)

    # it does create the empty versions directory though
    test -d $HOME/.cache/dfinity/versions
    test -z "$(ls -A $HOME/.cache/dfinity/versions)"
}

@test "non-forced install populates an empty cache" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test ! -e $(dfx cache show)/dfx

    dfx_new

    test -f $(dfx cache show)/dfx
}

@test "forced install populates an empty cache" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    test ! -e $(dfx cache show)/dfx

    assert_command dfx cache install

    test -f $(dfx cache show)/dfx
}

@test "forced install over an install succeeds" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    assert_command dfx cache install
    test -f $(dfx cache show)/dfx

    assert_command dfx cache install
}

@test "forced install overwrites a cached version" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    assert_command dfx cache install
    test -f $(dfx cache show)/dfx

    # add something extra to the cache
    echo "garbage" >$(dfx cache show)/garbage
    test -f $(dfx cache show)/garbage

    assert_command dfx cache install

    # dfx cache install should have removed it
    test ! -e $(dfx cache show)/garbage

    # and also installed the cache itself
    test -f $(dfx cache show)/dfx
}
