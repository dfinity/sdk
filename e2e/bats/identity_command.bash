#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    export TEMPORARY_HOME=$(mktemp -d -t dfx-identity-home-XXXXXXXX)
    export HOME=$TEMPORARY_HOME
}

teardown() {
    rm -rf $TEMPORARY_HOME
}

##
## dfx identity list
##

@test "identity list: shows identities in alpha order" {
    assert_command dfx identity new dan
    assert_command dfx identity new frank
    assert_command dfx identity new alice
    assert_command dfx identity new bob
    assert_command dfx identity list
    assert_match 'alice bob dan default frank'
    assert_command dfx identity new charlie
    assert_command dfx identity list
    assert_match 'alice bob charlie dan default frank'
}

@test "identity list: shows the anonymous identity" {
    assert_command dfx identity list
    # this should include anonymous, but we do not yet have support.
    assert_eq 'default' "$stdout"
}

@test "identity list: shows the default identity" {
    assert_command dfx identity list
    assert_match 'default' "$stdout"
    assert_match 'Created the "default" identity.' "$stderr"
}

##
## dfx identity new
##

@test "identity new: creates a new identity" {
    assert_command dfx identity new alice
    assert_command head $HOME/.config/dfx/identity/alice/identity.pem
    assert_match "BEGIN PRIVATE KEY"

    # does not change the default identity
    assert_command dfx identity whoami
    assert_eq 'default'
}

@test "identity new: cannot create an identity called anonymous" {
    assert_command_fail dfx identity new anonymous
}

@test "identity new: cannot create an identity that already exists" {
    assert_command dfx identity new bob
    assert_command_fail dfx identity new bob
    assert_match "Identity already exists"
}

##
## dfx identity remove
##

@test "identity remove: can remove an identity that exists" {
    assert_command_fail head $HOME/.config/dfx/identity/alice/identity.pem
    assert_command dfx identity new alice

    assert_command head $HOME/.config/dfx/identity/alice/identity.pem
    assert_match "BEGIN PRIVATE KEY"
    assert_command dfx identity list
    assert_match 'alice default'

    assert_command dfx identity remove alice
    assert_command_fail cat $HOME/.config/dfx/identity/alice/identity.pem

    assert_command dfx identity list
    assert_match 'default'
}

@test "identity remove: reports an error if no such identity" {
    assert_command_fail dfx identity remove charlie
}

@test "identity remove: cannot remove the non-default active identity" {
    assert_command dfx identity new alice
    assert_command dfx identity use alice
    assert_command_fail dfx identity remove alice

    assert_command head $HOME/.config/dfx/identity/alice/identity.pem
    assert_match "BEGIN PRIVATE KEY"
    assert_command dfx identity list
    assert_match 'alice default'
}

@test "identity remove: cannot remove the default identity" {
    # a new one will just get created again
    assert_command_fail dfx identity remove default
    assert_match "Cannot delete the default identity"
}


@test "identity remove: cannot remove the anonymous identity" {
    assert_command_fail dfx identity remove anonymous
}


##
## dfx identity rename
##

@test "identity rename: can rename an identity" {
    assert_command dfx identity new alice
    assert_command dfx identity list
    assert_match 'alice default'
    assert_command head $HOME/.config/dfx/identity/alice/identity.pem
    assert_match "BEGIN PRIVATE KEY"
    local key=$(cat $HOME/.config/dfx/identity/alice/identity.pem)

    assert_command dfx identity rename alice bob

    assert_command dfx identity list
    assert_match 'bob default'
    assert_command cat $HOME/.config/dfx/identity/bob/identity.pem
    assert_eq "$key" "$(cat $HOME/.config/dfx/identity/bob/identity.pem)"
    assert_match "BEGIN PRIVATE KEY"
    assert_command_fail cat $HOME/.config/dfx/identity/alice/identity.pem
}

@test "identity rename: can rename the default identity, which also changes the default" {
    assert_command dfx identity list
    assert_match 'default'
    assert_command dfx identity rename default bob
    assert_command dfx identity list
    assert_match 'bob'
    assert_command head $HOME/.config/dfx/identity/bob/identity.pem
    assert_match "BEGIN PRIVATE KEY"

    assert_command dfx identity whoami
    assert_eq 'bob'
}

@test "identity rename: can rename the selected identity, which also changes the default" {
    assert_command dfx identity new alice
    assert_command dfx identity use alice
    assert_command dfx identity list
    assert_match 'alice default'
    assert_command dfx identity rename alice charlie

    assert_command dfx identity list
    assert_match 'charlie default'

    assert_command dfx identity whoami
    assert_eq 'charlie'

    assert_command head $HOME/.config/dfx/identity/charlie/identity.pem
    assert_match "BEGIN PRIVATE KEY"
    assert_command_fail cat $HOME/.config/dfx/identity/alice/identity.pem
}

@test "identity rename: cannot create an anonymous identity via rename" {
  assert_command dfx identity new alice
    assert_command_fail dfx identity rename alice anonymous
    assert_match "Cannot create an anonymous identity"
}

##
## dfx identity use
##

@test "identity use: switches to an existing identity" {
    assert_command dfx identity new alice
    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx identity use alice
    assert_command dfx identity whoami
    assert_eq 'alice'

    ## and back
    assert_command dfx identity use default
    assert_command dfx identity whoami
    assert_eq 'default'
}

@test "identity use: cannot use an identity that has not been created yet" {
    assert_command_fail dfx identity use alice
    assert_command dfx identity whoami
    assert_eq 'default'
}

@test "identity use: cannot switch to the anonymous identity" {
    # this should actually succeed, but we do not yet have support for
    # the anonymous identity.
    assert_command_fail dfx identity use anonymous
}

##
## dfx identity whoami
##

@test "identity whoami: creates the default identity on first run" {
    # Just an example.  All the identity commands do this.
    assert_command dfx identity whoami
    assert_eq 'default' "$stdout"
    assert_eq 'Created the "default" identity.' "$stderr"
}

@test "identity whoami: shows the current identity" {
    assert_command dfx identity whoami
    assert_eq 'default' "$stdout"
    assert_command dfx identity new charlie
    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx identity use charlie
    assert_command dfx identity whoami
    assert_eq 'charlie'
}

## dfx --identity (+other commands)

@test "dfx --identity (name) identity whoami: shows the overriding identity" {
    assert_command dfx identity whoami
    assert_eq 'default' "$stdout"
    assert_command dfx identity new charlie
    assert_command dfx identity new alice
    assert_command dfx --identity charlie identity whoami
    assert_eq 'charlie'
    assert_command dfx --identity alice identity whoami
    assert_eq 'alice'
}

@test "dfx --identity does not persistently change the selected identity" {
    assert_command dfx identity whoami
    assert_eq 'default' "$stdout"
    assert_command dfx identity new charlie
    assert_command dfx identity new alice
    assert_command dfx identity use charlie
    assert_command dfx identity whoami
    assert_eq 'charlie'
    assert_command dfx --identity alice identity whoami
    assert_eq 'alice'
    assert_command dfx identity whoami
    assert_eq 'charlie'
}
