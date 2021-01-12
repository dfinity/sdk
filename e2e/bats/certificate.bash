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

    BACKEND=$(jq -r .networks.local.bind dfx.json)

    # Sometimes, something goes wrong with mitmdump's initialization.
    # It reports that it is listening, and the `nc` call succeeds,
    # but it does not actually respond to requests.
    #
    # This happens whether using a fixed port or a dynamic port.
    # For this reason, we retry initialization if `dfx ping` fails.
    #
    # I have seen this process take as many as 9 iterations to succeed,
    # so across a large number of CI runs, it could take even more.
    # The overall CI timeout will control the maximum time taken.

    while true
    do
        MITM_PORT=$(python3 ${BATS_TEST_DIRNAME}/utils/get_ephemeral_port.py)
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:$MITM_PORT\" dfx.json) >dfx.json

        mitmdump -p $MITM_PORT --mode reverse:http://$BACKEND  --replace '/~s/Hello,/Hullo,' &
        MITMDUMP_PID=$!

        timeout 5 sh -c \
            "until nc -z localhost $MITM_PORT; do echo waiting for mitmdump; sleep 1; done" \
            || (echo "mitmdump did not start on port $MITM_PORT" && exit 1)

        if timeout 10 dfx ping; then
            break
        fi

        kill -9 $MITMDUMP_PID
    done
}

teardown() {
    kill -9 $MITMDUMP_PID

    dfx_stop
}

@test "mitm attack - update: attack fails because certificate verification fails" {
    assert_command_fail dfx canister call certificate hello_update '("Buckaroo")'
    assert_match 'Certificate verification failed.'
}

@test "mitm attack - query: attack succeeds because there is no certificate to verify" {
    assert_command dfx canister call certificate hello_query '("Buckaroo")'
    assert_eq '("Hullo, Buckaroo!")'
}
