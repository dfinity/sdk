#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    stop_webserver

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

# Tries to start dfx on the default port, repeating until it succeeds or times out.
#
# Motivation: dfx nns install works only on port 8080, as URLs are compiled into the wasms.  This means that multiple
# tests MAY compete for the same port.
# - It may be possible in future for the wasms to detect their own URL and recompute signatures accordingly,
#   however until such a time, we have this restriction.
# - It may also be that ic-nns-install, if used on a non-standard port, installs only the core canisters not the UI.
# - However until we have implemented good solutions, all tests on ic-nns-install must run on port 8080.
dfx_start_for_nns_install() {
    # TODO: When nns-dapp supports dynamic ports, this wait can be removed.
    assert_command timeout 300 sh -c \
        "until dfx start --clean --background --host 127.0.0.1:8080; do echo waiting for port 8080 to become free; sleep 3; done" \
        || (echo "could not connect to replica on port 8080" && exit 1)
    assert_match "subnet_type: System"
    assert_match "127.0.0.1:8080"
}

# The nns canisters should be installed without changing any of the developer's project files,
# so we cannot rely on `dfx canister id` when testing.  We rely on these hard-wired values instead:
nns_canister_id() {
    case "$1" in
    nns-registry)          echo "rwlgt-iiaaa-aaaaa-aaaaa-cai" ;;
    nns-governance)        echo "rrkah-fqaaa-aaaaa-aaaaq-cai" ;;
    nns-ledger)            echo "ryjl3-tyaaa-aaaaa-aaaba-cai" ;;
    nns-root)              echo "r7inp-6aaaa-aaaaa-aaabq-cai" ;;
    nns-cycles-minting)    echo "rkp4c-7iaaa-aaaaa-aaaca-cai" ;;
    nns-lifeline)          echo "rno2w-sqaaa-aaaaa-aaacq-cai" ;;
    nns-genesis-token)     echo "renrk-eyaaa-aaaaa-aaada-cai" ;;
    nns-sns-wasm)          echo "qaa6y-5yaaa-aaaaa-aaafa-cai" ;;
    internet_identity)     echo "qhbym-qaaaa-aaaaa-aaafq-cai" ;;
    nns-dapp)              echo "qsgjb-riaaa-aaaaa-aaaga-cai" ;;
    *)                     echo "ERROR: Unknown NNS canister '$1'." >&2
                           exit 1;;
    esac
}

@test "dfx nns install runs" {
    echo Setting up...
    install_shared_asset subnet_type/shared_network_settings/system
    dfx_start_for_nns_install
    dfx nns install

    echo Getting canister IDs so that we can test that the expected wasms are installed...
    dfx nns import --network-mapping local=mainnet
    # nns install does not install all required wasms.  We also need:
    jq '.canisters.internet_identity.remote.id.local="qhbym-qaaaa-aaaaa-aaafq-cai"' dfx.json | sponge dfx.json
    jq '.canisters["nns-dapp"].remote.id.local="qsgjb-riaaa-aaaaa-aaaga-cai"' dfx.json | sponge dfx.json
    jq '.canisters["nns-sns-wasm"].remote.id.local="qaa6y-5yaaa-aaaaa-aaafa-cai"' dfx.json | sponge dfx.json
    # Showing the IDs we have...
    jq '.canisters | to_entries | map({key: .key, id: .value.remote.id.local})' dfx.json
    jq '.canisters["nns-sns-wasm"]' dfx.json

    echo Checking that the install worked...
    # Note:  The installation is quite expensive, so we test extensively on one installation
    #        rather than doing a separate installation for every test.  The tests are read-only
    #        so no test should affect the output of another.
    installed_wasm_hash() {
        dfx canister info "$(nns_canister_id "$1")" | awk '/Module hash/{print $3; exit 0}END{exit 1}'
    }
    download_dir() {
        echo ".dfx/wasms/nns/$(dfx --version | awk '{printf "%s-%s", $1, $2}')"
    }
    downloaded_wasm_hash() {
        sha256sum "$(download_dir)/$1" | awk '{print "0x" $1}'
    }
    wasm_matches() {
        echo "Comparing $* ..."
        [[ "$(installed_wasm_hash "$1")" == "$(downloaded_wasm_hash "$2")" ]] || {
                echo "ERROR:  There is a wasm hash mismatch between $1 and $2"
                echo "ERROR:  $(installed_wasm_hash "$1") != $(downloaded_wasm_hash "$2")"
                echo "ERROR:  Install:  $(dfx canister info "$1")"
                echo "ERROR:  Download: $(ls -l "$(download_dir "$2")")"
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

    echo Stopping dfx...
    dfx stop
}

test_project_import() {
    DFX_JSON_LOCATION="$1"

    # this test is meant to demonstrate that the various
    dfx beta project import "$DFX_JSON_LOCATION" --prefix "pfx-" --network-mapping ic=mainnet --all

    jq . dfx.json

    assert_command jq -r '.canisters."pfx-normal-canister".candid' dfx.json
    assert_eq "candid/pfx-normal-canister.did"
    # shellcheck disable=SC2154
    assert_files_eq \
      "${assets}/project-import/project-directory/normal-canister-directory/some-subdirectory/the-candid-filename.did" \
      "candid/pfx-normal-canister.did"

    assert_command jq -r '.canisters."pfx-normal-canister".remote.id.ic' dfx.json
    assert_eq "rrkah-fqaaa-aaaaa-aaaaq-cai"

    assert_command jq -r '.canisters."pfx-sibling".candid' dfx.json
    assert_eq "candid/pfx-sibling.did"
    assert_files_eq \
      "${assets}/project-import/sibling-project/canister/canister/the-sibling-candid-definition.did" \
      "candid/pfx-sibling.did"
}

@test "dfx project import from filesystem" {
    test_project_import "${assets}/project-import/project-directory/dfx.json"
}

@test "dfx project import from url" {
    start_webserver --directory "${assets}/project-import"

    test_project_import "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json"
}

test_project_import_specific_canister() {
    LOCATION="$1"

    # this test is meant to demonstrate that the various
    dfx beta project import "$LOCATION" normal-canister

    jq . dfx.json

    assert_command jq -r '.canisters."normal-canister".candid' dfx.json
    assert_eq "candid/normal-canister.did"
    assert_files_eq \
      "${assets}/project-import/project-directory/normal-canister-directory/some-subdirectory/the-candid-filename.did" \
      "candid/normal-canister.did"

    assert_command jq -r '.canisters.sibling.candid' dfx.json
    assert_eq "null"
}

@test "dfx project import specific canister" {
    test_project_import_specific_canister "${assets}/project-import/project-directory/dfx.json"
}

@test "import from url" {
    start_webserver --directory "${assets}/project-import"

    test_project_import_specific_canister "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json"
}

@test "project import from filesystem with no canister_ids.json" {
    mkdir www
    cp -R "${assets}/project-import" www/
    rm www/project-import/project-directory/canister_ids.json

    start_webserver --directory "www/project-import"

    dfx beta project import www/project-import/project-directory/dfx.json --all
}

@test "project import from url with no canister_ids.json" {
    mkdir www
    cp -R "${assets}/project-import" www/
    rm www/project-import/project-directory/canister_ids.json

    start_webserver --directory "www/project-import"

    dfx beta project import "http://localhost:$E2E_WEB_SERVER_PORT/project-directory/dfx.json" --all
}
