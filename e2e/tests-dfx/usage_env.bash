#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
}

@test "dfx config root env var stores identity & cache" {
    use_test_specific_cache_root   # Because this test depends on a clean cache state

    #identity
    dfx identity new --disable-encryption alice
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

    assert_command_fail head "$HOME/.config/dfx/identity/alice/identity.pem"
    assert_command_fail head "$HOME/.config/dfx/identity/default/identity.pem"

    #cache
    # create a new project to install dfx cache
    assert_command_fail ls "$DFX_CACHE_ROOT/.cache/dfinity/versions"
    dfx new hello
    assert_command ls "$DFX_CACHE_ROOT/.cache/dfinity/versions"
    assert_command_fail ls "$HOME/.cache/dfinity/versions"
    rm -rf hello

    (
        # use subshell to retain $DFX_CONFIG_ROOT for teardown
        # remove configured variable, should use $HOME now
        unset DFX_CACHE_ROOT
        unset DFX_CONFIG_ROOT

        dfx identity new --disable-encryption bob
        assert_command head "$HOME/.config/dfx/identity/bob/identity.pem"
        assert_command head "$HOME/.config/dfx/identity/default/identity.pem"

        #cache
        # create a new project to install dfx cache
        assert_command_fail ls "$HOME/.cache/dfinity/versions"
        dfx new hello
        assert_command ls "$HOME/.cache/dfinity/versions"
        rm -rf hello
    )
}
