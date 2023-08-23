#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
    export DFX_CI_USE_PROXY_KEYRING=""
}

teardown() {
    standard_teardown
}

#
# These tests generally only test the case where passwords are set.
# The case without password is tested already in other places, such as identity_command.bash or identity.bash
#

@test "can create and use identity with password" {
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/create_identity_with_password.exp"
}

@test "wrong password is rejected" {
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/wrong_password_rejected.exp"
}

@test "import and export identity with a password are inverse operations" {
    # key generated using `openssl ecparam -genkey -name secp256k1`
    cat >import.pem <<EOF
-----BEGIN EC PARAMETERS-----
BgUrgQQACg==
-----END EC PARAMETERS-----
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEIIPXmSpdZwI5YUwzukz8+GC9fikjMELmdbH4tHcQ9iD2oAcGBSuBBAAK
oUQDQgAEjjBKAxko3RPG8ot7PoeXM7ZHtek2xcbRN/JZVfKKNEnNG4wdnMdpRGyk
37fJkz9WEHR+Wol+nGAuQNnCOIVXdw==
-----END EC PRIVATE KEY-----
EOF
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/import_export_identity_with_password.exp"
    assert_eq "$(cat import.pem)" "$(cat export.pem)"

}

@test "rename identity works on identity with a password" {
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/rename_identity_with_password.exp"
}

@test "remove identity works on identity with a password" {
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_pw.exp"
    assert_command dfx identity remove alice
}

@test "creating/removing identity with --storage-mode password-protected does not attempt to touch keyring" {
    # if any of dfx identity new --storage-mode password-protected or dfx identity remove attempt to touch keyring,
    # then the test will time out because it's waiting for the user to accept/deny access to keyring
    unset DFX_CI_MOCK_KEYRING_LOCATION
    assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/init_alice_with_storage_mode_pwprotected.exp"
    assert_command dfx identity remove alice
}
