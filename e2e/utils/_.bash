load ${BATSLIB}/load.bash
load ../utils/assertions

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/../assets/$1/
    cp -R $ASSET_ROOT/* .
    # set write perms to overwrite local bind in assets which have a dfx.json
    chmod -R a+w .

    [ -f ./patch.bash ] && source ./patch.bash
}

standard_setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export DFX_E2E_TEMP_DIR="$x"

    mkdir "$x/working-dir"
    mkdir "$x/config-root"
    mkdir "$x/home-dir"

    cd "$x/working-dir" || exit

    export HOME="$x/home-dir"
    export DFX_CONFIG_ROOT="$x/config-root"
    export RUST_BACKTRACE=1
}

standard_teardown() {
    rm -rf "$DFX_E2E_TEMP_DIR"
}

dfx_new_frontend() {
    local project_name=${1:-e2e_project}
    dfx new ${project_name} --frontend
    test -d ${project_name}
    test -f ${project_name}/dfx.json
    cd ${project_name}

    echo PWD: $(pwd) >&2
}

dfx_new() {
    local project_name=${1:-e2e_project}
    dfx new ${project_name} --no-frontend
    test -d ${project_name}
    test -f ${project_name}/dfx.json
    cd ${project_name}

    echo PWD: $(pwd) >&2
}

dfx_patchelf() {
    # Only run this function on Linux
    (uname -a | grep Linux) || return 0
    echo dfx = $(which dfx)
    local CACHE_DIR="$(dfx cache show)"

    dfx cache install

    # Both ldd and iconv are providedin glibc.bin package
    local LD_LINUX_SO=$(ldd $(which iconv)|grep ld-linux-x86|cut -d' ' -f3)
    for binary in ic-starter icx-proxy replica; do
        local BINARY="${CACHE_DIR}/${binary}"
        test -f "$BINARY" || continue
        local IS_STATIC=$(ldd "${BINARY}" | grep 'not a dynamic executable')
        local USE_LIB64=$(ldd "${BINARY}" | grep '/lib64/ld-linux-x86-64.so.2')
        chmod +rw "${BINARY}"
        test -n "$IS_STATIC" || test -z "$USE_LIB64" || patchelf --set-interpreter "${LD_LINUX_SO}" "${BINARY}"
    done
}

# Start the replica in the background.
dfx_start() {
    dfx_patchelf
    if [ "$USE_IC_REF" ]
    then
        if [[ "$@" == "" ]]; then
            dfx start --emulator --background --host "127.0.0.1:0" 3>&- # Start on random port for parallel test execution
        else
            batslib_decorate "no arguments to dfx start --emulator supported yet"
            fail
        fi

        test -f .dfx/ic-ref.port
        local port=$(cat .dfx/ic-ref.port)

        # Overwrite the default networks.local.bind 127.0.0.1:8000 with allocated port
        local webserver_port=$(cat .dfx/webserver-port)
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json
    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        if [[ "$@" == "" ]]; then
            dfx start --background --host "127.0.0.1:0" 3>&- # Start on random port for parallel test execution
        else
            dfx start --background "$@" 3>&-
        fi

        local dfx_config_root=.dfx/replica-configuration
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        test -f ${dfx_config_root}/replica-1.port
        local port=$(cat ${dfx_config_root}/replica-1.port)

        # Overwrite the default networks.local.bind 127.0.0.1:8000 with allocated port
        local webserver_port=$(cat .dfx/webserver-port)
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json
    fi

    printf "Replica Configured Port: %s\n" "${port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    timeout 5 sh -c \
        "until nc -z localhost ${port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${port}" && exit 1)
}

wait_until_replica_healthy() {
    echo "waiting for replica to become healthy"
    (
        # dfx ping has side effects, like creating a default identity.
        DFX_CONFIG_ROOT="$DFX_E2E_TEMP_DIR/dfx-ping-tmp"
        dfx ping --wait-healthy
    )
    echo "replica became healthy"
}

# Start the replica in the background.
dfx_start_replica_and_bootstrap() {
    dfx_patchelf
    if [ "$USE_IC_REF" ]
    then
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx replica --emulator --port 0 "$@" 3>&- &
        export DFX_REPLICA_PID=$!

        timeout 60 sh -c \
            "until test -s .dfx/ic-ref.port; do echo waiting for ic-ref port; sleep 1; done" \
            || (echo "replica did not write to .dfx/ic-ref.port file" && exit 1)

        test -f .dfx/ic-ref.port
        local replica_port=$(cat .dfx/ic-ref.port)

    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx replica --port 0 "$@" 3>&- &
        export DFX_REPLICA_PID=$!

        timeout 60 sh -c \
            "until test -s .dfx/replica-configuration/replica-1.port; do echo waiting for replica port; sleep 1; done" \
            || (echo "replica did not write to port file" && exit 1)

        local dfx_config_root=.dfx/replica-configuration
        test -f ${dfx_config_root}/replica-1.port
        local replica_port=$(cat ${dfx_config_root}/replica-1.port)

    fi
    local webserver_port=$(cat .dfx/webserver-port)

    # Overwrite the default networks.local.bind 127.0.0.1:8000 with allocated port
    cat <<<$(jq .networks.local.bind=\"127.0.0.1:${replica_port}\" dfx.json) >dfx.json

    printf "Replica Configured Port: %s\n" "${replica_port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    timeout 5 sh -c \
        "until nc -z localhost ${replica_port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${replica_port}" && exit 1)

    wait_until_replica_healthy

    # This only works because we use the network by name
    #    (implicitly: --network local)
    # If we passed --network http://127.0.0.1:${replica_port}
    # we would get errors like this:
    #    "Cannot find canister ryjl3-tyaaa-aaaaa-aaaba-cai for network http___127_0_0_1_54084"
    dfx bootstrap --port 0 3>&- &
    export DFX_BOOTSTRAP_PID=$!

    timeout 5 sh -c \
        'until nc -z localhost $(cat .dfx/proxy-port); do echo waiting for bootstrap; sleep 1; done' \
        || (echo "could not connect to bootstrap on port $(cat .dfx/proxy-port)" && exit 1)

    local proxy_port=$(cat .dfx/proxy-port)
    printf "Proxy Configured Port: %s\n", "${proxy_port}"
}

# Start the replica in the background.
dfx_stop_replica_and_bootstrap() {
    if [[ -v DFX_REPLICA_PID ]]; then
        kill -TERM "$DFX_REPLICA_PID"
    fi
    if [[ -v DFX_BOOTSTRAP_PID ]]; then
        kill -TERM "$DFX_BOOTSTRAP_PID"
    fi
}

# Stop the replica and verify it is very very stopped.
dfx_stop() {
    dfx stop
    local dfx_root=.dfx/
    rm -rf $dfx_root

    # Verify that processes are killed.
    assert_no_dfx_start_or_replica_processes
}

dfx_set_wallet() {
  export WALLET_CANISTER_ID=$(dfx identity get-wallet)
  assert_command dfx identity  --network actuallylocal set-wallet ${WALLET_CANISTER_ID} --force
  assert_match 'Wallet set successfully.'
}

setup_actuallylocal_network() {
    webserver_port=$(cat .dfx/webserver-port)
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' dfx.json)" >dfx.json
}

use_wallet_wasm() {
    # shellcheck disable=SC2154
    export DFX_WALLET_WASM="${archive}/wallet/$1/wallet.wasm"
}
