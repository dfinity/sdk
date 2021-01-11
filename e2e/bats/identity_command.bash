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
## dfx identity get-principal
##

@test "identity get-principal: different identities have different principal ids" {
    assert_command dfx identity new jose
    assert_command dfx identity new juana

    PRINCPAL_ID_JOSE=$(dfx --identity jose identity get-principal)
    PRINCPAL_ID_JUANA=$(dfx --identity juana identity get-principal)

    if [ "$PRINCPAL_ID_JOSE" -eq "$PRINCPAL_ID_JUANA" ]; then
      echo "IDs should not match: Jose '${PRINCPAL_ID_JOSE}' == Juana '${PRINCPAL_ID_JUANA}'..." | fail
    fi  
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
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
}

@test "identity list: shows the default identity" {
    assert_command dfx identity list
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
    assert_match 'Creating the "default" identity.' "$stderr"
}

##
## dfx identity new
##

@test "identity new: creates a new identity" {
    assert_command dfx identity new alice
    assert_match 'Creating identity: "alice".' "$stderr"
    assert_match 'Created identity: "alice".' "$stderr"
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

@test "identity new: create an HSM-backed identity" {
    assert_command dfx identity new --hsm-pkcs11-lib-path /something/else/somewhere.so --hsm-key-id abcd4321 bob
    assert_command jq -r .hsm.pkcs11_lib_path $HOME/.config/dfx/identity/bob/identity.json
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id $HOME/.config/dfx/identity/bob/identity.json
    assert_eq "abcd4321"
}

@test "identity new: key_id must be hex digits" {
    assert_command_fail dfx identity new --hsm-pkcs11-lib-path xxx --hsm-key-id abcx bob
    assert_match "Key id must contain only hex digits"
}

@test "identity new: key_id must be an even number of digits" {
    assert_command_fail dfx identity new --hsm-pkcs11-lib-path xxx --hsm-key-id fed64 bob
    assert_match "Key id must consist of an even number of hex digits"
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
    assert_match 'Removing identity "alice".' "$stderr"
    assert_match 'Removed identity "alice".' "$stderr"
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

@test "identity remove: can remove an HSM-backed identity" {
    assert_command dfx identity new --hsm-pkcs11-lib-path /something/else/somewhere.so --hsm-key-id abcd4321 bob
    assert_command jq -r .hsm.pkcs11_lib_path $HOME/.config/dfx/identity/bob/identity.json
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id $HOME/.config/dfx/identity/bob/identity.json
    assert_eq "abcd4321"
    assert_command ls $HOME/.config/dfx/identity/bob

    assert_command dfx identity remove bob
    assert_command_fail ls $HOME/.config/dfx/identity/bob
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
    assert_match 'Renaming identity "alice" to "bob".' "$stderr"
    assert_match 'Renamed identity "alice" to "bob".' "$stderr"

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

@test "identity rename: can rename an HSM-backed identity" {
    assert_command dfx identity new --hsm-pkcs11-lib-path /something/else/somewhere.so --hsm-key-id abcd4321 bob
    assert_command dfx identity rename bob alice
    assert_command_fail ls $HOME/.config/dfx/identity/bob

    assert_command jq -r .hsm.pkcs11_lib_path $HOME/.config/dfx/identity/alice/identity.json
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id $HOME/.config/dfx/identity/alice/identity.json
    assert_eq "abcd4321"
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
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
    assert_match 'Creating the "default" identity.' "$stderr"
    assert_match 'Created the "default" identity.' "$stderr"
}

@test "identity whoami: shows the current identity" {
    assert_command dfx identity whoami
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
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
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
    assert_command dfx identity new charlie
    assert_command dfx identity new alice
    assert_command dfx --identity charlie identity whoami
    assert_eq 'charlie'
    assert_command dfx --identity alice identity whoami
    assert_eq 'alice'
}

@test "dfx --identity does not persistently change the selected identity" {
    assert_command dfx identity whoami
    assert_eq 'default' "$(printf "$stdout" | tail -1)"
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

##
## Identity key migration
##
@test "identity manager copies existing key from ~/.dfinity/identity/creds.pem" {
    assert_command dfx identity whoami
    assert_command mkdir -p $TEMPORARY_HOME/.dfinity/identity
    assert_command mv $TEMPORARY_HOME/.config/dfx/identity/default/identity.pem $TEMPORARY_HOME/.dfinity/identity/creds.pem
    ORIGINAL_KEY=$(cat $TEMPORARY_HOME/.dfinity/identity/creds.pem)
    assert_command rmdir $TEMPORARY_HOME/.config/dfx/identity/default
    assert_command rmdir $TEMPORARY_HOME/.config/dfx/identity
    assert_command rm $TEMPORARY_HOME/.config/dfx/identity.json
    assert_command rm $TEMPORARY_HOME/.config/dfx/telemetry/witness.blank
    assert_command rmdir $TEMPORARY_HOME/.config/dfx/telemetry
    assert_command rmdir $TEMPORARY_HOME/.config/dfx
    assert_command rmdir $TEMPORARY_HOME/.config

    assert_command dfx identity whoami

    assert_match "migrating key from"
    assert_eq "$(cat $TEMPORARY_HOME/.config/dfx/identity/default/identity.pem)" "$ORIGINAL_KEY"
}
