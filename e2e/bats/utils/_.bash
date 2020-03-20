source ${BATSLIB}/load.bash
load utils/assertions

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/assets/$1/
    cp -R $ASSET_ROOT/* .

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

# Start the client in the background.
dfx_start() {
    echo $USE_IC_REF
    if [ "$USE_IC_REF" = "true" ]
    then
        ic-ref --pick-port --write-port-to port 3>&- &
        echo $! > ic-ref.pid

        sleep 5

        test -f port
        local port=$(cat port)

        dfx bootstrap --port 8000 --providers http://127.0.0.1:${port}/api &
        echo $! > dfx-bootstrap.pid

    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        dfx start --background "$@" 3>&-
        local project_dir=${pwd}
        local dfx_config_root=.dfx/client-configuration
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        test -f ${dfx_config_root}/client-1.port
        local port=$(cat ${dfx_config_root}/client-1.port)
    fi
    printf "Client Configured Port: %s\n" "${port}"


    timeout 5 sh -c \
        "until nc -z localhost ${port}; do echo waiting for client; sleep 1; done" \
        || (echo "could not connect to client on port ${port}" && exit 1)
}

# Stop the client and verify it is very very stopped.
dfx_stop() {
    if [ "$USE_IC_REF" = "true" ]
    then
        test -f ic-ref.pid
        printf "Killing ic-ref at pid: %u\n" "$(cat ic-ref.pid)"
        kill $(cat ic-ref.pid)
        rm -f ic-ref.pid

        test -f dfx-bootstrap.pid
        printf "Killing dfx bootstrap at pid: %u\n" "$(cat dfx-bootstrap.pid)"
        kill $(cat dfx-bootstrap.pid)
        rm -f dfx-bootstrap.pid
    else
        dfx stop

        # Verify that processes are killed.
        ! ( ps | grep " [d]fx start" )
    fi
}
