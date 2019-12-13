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
    # Bats create a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leave a child process running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.
    dfx start --background "$@" 3>&-

    timeout 5 sh -c \
        'until nc -z localhost 8080; do echo waiting for client; sleep 1; done' \
        || (echo "could not connect to client on port 8080" && exit 1)
}

