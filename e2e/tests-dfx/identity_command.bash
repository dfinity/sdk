#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
  export DFX_CI_MOCK_KEYRING_LOCATION="$MOCK_KEYRING_LOCATION"
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

  PRINCIPAL_ID_JOSE=$(dfx identity get-principal --identity jose)
  PRINCIPAL_ID_JUANA=$(dfx identity get-principal --identity juana)

  if [ "$PRINCIPAL_ID_JOSE" -eq "$PRINCIPAL_ID_JUANA" ]; then
    echo "IDs should not match: Jose '${PRINCIPAL_ID_JOSE}' == Juana '${PRINCIPAL_ID_JUANA}'..." | fail
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
  assert_match \
'alice
anonymous
bob
dan
default
frank'
  assert_command dfx identity new charlie
  assert_command dfx identity list
  assert_match \
'alice
anonymous
bob
charlie
dan
default
frank'
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

@test "identity new: creates a new keyring identity" {
  assert_command dfx identity new alice
  assert_match 'Created identity: "alice".' "$stderr"
  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_match "internet_computer_identity_alice"

  # does not change the default identity
  assert_command dfx identity whoami
  assert_eq 'default'
}

@test "identity new --storage-mode plaintext: creates a new identity" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_match 'Created identity: "alice".' "$stderr"
  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"

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

@test "identity new: --force re-creates an identity" {
  assert_command dfx identity new alice
  dfx identity use alice
  PRINCIPAL_1="$(dfx identity get-principal)"
  assert_command dfx identity new --force alice
  PRINCIPAL_2="$(dfx identity get-principal)"
  assert_neq "$PRINCIPAL_1" "$PRINCIPAL_2"
}

@test "identity new: --force does not switch to created identity" {
  # Was a bug: https://dfinity.atlassian.net/browse/SDK-911
  assert_command dfx identity new --force alice
  PRINCIPAL_ORIGINAL="$(dfx identity get-principal)"
  assert_command dfx identity use alice
  PRINCIPAL_ALICE="$(dfx identity get-principal)"
  assert_neq "$PRINCIPAL_ORIGINAL" "$PRINCIPAL_ALICE"
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

@test "identity new: key is compatible with openssl" {
  assert_command dfx identity new --storage-mode plaintext bob
  assert_command openssl ec -in "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
}

##
## dfx identity remove
##

@test "identity remove: can remove an identity that exists" {
  assert_command_fail cat "$MOCK_KEYRING_LOCATION"
  assert_command dfx identity new alice

  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_match "internet_computer_identity_alice"
  assert_command dfx identity list
  assert_match \
'alice
anonymous
default'

  assert_command dfx identity remove alice
  assert_match 'Removed identity "alice".' "$stderr"
  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_not_match "internet_computer_identity_alice"

  assert_command dfx identity list
  assert_match 'default'
}

@test "identity remove --storage-mode plaintext: can remove an identity that exists" {
  assert_command_fail head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_command dfx identity new alice --storage-mode plaintext

  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"
  assert_command dfx identity list
  assert_match \
'alice
anonymous
default'

  assert_command dfx identity remove alice
  assert_match 'Removed identity "alice".' "$stderr"
  assert_command_fail cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"

  assert_command dfx identity list
  assert_match 'default'
}

@test "identity remove: reports an error if no such identity" {
  assert_command_fail dfx identity remove charlie
}

@test "identity remove: only remove identities with configured wallet if --drop-wallets is specified" {
  # There's no replica running, and no real wallet.  This is just a valid principal.
  WALLET="rwlgt-iiaaa-aaaaa-aaaaa-cai"
  assert_command dfx identity new alice
  assert_command dfx identity use alice
  assert_command dfx identity set-wallet --force "$WALLET" --network ic
  assert_command dfx identity use default
  assert_command_fail dfx identity remove alice
  # make sure the configured wallet is displayed
  assert_match "identity 'alice' on network 'ic' has wallet $WALLET"
  assert_command dfx identity remove alice --drop-wallets
  assert_match "identity 'alice' on network 'ic' has wallet $WALLET"
}

@test "identity remove: cannot remove the non-default active identity" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity use alice
  assert_command_fail dfx identity remove alice

  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"
  assert_command dfx identity list
  assert_match \
'alice
anonymous
default'
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
  assert_match \
'alice
anonymous
default'
  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_match "internet_computer_identity_alice"
  KEY="$(dfx identity export alice)"

  assert_command dfx identity rename alice bob
  assert_match 'Renamed identity "alice" to "bob".' "$stderr"

  assert_command dfx identity list
  assert_match \
'anonymous
bob
default'
  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_match "internet_computer_identity_bob"
  assert_eq "$KEY" "$(dfx identity export bob)"
  assert_command cat "$MOCK_KEYRING_LOCATION"
  assert_not_match "internet_computer_identity_alice"
}

@test "identity rename --storage-mode plaintext: can rename an identity" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity list
  assert_match \
'alice
anonymous
default'
  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"
  x=$(cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem")
  KEY="$x"

  assert_command dfx identity rename alice bob
  assert_match 'Renamed identity "alice" to "bob".' "$stderr"

  assert_command dfx identity list
  assert_match \
'anonymous
bob
default'
  assert_command cat "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
  assert_eq "$KEY" "$(cat "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem")"
  assert_match "BEGIN EC PRIVATE KEY"
  assert_command_fail cat "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
}

@test "identity rename: can rename the default identity, which also changes the default" {
  assert_command dfx identity list
  assert_match 'default'
  assert_command dfx identity rename default bob
  assert_command dfx identity list
  assert_match 'bob'
  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"

  assert_command dfx identity whoami
  assert_eq 'bob'
}

@test "identity rename: can rename the selected identity, which also changes the default" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity use alice
  assert_command dfx identity list
  assert_match \
'alice
anonymous
default'
  assert_command dfx identity rename alice charlie

  assert_command dfx identity list
  assert_match \
'anonymous
charlie
default'

  assert_command dfx identity whoami
  assert_eq 'charlie'

  assert_command head "$DFX_CONFIG_ROOT/.config/dfx/identity/charlie/identity.pem"
  assert_match "BEGIN EC PRIVATE KEY"
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

@test "identity use: is overridden by env var DFX_IDENTITY" {
  assert_command dfx identity new dan
  assert_command dfx identity new frank
  assert_command dfx identity new alice
  assert_command dfx identity use dan
  assert_command dfx identity whoami
  assert_eq 'dan'
  DFX_IDENTITY=frank
  export DFX_IDENTITY
  assert_command dfx identity whoami
  assert_eq 'frank'
  assert_command dfx identity whoami --identity alice
  assert_eq 'alice'
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

## dfx (+other commands) --identity

@test "dfx identity whoami --identity (name): shows the overriding identity" {
  assert_command dfx identity whoami
  assert_eq 'default' "$stdout"
  assert_command dfx identity new charlie
  assert_command dfx identity new alice
  assert_command dfx identity whoami --identity charlie
  assert_eq 'charlie'
  assert_command dfx identity whoami --identity alice
  assert_eq 'alice'
}

@test "dfx (command) --identity does not persistently change the selected identity" {
  assert_command dfx identity whoami
  assert_eq 'default' "$stdout"
  assert_command dfx identity new charlie
  assert_command dfx identity new alice
  assert_command dfx identity use charlie
  assert_command dfx identity whoami
  assert_eq 'charlie'
  assert_command dfx identity whoami --identity alice
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
  assert_match 'Imported identity: "alice".' "$stderr"
  assert_command bash -c "dfx identity export alice > alice.pem"
  assert_command diff identity.pem alice.pem
  assert_eq ""
}

@test "identity: import can only overwrite identity with --force" {
  openssl ecparam -name secp256k1 -genkey -out identity.pem
  openssl ecparam -name secp256k1 -genkey -out identity2.pem
  assert_command dfx identity import alice identity.pem
  assert_match 'Imported identity: "alice".' "$stderr"
  dfx identity use alice
  PRINCIPAL_1="$(dfx identity get-principal)"

  assert_command_fail dfx identity import alice identity2.pem
  assert_match "Identity already exists."
  assert_command dfx identity import --force alice identity2.pem
  assert_match 'Imported identity: "alice".'
  PRINCIPAL_2="$(dfx identity get-principal)"

  assert_neq "$PRINCIPAL_1" "$PRINCIPAL_2"
}

@test "identity: import default" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity import bob "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem"
  assert_match 'Imported identity: "bob".' "$stderr"
  assert_command bash -c "dfx identity export bob > bob.pem"
  assert_command diff "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" "bob.pem"
  assert_eq ""
}

@test "identity: import --storage-mode plaintext" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity import bob "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" --storage-mode plaintext
  assert_match 'Imported identity: "bob".' "$stderr"
  assert_command diff "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" "$DFX_CONFIG_ROOT/.config/dfx/identity/bob/identity.pem"
  assert_eq ""
}

@test "identity: cannot import invalid PEM file" {
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command cp "$DFX_CONFIG_ROOT/.config/dfx/identity/alice/identity.pem" ./alice.pem
  # Following 3 lines manipulate the pem file so that it will be invalid
  head -n 1 alice.pem > bob.pem
  echo -n 1 >> bob.pem
  tail -n 3 alice.pem > bob.pem
  assert_command_fail dfx identity import bob bob.pem
  assert_match 'Failed to validate PEM content' "$stderr"
}

@test "identity: can import an EC key without an EC PARAMETERS section (as quill generate makes)" {
  cat >private-key-no-ec-parameters.pem <<EOF
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEIE+3ipe2ruuJOmeBAhImUP/jic7Qwk2fXC8BaAmu6VK4oAcGBSuBBAAK
oUQDQgAEBQKn0CLyiA/fQf6L8S07/MDJ9kIJTzZvm2jFo2/yvSToGee+XzP/GCE4
08ZcZFM1EwUsknDBoSd0EF1PzFRmJg==
-----END EC PRIVATE KEY-----
EOF
  assert_command dfx identity import private-key-no-ec-parameters private-key-no-ec-parameters.pem
  assert_command dfx identity get-principal --identity private-key-no-ec-parameters
  assert_eq "j4p4p-o5ogq-4gzev-t3kay-hpm5o-xuwpz-yvrpp-47cc4-qyunt-k76yw-qae"
  echo "{}" >dfx.json # avoid "dfx.json not found, using default."
  assert_command dfx ledger account-id --identity private-key-no-ec-parameters
  assert_eq "3c00cf85d77b9dbf74a2acec1d9a9e73a3fc65f5048c64800b15f3b2c4c8eb11"
}

@test "identity: can export and re-import an identity" {
  assert_command dfx identity new alice
  dfx identity export alice > export.pem
  assert_file_exists export.pem
  assert_command dfx identity import bob export.pem
}

@test "identity: can import a seed phrase" {
  reg="seed phrase for identity 'alice': ([a-z ]+)"
  assert_command dfx identity new alice
  [[ $stderr =~ $reg ]]
  echo "${BASH_REMATCH[1]}" >seed.txt
  principal=$(dfx identity get-principal --identity alice)
  assert_command dfx identity import alice2 --seed-file seed.txt --storage-mode plaintext
  assert_command dfx identity get-principal --identity alice2
  assert_eq "$principal"
  dfx identity export alice2 > export.pem
  assert_command openssl asn1parse -in export.pem
  assert_match ':secp256k1'
}

@test "identity: consistently imports a known seed phrase" {
  echo "display dawn estate night naive stomach receive lock expose boring square boy deposit mistake volume soldier coil rocket match diamond repair opinion action paddle">seed.txt
  assert_command dfx identity import alice --seed-file seed.txt --storage-mode plaintext
  assert_command dfx identity get-principal --identity alice
  assert_eq "qimd7-lqrvx-kdvsm-7zeqn-bgoix-ukjfi-hgmfg-ur2he-odgb2-joms4-nae"
}
