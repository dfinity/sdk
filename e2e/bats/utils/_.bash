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
    if [ "$USE_IC_REF" ]
    then
        ic-ref --pick-port --write-port-to port 3>&- &
        echo $! > ic-ref.pid

        sleep 5

        test -f port
        local port=$(cat port)

        cat <<<$(jq .networks.local.bind=\"127.0.0.1:${port}\" dfx.json) >dfx.json
        cat dfx.json
        if [[ "$@" == "" ]]; then
            dfx bootstrap --port 0 & # Start on random port for parallel test execution
        else
            dfx bootstrap --port "$@" &
        fi
        local webserver_port=$(cat .dfx/webserver-port)
        echo $! > dfx-bootstrap.pid
    else
        # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
        # wait for it to close. Because `dfx start` leaves child processes running, we need
        # to close this pipe, otherwise Bats will wait indefinitely.
        if [[ "$@" == "" ]]; then
            dfx start --background --host "127.0.0.1:0" 3>&- # Start on random port for parallel test execution
        else
            dfx start --background "$@" 3>&-
        fi
        local project_dir=${pwd}
        local dfx_config_root=.dfx/client-configuration
        printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
        test -f ${dfx_config_root}/client-1.port
        sleep 3
        local port=$(cat ${dfx_config_root}/client-1.port)

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

# Stop the client and verify it is very very stopped.
dfx_stop() {
    if [ "$USE_IC_REF" ]
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
        local dfx_root=.dfx/
        rm -rf $dfx_root

        # Verify that processes are killed.
        ! ( ps | grep "[/\s]dfx start" )
        ! ( ps | grep "[/\s]replica" )
    fi
}
