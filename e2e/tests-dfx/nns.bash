#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "ic-nns-init binary exists and is executable" {
    dfx cache install

    # it panics, but still shows help
    assert_command_fail "$(dfx cache show)/ic-nns-init" --help
    assert_match "thread 'main' panicked at 'Illegal arguments:"
    assert_match "ic-nns-init \[OPTIONS\]"
    assert_match "-h, --help.*Print help information"
    assert_match '--version.*Print version information'

    # --version fails too
    assert_command_fail "$(dfx cache show)/ic-nns-init" --version
}

@test "ic-admin binary exists and is executable" {
    dfx cache install

    assert_command "$(dfx cache show)/ic-admin" --help
    assert_match "Common command-line options for \`ic-admin\`"
}

@test "sns binary exists and is executable" {
    dfx cache install

    assert_command_fail "$(dfx cache show)/sns" --help
    assert_match "Initialize, deploy and interact with an SNS."
}

@test "dfx nns install command exists" {
    assert_command dfx nns install --help
}

@test "dfx nns install runs" {
    # Setup
    install_shared_asset subnet_type/shared_network_settings/system
    assert_command dfx start --clean --background --host 127.0.0.1:0
    assert_match "subnet_type: System" 

    # Run th einstallation
    assert_command dfx nns install

    # Checking that the install worked.
    # Note:  The installation is quite expensive, so we test extensively on one installation
    #        rather than doing a separate installation for every test.  The tests are read-only
    #        so no test should affect the output of another.
    # ... Canisters should exist:
    # ... ... Backend canisters:
    dfx canister id nns-registry
    dfx canister id nns-governance
    dfx canister id nns-ledger
    dfx canister id nns-root
    dfx canister id nns-cycles-minting
    dfx canister id nns-lifeline
    dfx canister id nns-genesis-token
    dfx canister id nns-sns-wasm
    # ... ... Frontend canisters:
    dfx canister id nns-identity
    dfx canister id nns-ui
    # ... Just to be sure that the existence check does not always pass:
    assert_command_fail dfx canister id i-always-return-true
    # ... Pages should be accessible for the front end canisters:
    BOUNDARY_ORIGIN="localhost:$(dfx info webserver-port)"
    canister_url() {
      echo "http://$(dfx canister id "$1").${BOUNDARY_ORIGIN}"
    }
    curl --fail -sSL "$(canister_url nns-identity)"
    curl --fail -sSL "$(canister_url nns-ui)"
    # The downloaded wasm files match the installed wasms
    installed_wasm_hash() {
        dfx canister info "$1" | awk '/Module hash/{print $3}'
    }
    downloaded_wasm_hash() {
        sha256sum ".dfx/wasms/nns/$(dfx --version | awk '{printf "%s-$%s", $1, $2}')/$1" | awk '{print "0x" $1}'
    }
    wasm_matches() {
            [[ "$(installed_wasm_hash $1)" == "$(downloaded_wasm_hash $2)" ]] || {
                echo "ERROR:  There is a wasm hash mismatch between $1 and $2"
                exit 1
            }>&2
    }
    wasm_matches nns-registry registry-canister.wasm
    wasm_matches nns-governance governance-canister_test.wasm
}
