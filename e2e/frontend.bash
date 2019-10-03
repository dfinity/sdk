#!/usr/bin/env bats

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx new e2e-project
    cd e2e-project
}

teardown() {
    # Kill the node manager, the dfx and the client. Ignore errors (ie. if processes aren't
    # running).
    killall dfx nodemanager client || true
}

# Creates a new project and starts its client in the background.
dfx_start() {
    # Bats create a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leaves a child process running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.
    dfx start --background $* 3>&-
}

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT=${BATS_TEST_DIRNAME}/assets/$1/
    cp -R $ASSET_ROOT/* .
}

@test "dfx start serves a frontend" {
    dfx_start

    run curl http://localhost:8000 # 8000 = default port.
    [[ $status == 0 ]]
    grep -i "<html>" <(echo $output)
}

@test "dfx start serves a frontend on a port" {
    dfx_start --host 127.0.0.1:12345

    run curl http://localhost:12345 # 8000 = default port.
    [[ $status == 0 ]]
    grep -i "<html>" <(echo $output)

    run curl http://localhost:8000
    [[ $status != 0 ]]  # This should have failed (no frontend on 8000).
}
