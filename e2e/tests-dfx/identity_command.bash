#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    standard_teardown
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
    assert_match 'alice anonymous bob dan default frank'
    assert_command dfx identity new charlie
    assert_command dfx identity list
    assert_match 'alice anonymous bob charlie dan default frank'
}

@test "identity list: shows the anonymous identity" {
    assert_command dfx identity list
    # shellcheck disable=SC2154
    assert_match 'anonymous' "$stdout"
}

@test "identity list: shows the default identity" {
    assert_command dfx identity list
    assert_match 'default' "$stdout"
    # shellcheck disable=SC2154
    assert_match 'Creating the "default" identity.' "$stderr"
}

##
## dfx identity new
##

@test "identity new: creates a new identity" {
    assert_command dfx identity new alice
    assert_match 'Creating identity: "alice".' "$stderr"
    assert_match 'Created identity: "alice".' "$stderr"
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
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
    assert_command jq -r .hsm.pkcs11_lib_path "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.json"
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.json"
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
    assert_command_fail head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_command dfx identity new alice

    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_match "BEGIN PRIVATE KEY"
    assert_command dfx identity list
    assert_match 'alice anonymous default'

    assert_command dfx identity remove alice
    assert_match 'Removing identity "alice".' "$stderr"
    assert_match 'Removed identity "alice".' "$stderr"
    assert_command_fail cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"

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

    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_match "BEGIN PRIVATE KEY"
    assert_command dfx identity list
    assert_match 'alice anonymous default'
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
    assert_command jq -r .hsm.pkcs11_lib_path "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.json"
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.json"
    assert_eq "abcd4321"
    assert_command ls "$DFX_CONFIG_ROOT/.config/dfx/identity/bob"

    assert_command dfx identity remove bob
    assert_command_fail ls "$DFX_CONFIG_ROOT/.config/dfx/identity/bob"
}

##
## dfx identity rename
##

@test "identity rename: can rename an identity" {
    assert_command dfx identity new alice
    assert_command dfx identity list
    assert_match 'alice anonymous default'
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_match "BEGIN PRIVATE KEY"
    x=$(cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem")
    local key="$x"

    assert_command dfx identity rename alice bob
    assert_match 'Renaming identity "alice" to "bob".' "$stderr"
    assert_match 'Renamed identity "alice" to "bob".' "$stderr"

    assert_command dfx identity list
    assert_match 'anonymous bob default'
    assert_command cat "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
    assert_eq "$key" "$(cat "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem")"
    assert_match "BEGIN PRIVATE KEY"
    assert_command_fail cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
}

@test "identity rename: can rename the default identity, which also changes the default" {
    assert_command dfx identity list
    assert_match 'default'
    assert_command dfx identity rename default bob
    assert_command dfx identity list
    assert_match 'bob'
    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
    assert_match "BEGIN PRIVATE KEY"

    assert_command dfx identity whoami
    assert_eq 'bob'
}

@test "identity rename: can rename the selected identity, which also changes the default" {
    assert_command dfx identity new alice
    assert_command dfx identity use alice
    assert_command dfx identity list
    assert_match 'alice anonymous default'
    assert_command dfx identity rename alice charlie

    assert_command dfx identity list
    assert_match 'anonymous charlie default'

    assert_command dfx identity whoami
    assert_eq 'charlie'

    assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/charlie/identity.pem"
    assert_match "BEGIN PRIVATE KEY"
    assert_command_fail cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
}

@test "identity rename: cannot create an anonymous identity via rename" {
  assert_command dfx identity new alice
    assert_command_fail dfx identity rename alice anonymous
    assert_match "Cannot create an anonymous identity"
}

@test "identity rename: can rename an HSM-backed identity" {
    skip "Need to instantiate identity when renaming so skipping until we have an hsm mock"
    assert_command dfx identity new --hsm-pkcs11-lib-path /something/else/somewhere.so --hsm-key-id abcd4321 bob
    assert_command dfx identity rename bob alice
    assert_command_fail ls "$DFX_CONFIG_ROOT/.config/dfx/identity/bob"

    assert_command jq -r .hsm.pkcs11_lib_path "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.json"
    assert_eq "/something/else/somewhere.so"
    assert_command jq -r .hsm.key_id "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.json"
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

@test "identity use: can switch to the anonymous identity" {
    assert_command dfx identity use anonymous
    assert_command dfx identity whoami
    assert_eq 'anonymous'
    assert_command dfx identity get-principal
    assert_eq '2vxsx-fae'
}

##
## dfx identity whoami
##

@test "identity whoami: creates the default identity on first run" {
    # Just an example.  All the identity commands do this.
    assert_command dfx identity whoami
    assert_eq 'default' "$stdout"
    assert_match 'Creating the "default" identity.' "$stderr"
    assert_match 'Created the "default" identity.' "$stderr"
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

##
## Identity key migration
##
@test "identity manager copies existing key from $DFX_CONFIG_ROOT/.dfinity/identity/creds.pem" {
    assert_command dfx identity whoami
    assert_command mkdir -p "$DFX_CONFIG_ROOT/.dfinity/identity"
    assert_command mv "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem" "$DFX_CONFIG_ROOT/.dfinity/identity/creds.pem"
    ORIGINAL_KEY=$(cat "$DFX_CONFIG_ROOT/.dfinity/identity/creds.pem")
    assert_command rmdir "$DFX_CONFIG_ROOT/.config/dfx/identity/default"
    assert_command rmdir "$DFX_CONFIG_ROOT/.config/dfx/identity"
    assert_command rm "$DFX_CONFIG_ROOT/.config/dfx/identity.json"
    assert_command rmdir "$DFX_CONFIG_ROOT/.config/dfx"
    assert_command rmdir "$DFX_CONFIG_ROOT/.config"

    assert_command dfx identity whoami

    assert_match "migrating key from"
    assert_eq "$(cat "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem)" "$ORIGINAL_KEY"
}

@test "identity: import" {
    openssl ecparam -name secp256k1 -genkey -out identity.pem
    assert_command dfx identity import alice identity.pem
    assert_match 'Creating identity: "alice".' "$stderr"
    assert_match 'Created identity: "alice".' "$stderr"
    assert_command diff identity.pem "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_eq ""
}

@test "identity: import default" {
    assert_command dfx identity new alice
    assert_command dfx identity import bob "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
    assert_match 'Creating identity: "bob".' "$stderr"
    assert_match 'Created identity: "bob".' "$stderr"
    assert_command diff "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
    assert_eq ""
}

@test "identity: cannot import invalid PEM file" {
    assert_command dfx identity new alice
    assert_command cp "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" ./alice.pem
    # Following 3 lines manipulate the pem file so that it will be invalid
    head -n 1 alice.pem > bob.pem
    echo -n 1 >> bob.pem
    tail -n 3 alice.pem > bob.pem
    assert_command_fail dfx identity import bob bob.pem
    assert_match 'Creating identity: "bob".' "$stderr"
    assert_match 'Invalid Ed25519 private key in PEM file at' "$stderr"
}
