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
    # Both ldd and iconv are providedin glibc.bin package
    local LD_LINUX_SO=$(ldd $(which iconv)|grep ld-linux-x86|cut -d' ' -f3)
    for binary in ic-starter replica; do
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

# Start the replica in the background.
dfx_start_replica_and_bootstrap() {
    echo "1" >/Users/ericswanson/x.txt
    dfx_patchelf
    if [ "$USE_IC_REF" ]
    then
        echo "No replica+bootstrap with emulator"
        fail
    else
        echo "2.-2" >>/Users/ericswanson/x.txt
        dfx cache install
        echo "2.-1" >>/Users/ericswanson/x.txt
        echo "2" >>/Users/ericswanson/x.txt
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx replica --port 0 "$@" 3>&- &
        echo "2.1" >>/Users/ericswanson/x.txt

        timeout 60 sh -c \
            "until cat .dfx/replica-configuration/replica-1.port; do echo waiting for replica port; sleep 1; done" \
            || (echo "replica did not write to port file" && exit 1)

        echo "2.2" >>/Users/ericswanson/x.txt
        ls -lR .dfx >>/Users/ericswanson/x.txt

        echo "3" >>/Users/ericswanson/x.txt
        local dfx_config_root=.dfx/replica-configuration
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        echo "3.1" >>/Users/ericswanson/x.txt
        test -f ${dfx_config_root}/replica-1.port
        echo "3.2" >>/Users/ericswanson/x.txt
        local port=$(cat ${dfx_config_root}/replica-1.port)

        echo "4" >>/Users/ericswanson/x.txt

        # Overwrite the default networks.local.bind 127.0.0.1:8000 with allocated port
        local webserver_port=$(cat .dfx/webserver-port)
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json

        echo "5" >>/Users/ericswanson/x.txt
    fi

    printf "Replica Configured Port: %s\n" "${port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    echo "6" >>/Users/ericswanson/x.txt

    timeout 5 sh -c \
        "until nc -z localhost ${port}; do echo waiting for replica; sleep 1; done" \
        || (echo "could not connect to replica on port ${port}" && exit 1)

    echo "7" >>/Users/ericswanson/x.txt

    dfx bootstrap --network http://127.0.0.1:${port} --port 0 3>&- &

    echo "8" >>/Users/ericswanson/x.txt

    timeout 5 sh -c \
        'until nc -z localhost $(cat .dfx/candid-port); do echo waiting for bootstrap; sleep 1; done' \
        || (echo "could not connect to bootstrap on port $(cat .dfx/candid-port)" && exit 1)

    echo "9" >>/Users/ericswanson/x.txt

    local candid_port=$(cat .dfx/candid-port)
    printf "Candid Configured Port: %s", "${candid_port}"
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
