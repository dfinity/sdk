#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new certificate

    install_asset certificate
    dfx_start

    echo "dfx deploy..." >>${BATS_TEST_DIRNAME}/xx.log

    dfx deploy

    BACKEND=$(jq -r .networks.local.bind dfx.json)

    echo "trying to start mitmproxy..." >>${BATS_TEST_DIRNAME}/xx.log

    for i in $(seq 1 1000);
    do
        echo "attempt $i starts" >>${BATS_TEST_DIRNAME}/xx.log

        MITM_PORT=$(python3 ${BATS_TEST_DIRNAME}/utils/get_ephemeral_port.py)
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:$MITM_PORT\" dfx.json) >dfx.json
        #sleep 5
        pwd >${BATS_TEST_DIRNAME}/$MITM_PORT.pwd

        mitmdump -p $MITM_PORT -w ${BATS_TEST_DIRNAME}/$MITM_PORT.out --mode reverse:http://$BACKEND  --replace '/~s/Hello,/Hullo,' >${BATS_TEST_DIRNAME}/$MITM_PORT.txt 2>&1 &
        MITMDUMP_PID=$!

        #sleep 5
        timeout 5 sh -c \
            "until nc -z localhost $MITM_PORT; do echo waiting for mitmdump; sleep 1; done" \
            || (echo "mitmdump did not start on port $MITM_PORT" && exit 1)

        echo "  - mitmdump pid $MITMDUMP_PID listening on port $MITM_PORT" >>${BATS_TEST_DIRNAME}/xx.log

        # Sometimes, something goes wrong with mitmdump's initialization.  It reports that it is listening,
        # and the `nc` call above succeeds, but it does not actually respond to requests.
        # This happens whether using a fixed port or a dynamic port.
        # For this reason, we retry initialization if `dfx ping` fails.
        for p in $(seq 1 2);
        do
            echo "  - dfx ping ($p)..." >>${BATS_TEST_DIRNAME}/xx.log
            if timeout 10 dfx ping; then
                echo "    - succeeded on attempt $i (ping: $p)" >>${BATS_TEST_DIRNAME}/xx.log
                break 2
            fi
        done
        echo "    - failed" >>${BATS_TEST_DIRNAME}/xx.log

        kill -9 $MITMDUMP_PID
    done
}

teardown() {
    kill -9 $MITMDUMP_PID

    dfx_stop
}

@test "mitm attack - update: attack fails because certificate verification fails" {
    assert_command_fail dfx canister call certificate hello_update '("Banzai")'
    assert_match 'Certificate verification failed.'
}

@test "mitm attack - query: attack succeeds because there is no certificate to verify" {
    assert_command dfx canister call certificate hello_query '("Banzai")'
    assert_eq '("Hullo, Banzai!")'
}
