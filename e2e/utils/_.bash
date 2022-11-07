set -e
load "${BATSLIB}/load.bash"
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
    dfx new "${project_name}" --frontend
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

dfx_patchelf() {
    # Don't run this function during github actions
    [ "$GITHUB_ACTIONS" ] && return 0

    # Only run this function on Linux
    (uname -a | grep Linux) || return 0

    local CACHE_DIR LD_LINUX_SO BINARY IS_STATIC USE_LIB64

    echo dfx = "$(which dfx)"
    CACHE_DIR="$(dfx cache show)"

    dfx cache install

    # Both ldd and iconv are providedin glibc.bin package
    LD_LINUX_SO=$(ldd "$(which iconv)"|grep ld-linux-x86|cut -d' ' -f3)
    for binary in ic-starter icx-proxy replica; do
        BINARY="${CACHE_DIR}/${binary}"
        test -f "$BINARY" || continue
        IS_STATIC=$(ldd "${BINARY}" | grep 'not a dynamic executable')
        USE_LIB64=$(ldd "${BINARY}" | grep '/lib64/ld-linux-x86-64.so.2')
        chmod +rw "${BINARY}"
        test -n "$IS_STATIC" || test -z "$USE_LIB64" || patchelf --set-interpreter "${LD_LINUX_SO}" "${BINARY}"
    done
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
    dfx_patchelf

    # Start on random port for parallel test execution
    FRONTEND_HOST="127.0.0.1:0"

    determine_network_directory
    if [ "$USE_IC_REF" ]
    then
        if [[ $# -eq 0 ]]; then
            dfx start --emulator --background --host "$FRONTEND_HOST" 3>&-
        else
            batslib_decorate "no arguments to dfx start --emulator supported yet"
            fail
        fi

        test -f "$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port"
        port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port")
    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        if [[ $# -eq 0 ]]; then
            dfx start --background --host "$FRONTEND_HOST" 3>&- # Start on random port for parallel test execution
        else
            dfx start --background "$@" 3>&-
        fi

        dfx_config_root="$E2E_NETWORK_DATA_DIRECTORY/replica-configuration"
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        test -f "${dfx_config_root}/replica-1.port"
        port=$(cat "${dfx_config_root}/replica-1.port")
    fi

    webserver_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/webserver-port")

    printf "Replica Configured Port: %s\n" "${port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    timeout 5 sh -c \
        "until nc -z localhost ${port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${port}" && exit 1)
}

wait_until_replica_healthy() {
    echo "waiting for replica to become healthy"
    dfx ping --wait-healthy
    echo "replica became healthy"
}

# Start the replica in the background.
dfx_replica() {
    local replica_port dfx_config_root
    dfx_patchelf
    determine_network_directory
    if [ "$USE_IC_REF" ]
    then
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx replica --emulator --port 0 "$@" 3>&- &
        export DFX_REPLICA_PID=$!

        timeout 60 sh -c \
            "until test -s \"$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port\"; do echo waiting for ic-ref port; sleep 1; done" \
            || (echo "replica did not write to \"$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port\" file" && exit 1)

        test -f "$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port"
        replica_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port")

    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx replica --port 0 "$@" 3>&- &
        export DFX_REPLICA_PID=$!

        timeout 60 sh -c \
            "until test -s \"$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port\"; do echo waiting for replica port; sleep 1; done" \
            || (echo "replica did not write to port file" && exit 1)

        dfx_config_root="$E2E_NETWORK_DATA_DIRECTORY/replica-configuration"
        test -f "${dfx_config_root}/replica-1.port"
        replica_port=$(cat "${dfx_config_root}/replica-1.port")

    fi

    printf "Replica Configured Port: %s\n" "${replica_port}"

    timeout 5 sh -c \
        "until nc -z localhost ${replica_port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${replica_port}" && exit 1)

    # ping the replica directly, because the bootstrap (that launches icx-proxy, which dfx ping usually connects to)
    # is not running yet
    dfx ping --wait-healthy "http://127.0.0.1:${replica_port}"
}

dfx_bootstrap() {
    # This only works because we use the network by name
    #    (implicitly: --network local)
    # If we passed --network http://127.0.0.1:${replica_port}
    # we would get errors like this:
    #    "Cannot find canister ryjl3-tyaaa-aaaaa-aaaba-cai for network http___127_0_0_1_54084"
    dfx bootstrap --port 0 3>&- &
    export DFX_BOOTSTRAP_PID=$!

    timeout 5 sh -c \
        "until nc -z localhost \$(cat \"$E2E_NETWORK_DATA_DIRECTORY/webserver-port\"); do echo waiting for webserver; sleep 1; done" \
        || (echo "could not connect to webserver on port $(get_webserver_port)" && exit 1)

    wait_until_replica_healthy

    webserver_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/webserver-port")
    printf "Webserver Configured Port: %s\n", "${webserver_port}"
}

# Stop the `dfx replica` process that is running in the background.
stop_dfx_replica() {
    [ "$DFX_REPLICA_PID" ] && kill -TERM "$DFX_REPLICA_PID"
    unset DFX_REPLICA_PID
}

# Stop the `dfx bootstrap` process that is running in the background
stop_dfx_bootstrap() {
    [ "$DFX_BOOTSTRAP_PID" ] && kill -TERM "$DFX_BOOTSTRAP_PID"
    unset DFX_BOOTSTRAP_PID
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
    if [ "$USE_IC_REF" ]
    then
        replica_port=$(get_ic_ref_port)
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

wallet_sha() {
    shasum -a 256 "${archive}/wallet/$1/wallet.wasm" | awk '{ print $1 }'
}

use_default_wallet_wasm() {
    unset DFX_WALLET_WASM
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

get_ic_ref_port() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-ref.port"

}
get_replica_port() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port"
}

get_btc_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

get_canister_http_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-canister-http-adapter-pid"
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
