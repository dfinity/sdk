#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new_assets
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "identity new: name validation" {
  assert_command_fail dfx identity new iden%tity --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command_fail dfx identity new 'iden tity' --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command_fail dfx identity new "iden\$tity" --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command_fail dfx identity new iden\\tity --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command_fail dfx identity new 'iden\ttity' --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command_fail dfx identity new iden/tity --storage-mode plaintext
  assert_match "Invalid identity name"

  assert_command dfx identity new i_den.ti-ty --storage-mode plaintext

  assert_command dfx identity new i_den@ti-ty --storage-mode plaintext
}

@test "identity get-principal: the get-principal is the same as sender id" {
  install_asset identity
  dfx_start
  assert_command dfx identity new --storage-mode plaintext jose

  PRINCIPAL_ID=$(dfx identity get-principal --identity jose)

  dfx canister create e2e_project_backend --identity jose
  dfx build e2e_project_backend --identity jose
  dfx canister install e2e_project_backend --identity jose

  assert_command dfx canister call e2e_project_backend amInitializer --identity jose

  SENDER_ID=$(dfx canister call e2e_project_backend fromCall --identity jose)

  if [ "$PRINCIPAL_ID" -ne "$SENDER_ID" ]; then
    echo "IDs did not match: Principal '${PRINCIPAL_ID}' != Sender '${SENDER_ID}'..." | fail
  fi
}

@test "identity get-principal (anonymous): the get-principal is the same as sender id" {
  install_asset identity
  dfx_start
  assert_command dfx identity new --storage-mode plaintext jose

  ANONYMOUS_PRINCIPAL_ID="2vxsx-fae"

  PRINCIPAL_ID=$(dfx identity get-principal --identity anonymous)

  if [ "$PRINCIPAL_ID" -ne "$ANONYMOUS_PRINCIPAL_ID" ]; then
    echo "IDs did not match: Principal '${ANONYMOUS_PRINCIPAL_ID}' != Sender '${PRINCIPAL_ID}'..." | fail
  fi

  dfx canister create e2e_project_backend --identity jose
  dfx build e2e_project_backend --identity jose
  dfx canister install e2e_project_backend --identity jose

  SENDER_ID=$(dfx canister call e2e_project_backend fromCall --identity anonymous)

  if [ "$ANONYMOUS_PRINCIPAL_ID" -ne "$SENDER_ID" ]; then
    echo "IDs did not match: Principal '${ANONYMOUS_PRINCIPAL_ID}' != Sender '${SENDER_ID}'..." | fail
  fi
}

@test "calls and query receive the same principal from dfx" {
  install_asset identity
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_command dfx canister install --all

  ID_CALL=$(dfx canister call e2e_project_backend fromCall)
  ID_QUERY=$(dfx canister call e2e_project_backend fromQuery)
  if [ "$ID_CALL" -ne "$ID_QUERY" ]; then
    echo "IDs did not match: call '${ID_CALL}' != query '${ID_QUERY}'..." | fail
  fi

  ID=$(dfx canister call e2e_project_backend getCanisterId)
  assert_command dfx canister call e2e_project_backend isMyself "$ID"
  assert_eq '(true)'
  assert_command dfx canister call e2e_project_backend isMyself "$ID_CALL"
  assert_eq '(false)'
}

@test "dfx ping does not create a default identity" {
  dfx_start

  assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity.json"
  assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

  assert_command dfx ping

  assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity.json"
  assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

  # shellcheck disable=SC2154
  assert_not_match 'Creating' "$stderr"
  # shellcheck disable=SC2154
  assert_not_match '(default.*identity|identity.*default)' "$stderr"
  # shellcheck disable=SC2154
  assert_match "ic_api_version" "$stdout"
}

@test "dfx canister: creates the default identity on first run" {
  install_asset identity
  dfx_start
  assert_command dfx canister create e2e_project_backend
  assert_match 'Creating the "default" identity.' "$stderr"
}

@test "after using a specific identity while creating a canister, that user is the initializer" {
  install_asset identity
  dfx_start
  assert_command dfx identity new --storage-mode plaintext alice
  assert_command dfx identity new --storage-mode plaintext bob

  dfx canister create --all --identity alice
  assert_command dfx build --identity alice
  assert_command dfx canister install --all --identity alice

  # The user Identity's principal is the initializer
  assert_command dfx canister call e2e_project_backend amInitializer --identity alice
  assert_eq '(true)'

  assert_command dfx canister call e2e_project_backend amInitializer --identity bob
  assert_eq '(false)'

  # these all fail (other identities are not initializer; cannot store assets):
  assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob"XWV"})' --identity bob
  assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob"XWV"})' --identity default
  assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob"XWV"})'
  assert_command_fail dfx canister call e2e_project_frontend retrieve '("B")'

  # but alice, the initializer, can store assets:
  assert_command dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob"XWV"})' --identity alice
  assert_eq '()'
  assert_command dfx canister call --output idl e2e_project_frontend retrieve '("B")'
  # shellcheck disable=SC2154
  assert_eq '(blob "XWV")' "$stdout"
}

@test "after renaming an identity, the renamed identity is still initializer" {
  install_asset identity
  dfx_start
  assert_command dfx identity new --storage-mode plaintext alice

  dfx canister create --all --identity alice
  assert_command dfx build --identity alice
  assert_command dfx canister install --all --identity alice
  assert_command dfx canister call e2e_project_backend amInitializer --identity alice
  assert_eq '(true)'
  assert_command dfx canister call e2e_project_backend amInitializer
  assert_eq '(false)'

  assert_command dfx identity rename alice bob

  assert_command dfx identity whoami
  assert_eq 'default'
  assert_command dfx canister call e2e_project_backend amInitializer --identity bob
  assert_eq '(true)'

  assert_command dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob "hello"})' --identity bob
  assert_eq '()'
  assert_command dfx canister call --output idl e2e_project_frontend retrieve '("B")'
  # shellcheck disable=SC2154
  assert_eq '(blob "hello")' "$stdout"
}

@test "using an unencrypted identity on mainnet provokes a hard error which can be surpressed" {
  assert_command_fail dfx ledger balance --network ic
  assert_match "The default identity is not stored securely." "$stderr"
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/get_ledger_balance.exp"
  dfx identity new bob --storage-mode plaintext
  assert_command_fail dfx ledger balance --network ic --identity bob
  assert_match "The bob identity is not stored securely." "$stderr"
  # can surpress the error
  export DFX_WARNING=-mainnet_plaintext_identity
  assert_command dfx ledger balance --network ic --identity bob
  assert_not_contains "not stored securely" "$stderr"

}

@test "can call a canister using an ed25519 identity" {
  install_asset ed25519
  assert_command dfx identity import --storage-mode plaintext ed25519 identity.pem
  dfx_new # This installs replica and other binaries
  dfx identity use ed25519
  install_asset whoami
  dfx_start
  dfx canister create whoami
  dfx build
  dfx canister install whoami
  assert_command dfx canister call whoami whoami
  assert_eq '(principal "2nor3-keehi-duuup-d7jcn-onggn-3atzm-gejtl-5tlzn-k4g6c-nnbf7-7qe")'
  assert_command dfx identity get-principal
  assert_eq "2nor3-keehi-duuup-d7jcn-onggn-3atzm-gejtl-5tlzn-k4g6c-nnbf7-7qe"
}
