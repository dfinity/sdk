#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
}

@test "dfx config root env var stores identity & cache" {
    #identity
    dfx identity new alice
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

    assert_command_fail head "$HOME/.config/dfx/identity/alice/identity.pem"
    assert_command_fail head "$HOME/.config/dfx/identity/default/identity.pem"

    #cache
    # create a new project to install dfx cache
    assert_command_fail ls "$DFX_CONFIG_ROOT/.cache/dfinity/versions"
    dfx new hello
    assert_command ls "$DFX_CONFIG_ROOT/.cache/dfinity/versions"
    assert_command_fail ls "$HOME/.cache/dfinity/versions"
    rm -rf hello

    (
        # use subshell to retain $DFX_CONFIG_ROOT for teardown
        # remove configured variable, should use $HOME now
        unset DFX_CONFIG_ROOT

        dfx identity new bob
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
