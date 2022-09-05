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

# dfx nns install works only on port 8080, as URLs are compiled into the wasms.
# - It may be possible in future for the wasms to detect their own URL and recompute signatures accordingly,
#   however until such a time, we have this restriction.
# - It may also be that ic-nns-install, if used on a non-standard port, installs only the core canisters not the UI.
# - However until we have implemented good solutions, all tests on ic-nns-install must run on port 8080.
dfx_start_for_nns_install() {
    # TODO: When nns-dapp supports dynamic ports, this wait can be removed.
    assert_command timeout 600 sh -c \
        "until dfx start --clean --background --host 127.0.0.1:8080; do echo waiting for port 8080 to become free; sleep 3; done" \
        || (echo "could not connect to replica on port ${replica_port}" && exit 1)
    assert_match "subnet_type: System"
    assert_match "127.0.0.1:8080"
}

# The nns canister IDs should be installed without touching any of the repository files,
# so we cannot rely on `dfx canister id` when testing.
nns_canister_id() {
    case "$1" in
    nns-registry)          echo "rwlgt-iiaaa-aaaaa-aaaaa-cai" ;;
    nns-governance)        echo "rrkah-fqaaa-aaaaa-aaaaq-cai" ;;
    nns-ledger)            echo "ryjl3-tyaaa-aaaaa-aaaba-cai" ;;
    nns-root)              echo "r7inp-6aaaa-aaaaa-aaabq-cai" ;;
    nns-cycles-minting)    echo "rkp4c-7iaaa-aaaaa-aaaca-cai" ;;
    nns-lifeline)          echo "rno2w-sqaaa-aaaaa-aaacq-cai" ;;
    nns-genesis-token)     echo "renrk-eyaaa-aaaaa-aaada-cai" ;;
    nns-sns-wasm)          echo "qjdve-lqaaa-aaaaa-aaaeq-cai" ;;
    internet_identity)     echo "qaa6y-5yaaa-aaaaa-aaafa-cai" ;;
    nns-dapp)              echo "qhbym-qaaaa-aaaaa-aaafq-cai" ;;
    *)                     echo "ERROR: Unknown NNS canister '$1'." >&2
                           exit 1;;
    esac
}

@test "dfx nns install runs" {
    # Setup
    install_shared_asset subnet_type/shared_network_settings/system
    dfx_start_for_nns_install
    dfx nns install

    # Checking that the install worked.
    # Note:  The installation is quite expensive, so we test extensively on one installation
    #        rather than doing a separate installation for every test.  The tests are read-only
    #        so no test should affect the output of another.
    BOUNDARY_ORIGIN="localhost:$(dfx info webserver-port)"
    canister_url() {
      echo "http://$(nns_canister_id "$1").${BOUNDARY_ORIGIN}"
    }
    curl --fail -sSL "$(canister_url internet_identity)"
    curl --fail -sSL "$(canister_url nns-dapp)"
    # The downloaded wasm files match the installed wasms
    installed_wasm_hash() {
        dfx canister info "$(nns_canister_id "$1")" | awk '/Module hash/{print $3; exit 0}END{exit 1}'
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
    wasm_matches nns-ledger ledger-canister_notify-method.wasm
    wasm_matches nns-root root-canister.wasm
    wasm_matches nns-cycles-minting cycles-minting-canister.wasm
    wasm_matches nns-lifeline lifeline.wasm
    wasm_matches nns-genesis-token genesis-token-canister.wasm
    wasm_matches nns-sns-wasm sns-wasm-canister.wasm
    wasm_matches internet_identity internet_identity_dev.wasm
    wasm_matches nns-dapp nns-dapp_local.wasm
    dfx stop
}

@test "dfx nns install should fail on unclean testnet" {
    # Setup
    # ... Install the usual configuration
    install_shared_asset subnet_type/shared_network_settings/system
    # ... Check that the usual configuration is suitable
    (( $(jq '.canisters | to_entries | del(.[] | select(.value.remote)) | length' dfx.json) > 0 )) || {
        echo "This test needs dfx.json to define at least one non-remote canister"
        exit 1
    } >&2
    # ... Start dfx
    dfx_start_for_nns_install
    # ... Steal canister numbber zero
    dfx canister create --all --no-wallet
    # ... Installing the nns should now fail but there should be a helpful error message.
    assert_command_fail dfx nns install
    assert_match "dfx start --clean"

    dfx stop
}