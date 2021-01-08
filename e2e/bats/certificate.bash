#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new certificate

    install_asset certificate
    dfx_start
    dfx deploy

    echo "Initial local.bind is $(jq .networks.local.bind dfx.json)"
    jq . dfx.json

    BACKEND=$(jq -r .networks.local.bind dfx.json)
    echo "backend is $BACKEND"

    MITM_PORT=$(python3 ${BATS_TEST_DIRNAME}/utils/get_ephemeral_port.py)

    mitmdump -p $MITM_PORT --mode reverse:http://$BACKEND  --replace '/~s/Hello,/Hullo,' &
    MITMDUMP_PID=$!
    #sleep 5

    timeout 5 sh -c \
        "until nc -z localhost $MITM_PORT; do echo waiting for mitmdump; sleep 1; done" \
        || (echo "mitmdump did not start on port $MITM_PORT" && exit 1)


    cat <<<$(jq .networks.local.bind=\"127.0.0.1:$MITM_PORT\" dfx.json) >dfx.json
    echo "MITM local.bind is $(jq .networks.local.bind dfx.json)"
}

teardown() {
    kill -9 $MITMDUMP_PID

    dfx_stop
}

@test "mitm attack - update: certificate verification fails" {
    assert_command_fail dfx canister call certificate hello_update '("Banzai")'
    assert_match 'Certificate verification failed.'
}

@test "mitm attack - query: no certificate verification" {
    assert_command dfx canister call certificate hello_query '("Banzai")'
    assert_eq '("Hullo, Banzai!")'
}
