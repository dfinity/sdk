#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    export TEMPORARY_HOME=$(mktemp -d -t dfx-usage-env-home-XXXXXXXX)
    export HOME=$TEMPORARY_HOME
    export CONFIG_ROOT=$(mktemp -d -t dfx-usage-env-config-root-XXXXXXXX)
    export DFX_CONFIG_ROOT=$CONFIG_ROOT
}

teardown() {
    rm -rf $CONFIG_ROOT
    rm -rf TEMPORARY_HOME
}

@test "dfx config root env var stores identity & cache" {
	#identity
    dfx identity new alice
    assert_command head $DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem
    assert_command head $DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem

    assert_command_fail head $HOME/.config/dfx/identity/alice/identity.pem
    assert_command_fail head $HOME/.config/dfx/identity/default/identity.pem

    #cache
    # create a new project to install dfx cache
    assert_command_fail ls $DFX_CONFIG_ROOT/.cache/dfinity/versions
	dfx new hello
    assert_command ls $DFX_CONFIG_ROOT/.cache/dfinity/versions
    assert_command_fail ls $HOME/.cache/dfinity/versions
	rm -rf hello

	# remove configured variable, should use $HOME now
    unset DFX_CONFIG_ROOT

	dfx identity new bob
    assert_command head $HOME/.config/dfx/identity/bob/identity.pem
    assert_command head $HOME/.config/dfx/identity/default/identity.pem

    #cache
    # create a new project to install dfx cache
    assert_command_fail ls $HOME/.cache/dfinity/versions
	dfx new hello
    assert_command ls $HOME/.cache/dfinity/versions
	rm -rf hello
}
