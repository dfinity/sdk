set -eo pipefail
load ../utils/bats-support/load
load ../utils/assertions
load ../utils/webserver

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/../assets/$1/
    cp -R "$ASSET_ROOT"/* .

    # shellcheck source=/dev/null
    if [ -f ./patch.bash ]; then source ./patch.bash; fi
    if [ -f ./Cargo.toml ]; then cargo update; fi
}

install_shared_asset() {
    mkdir -p "$(dirname "$E2E_NETWORKS_JSON")"

    ASSET_ROOT=${BATS_TEST_DIRNAME}/../assets/$1/
    cp -R "$ASSET_ROOT"/* "$(dirname "$E2E_NETWORKS_JSON")"
}

standard_setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export E2E_TEMP_DIR="$x"

    cache_root="${E2E_CACHE_ROOT:-"$HOME/.e2e-cache-root"}"

    mkdir "$x/working-dir"
    mkdir -p "$cache_root"
    mkdir "$x/config-root"
    mkdir "$x/home-dir"

    cd "$x/working-dir" || exit

    export HOME="$x/home-dir"
    export DFX_CACHE_ROOT="$cache_root"
    export DFX_CONFIG_ROOT="$x/config-root"
    export RUST_BACKTRACE=1
    export MOCK_KEYRING_LOCATION="$HOME/mock_keyring.json"

    if [ "$(uname)" == "Darwin" ]; then
        export E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY="$HOME/Library/Application Support/org.dfinity.dfx/network/local"
    elif [ "$(uname)" == "Linux" ]; then
        export E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY="$HOME/.local/share/dfx/network/local"
    fi
    export E2E_NETWORKS_JSON="$DFX_CONFIG_ROOT/.config/dfx/networks.json"
}

standard_teardown() {
    rm -rf "$E2E_TEMP_DIR" || rm -rf "$E2E_TEMP_DIR"
}

dfx_new_frontend() {
    local project_name=${1:-e2e_project}
    dfx new "${project_name}" --frontend vanilla
    test -d "${project_name}"
    test -f "${project_name}"/dfx.json
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_new_assets() {
    local project_name=${1:-e2e_project}
    dfx new "${project_name}" --frontend simple-assets
    test -d "${project_name}"
    test -f "${project_name}"/dfx.json
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_new() {
    local project_name=${1:-e2e_project}
    dfx new "${project_name}" --no-frontend
    test -d "${project_name}"
    test -f "${project_name}/dfx.json"
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_new_rust() {
    local project_name=${1:-e2e_project}
    rustup default stable
    rustup target add wasm32-unknown-unknown
    dfx new "${project_name}" --type=rust --no-frontend
    test -d "${project_name}"
    test -f "${project_name}/dfx.json"
    test -f "${project_name}/Cargo.toml"
    test -f "${project_name}/Cargo.lock"
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

determine_network_directory() {
    # not perfect: dfx.json can actually exist in a parent
    if [ -f dfx.json ] && [ "$(jq .networks.local dfx.json)" != "null" ]; then
        echo "found dfx.json with local network in $(pwd)"
        data_dir="$(pwd)/.dfx/network/local"
        wallets_json="$(pwd)/.dfx/local/wallets.json"
        dfx_json="$(pwd)/dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$data_dir"
        export E2E_NETWORK_WALLETS_JSON="$wallets_json"
        export E2E_ROUTE_NETWORKS_JSON="$dfx_json"
    else
        echo "no dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY"
        export E2E_NETWORK_WALLETS_JSON="$E2E_NETWORK_DATA_DIRECTORY/wallets.json"
        export E2E_ROUTE_NETWORKS_JSON="$E2E_NETWORKS_JSON"
    fi
}

# Start the replica in the background.
dfx_start() {
    local port dfx_config_root webserver_port

    local args=( "$@" )

    add_default_parameter() {
        local param_name=$1
        if [[ $# != 1 ]]; then
            local param_value=$2
        fi
        local has_param=false
        for arg in "${args[@]}"; do
            if [[ $arg == "$param_name" ]]; then
                has_param=true
                break
            fi
        done
        if ! $has_param; then
            args+=( "$param_name" ${param_value+"$param_value"} )
        fi
    }

    # By default, start on random port for parallel test execution
    add_default_parameter "--host" "127.0.0.1:0"
    if [[ "$USE_POCKETIC" ]]; then
        add_default_parameter "--pocketic"
    else
        add_default_parameter "--artificial-delay" "100"
    fi

    determine_network_directory

    # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leaves child processes running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.

    dfx start --background "${args[@]}" 3>&-

    if [[ "$USE_POCKETIC" ]]; then
        test -f "$E2E_NETWORK_DATA_DIRECTORY/pocket-ic.port"
        port=$(< "$E2E_NETWORK_DATA_DIRECTORY/pocket-ic.port")
    else
        dfx_config_root="$E2E_NETWORK_DATA_DIRECTORY/replica-configuration"
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        test -f "${dfx_config_root}/replica-1.port"
        port=$(cat "${dfx_config_root}/replica-1.port")
        if [ "$port" == "" ]; then
          port=$(jq -r .local.replica.port "$E2E_NETWORKS_JSON")
        fi
    fi
    webserver_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/webserver-port")

    printf "Replica Configured Port: %s\n" "${port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    timeout 5 sh -c \
        "until nc -z localhost ${port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${port}" && exit 1)
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
        "until dfx start --clean --background --host 127.0.0.1:8080 --verbose --artificial-delay 100; do echo waiting for port 8080 to become free; sleep 3; done" \
        || (echo "could not connect to replica on port 8080" && exit 1)
    assert_match "subnet type: System"
    assert_match "127.0.0.1:8080"
}

wait_until_replica_healthy() {
    echo "waiting for replica to become healthy"
    dfx ping --wait-healthy
    echo "replica became healthy"
}

# Stop the replica and verify it is very very stopped.
dfx_stop() {
    # to help tell if other icx-proxy processes are from this test:
    echo "pwd: $(pwd)"
    # A suspicion: "address already is use" errors are due to an extra icx-proxy process.
    echo "icx-proxy processes:"
    pgrep -l icx-proxy || echo "no ps/grep/icx-proxy output"

    dfx stop
    local dfx_root=.dfx/
    rm -rf $dfx_root

    # Verify that processes are killed.
    assert_no_dfx_start_or_replica_processes
}

dfx_set_wallet() {
  export WALLET_CANISTER_ID
  WALLET_CANISTER_ID=$(dfx identity get-wallet)
  assert_command dfx identity set-wallet "${WALLET_CANISTER_ID}" --force --network actuallylocal
  assert_match 'Wallet set successfully.'
}

setup_actuallylocal_project_network() {
    webserver_port=$(get_webserver_port)
    # [ ! -f "$E2E_ROUTE_NETWORKS_JSON" ] && echo "{}" >"$E2E_ROUTE_NETWORKS_JSON"
    jq '.networks.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' dfx.json | sponge dfx.json
}

setup_actuallylocal_shared_network() {
    webserver_port=$(get_webserver_port)
    [ ! -f "$E2E_NETWORKS_JSON" ] && echo "{}" >"$E2E_NETWORKS_JSON"
    jq '.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

setup_local_shared_network() {
    local replica_port
    if [[ "$USE_POCKETIC" ]]; then
        replica_port=$(get_pocketic_port)
    else
        replica_port=$(get_replica_port)
    fi

    [ ! -f "$E2E_NETWORKS_JSON" ] && echo "{}" >"$E2E_NETWORKS_JSON"

    jq ".local.bind=\"127.0.0.1:${replica_port}\"" "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

use_wallet_wasm() {
    # shellcheck disable=SC2154
    export DFX_WALLET_WASM="${archive}/wallet/$1/wallet.wasm"
}

use_asset_wasm() {
    # shellcheck disable=SC2154
    export DFX_ASSETS_WASM="${archive}/frontend/$1/assetstorage.wasm.gz"
}

wallet_sha() {
    shasum -a 256 "${archive}/wallet/$1/wallet.wasm" | awk '{ print $1 }'
}

use_default_wallet_wasm() {
    unset DFX_WALLET_WASM
}

use_default_asset_wasm() {
    unset DFX_ASSETS_WASM
}

get_webserver_port() {
  dfx info webserver-port
}
overwrite_webserver_port() {
  echo "$1" >"$E2E_NETWORK_DATA_DIRECTORY/webserver-port"
}

get_replica_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-pid"
}

get_replica_port() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port"
}

get_pocketic_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/pocket-ic-pid"
}

get_pocketic_port() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/pocket-ic.port"
}

get_btc_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

get_canister_http_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

get_icx_proxy_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/icx-proxy-pid"
}

create_networks_json() {
  mkdir -p "$(dirname "$E2E_NETWORKS_JSON")"
  [ ! -f "$E2E_NETWORKS_JSON" ] && echo "{}" >"$E2E_NETWORKS_JSON"
}

define_project_network() {
    jq .networks.local.bind=\"127.0.0.1:8000\" dfx.json | sponge dfx.json
}

use_test_specific_cache_root() {
    # Use this when a test depends on the initial state of the cache being empty,
    # or if the test corrupts the cache in some way.
    # The effect is to ignore the E2E_CACHE_ROOT environment variable, if set.
    export DFX_CACHE_ROOT="$E2E_TEMP_DIR/cache-root"
    mkdir -p "$DFX_CACHE_ROOT"
}

get_ephemeral_port() {
    local script_dir
    script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    python3 "$script_dir/get_ephemeral_port.py"
}

stop_and_delete() {
    assert_command dfx canister stop "$1"
    assert_command dfx canister delete -y --no-withdrawal "$1"
    echo "Canister $1 deleted"
}
