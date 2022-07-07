#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new certificate

    install_asset certificate
    dfx_start

    dfx deploy

    BACKEND="127.0.0.1:$(get_webserver_port)"

    # In github workflows, at the time of this writing, we get:
    #     macos-latest: mitmproxy 7.0.4
    #     ubuntu-latest: mitmproxy 4.x
    if [ "$(mitmdump --version | grep Mitmproxy | cut -d ' ' -f 2 | cut -c 1-2)" = "4." ]; then
        MODIFY_BODY_ARG="--replacements"
    else
        MODIFY_BODY_ARG="--modify-body"
    fi

    # Sometimes, something goes wrong with mitmdump's initialization.
    # It reports that it is listening, and the `nc` call succeeds,
    # but it does not actually respond to requests.
    #
    # This happens whether using a fixed port or a dynamic port.
    # For this reason, we retry initialization until `dfx ping` succeeds.
    #
    # I have seen this process take as many as 9 iterations to succeed,
    # so across a large number of CI runs, it could take even more.
    # The overall CI timeout will limit the maximum time taken.

    while true
    do
        MITM_PORT=$(python3 "${BATS_TEST_DIRNAME}/../utils/get_ephemeral_port.py")
        overwrite_webserver_port "$MITM_PORT"

        mitmdump -p "$MITM_PORT" --mode "reverse:http://$BACKEND"  "$MODIFY_BODY_ARG" '/~s/Hello,/Hullo,' &
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
    # Kill child processes of mitmdump. Otherwise they hang around way too long
    pkill -P $MITMDUMP_PID
    kill $MITMDUMP_PID

    dfx_stop

    standard_teardown
}

@test "mitm attack - update: attack fails because certificate verification fails" {
    assert_command_fail dfx canister call certificate hello_update '("Buckaroo")'
    assert_match 'Certificate verification failed.'
}

@test "mitm attack - query: attack succeeds because there is no certificate to verify" {
    # The wallet does not have a query call forward method (currently calls forward from wallet's update method)
    # So call with users Identity as sender here
    # There may need to be a query version of wallet_call
    assert_command dfx canister call certificate hello_query '("Buckaroo")'
    assert_eq '("Hullo, Buckaroo!")'
}
