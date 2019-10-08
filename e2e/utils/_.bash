source ${BATSLIB}/load.bash
load utils/assertions

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/assets/$1/
    cp -R $ASSET_ROOT/* .

    [ -f ./patch.bash ] && source ./patch.bash
}

dfx_new() {
    dfx new e2e-project
    test -d e2e-project
    test -f e2e-project/dfinity.json
    cd e2e-project

    echo PWD: $(pwd) >&2
}

# Start the client in the background.
dfx_start() {
    # Bats create a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leave a child process running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.
    dfx start --background "$@" 3>&-
}

