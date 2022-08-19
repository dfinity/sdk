#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "canister call and sign" {
    install_asset remote/call/actual
    dfx_start
    setup_actuallylocal_shared_network

    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice
    assert_command dfx canister call remote write '("initial data in the remote canister")' --identity alice --network actuallylocal
    assert_command dfx canister call remote read --identity alice --network actuallylocal
    assert_eq '("initial data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/call/mock
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json
    setup_actuallylocal_shared_network
    setup_local_shared_network

    # set up: remote method is update, local is query
    # call remote method as update to make a change
    assert_command dfx deploy --network actuallylocal
    assert_command dfx canister call remote which_am_i --network actuallylocal
    assert_eq '("actual")'

    cat dfx.json
    cat canister_ids.json

    #
    # Here the mock doesn't know about the method at all.
    # In order for the candid decoder to know the type, dfx must both:
    #   - look up the canister by (remote) canister id
    #   - use the remote candid definition
    #
    assert_command dfx canister call --query  "$REMOTE_CANISTER_ID" make_struct '("A query by principal", "B query by principal")' --network actuallylocal
    assert_eq '(record { a = "A query by principal"; b = "B query by principal" })'
    assert_command dfx canister call          "$REMOTE_CANISTER_ID" make_struct '("A default by principal", "B default by principal")' --network actuallylocal
    assert_eq '(record { a = "A default by principal"; b = "B default by principal" })'
    assert_command dfx canister call --update "$REMOTE_CANISTER_ID" make_struct '("A update by principal", "B update by principal")' --network actuallylocal
    assert_eq '(record { a = "A update by principal"; b = "B update by principal" })'

    assert_command dfx canister call --query  remote make_struct '("A query by name", "B query by name")' --network actuallylocal
    assert_eq '(record { a = "A query by name"; b = "B query by name" })'
    assert_command dfx canister call          remote make_struct '("A default by name", "B default by name")' --network actuallylocal
    assert_eq '(record { a = "A default by name"; b = "B default by name" })'
    assert_command dfx canister call --update remote make_struct '("A update by name", "B update by name")' --network actuallylocal
    assert_eq '(record { a = "A update by name"; b = "B update by name" })'

    # This also should work when no canister type can be determined / if no info but the bare minimum of remote id and remote candid is given:
    jq 'del(.canisters.remote.main)' dfx.json | sponge dfx.json
    assert_command dfx canister call --query  remote make_struct '("A query by name", "B query by name")' --network actuallylocal
    assert_eq '(record { a = "A query by name"; b = "B query by name" })'

    # We can't check this for sign, because dfx canister send outputs something like this:
    #   To see the content of response, copy-paste the encoded string into cbor.me.
    #   Response: d9d9f7a2667374617475736[snip]2696e636970616c

    #
    # Here:
    #   - the actual method is an update
    #   - the mock method is a query
    #   - in remote candid definition, the method is an update
    # We try to call with --query.
    # We expect dfx to notice that the method is in fact an update, which it knows from the remote candid definition.
    #
    assert_command_fail dfx canister call --query "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("call by principal with --query")' --network actuallylocal
    assert_match 'not a query method'
    assert_command_fail dfx canister call --query remote actual_update_mock_query_remote_candid_update '("call by name with --query")' --network actuallylocal
    assert_match 'not a query method'

    # And the same for dfx canister sign:
    assert_command_fail dfx canister sign --query "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("call by principal with --query")' --network actuallylocal
    assert_match 'not a query method'
    assert_command_fail dfx canister sign --query remote actual_update_mock_query_remote_candid_update '("call by name with --query")' --network actuallylocal
    assert_match 'not a query method'
}

@test "canister create <canister> fails for a remote canister" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_shared_network

    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_shared_network
    setup_local_shared_network
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json

    assert_command_fail dfx canister create remote --network actuallylocal
    assert_match "Canister 'remote' is a remote canister on network 'actuallylocal', and cannot be created from here."
}

@test "canister install <canister> fails for a remote canister" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_shared_network

    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_shared_network
    setup_local_shared_network
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json

    assert_command_fail dfx canister install remote --network actuallylocal
    assert_match "Canister 'remote' is a remote canister on network 'actuallylocal', and cannot be installed from here."
}

@test "canister create --all and canister install --all skip remote canisters" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_shared_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice
    assert_command dfx canister call remote write '("this is data in the remote canister")' --identity alice --network actuallylocal

    assert_command dfx canister call remote read --identity alice --network actuallylocal
    assert_eq '("this is data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_shared_network
    setup_local_shared_network
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json

    # Here we want to make sure that create+build+install works with the common flow
    assert_command dfx canister create --all
    assert_command dfx build
    assert_command dfx canister install --all

    assert_command dfx canister call basic read_remote
    assert_eq '("")'
    assert_command dfx canister call remote which_am_i
    assert_eq '("mock")'

    assert_command dfx canister create --all --network actuallylocal
    assert_command dfx build --network actuallylocal
    assert_command dfx canister install --all --network actuallylocal

    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("this is data in the remote canister")'

    # We can change the value by calling the original canister
    assert_command dfx canister call "${REMOTE_CANISTER_ID}" write '("altered data (by canister id) in the remote canister")' --network actuallylocal
    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("altered data (by canister id) in the remote canister")'

    # We can also call through the canister name
    assert_command dfx canister call remote write '("altered data (by canister name) in the remote canister")' --network actuallylocal
    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("altered data (by canister name) in the remote canister")'


    assert_command dfx canister call remote which_am_i --network actuallylocal
    assert_eq '("actual")'

    assert_command jq .basic.actuallylocal canister_ids.json
    assert_eq '"'"$(dfx canister id basic --network actuallylocal)"'"'

    assert_command jq .remote canister_ids.json
    assert_eq "null"
}

@test "for remote build, checks imports against the candid file rather than the mock" {
    # In this test, a canister with a remote ID also has a candid file specified.
    # When building for the remote network, we expect to use this candid file,
    # and for methods calls that don't match the candid file to fail.

    install_asset remote/actual
    dfx_start
    setup_actuallylocal_shared_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/extra
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json
    setup_actuallylocal_shared_network
    setup_local_shared_network

    # We expect the local network deploy to succeed, because it is built using the candid file from
    # the local canister.
    assert_command dfx deploy

    # And we can call the extra method,
    assert_command dfx canister call extra remote_extra
    assert_eq '("extra!")'

    # We expect the remote network deploy to fail, because it is built using the candid file
    # specified for the remote canister.  That candid file doesn't define the extra method
    # that is present in the mock.
    assert_command_fail dfx deploy --network actuallylocal
}

@test "build + install + call -- remote" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_shared_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx deploy --network actuallylocal --identity alice
    assert_command dfx canister call remote write '("this is data in the remote canister")' --network actuallylocal --identity alice

    assert_command dfx canister call remote read --network actuallylocal --identity alice
    assert_eq '("this is data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_shared_network
    setup_local_shared_network
    jq ".canisters.remote.remote.id.actuallylocal=\"$REMOTE_CANISTER_ID\"" dfx.json | sponge dfx.json

    assert_command dfx deploy
    assert_command dfx canister call basic read_remote
    assert_eq '("")'
    assert_command dfx canister call remote which_am_i
    assert_eq '("mock")'

    assert_command dfx deploy --network actuallylocal
    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("this is data in the remote canister")'

    # We can change the value by calling the original canister
    assert_command dfx canister call "${REMOTE_CANISTER_ID}" write '("data altered by canister id in the remote canister")' --network actuallylocal
    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("data altered by canister id in the remote canister")'

    # We can also call through the canister name
    assert_command dfx canister call remote write '("data altered by canister name in the remote canister")' --network actuallylocal
    assert_command dfx canister call basic read_remote --network actuallylocal
    assert_eq '("data altered by canister name in the remote canister")'


    assert_command dfx canister call remote which_am_i --network actuallylocal
    assert_eq '("actual")'

    assert_command jq .basic.actuallylocal canister_ids.json
    assert_eq '"'"$(dfx canister id basic --network actuallylocal)"'"'

    assert_command jq .remote canister_ids.json
    assert_eq "null"
}
