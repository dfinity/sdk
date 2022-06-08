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
    skip
    install_asset remote/call/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal
    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_eq '("initial data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/call/mock
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.remote.remote.candid="remote.did"' dfx.json)" >dfx.json

    # set up: remote method is update, local is query
    # call remote method as update to make a change
    assert_command dfx deploy --network actuallylocal
    assert_command dfx canister --network actuallylocal call remote which_am_i
    assert_eq '("actual")'

    cat dfx.json
    cat canister_ids.json

    #
    # Here the mock doesn't know about the method at all.
    # In order for the candid decoder to know the type, dfx must both:
    #   - look up the canister by (remote) canister id
    #   - use the remote candid definition
    #
    assert_command dfx canister --network actuallylocal call --query  "$REMOTE_CANISTER_ID" make_struct '("A query by principal", "B query by principal")'
    assert_eq '(record { a = "A query by principal"; b = "B query by principal" })'
    assert_command dfx canister --network actuallylocal call          "$REMOTE_CANISTER_ID" make_struct '("A default by principal", "B default by principal")'
    assert_eq '(record { a = "A default by principal"; b = "B default by principal" })'
    assert_command dfx canister --network actuallylocal call --update "$REMOTE_CANISTER_ID" make_struct '("A update by principal", "B update by principal")'
    assert_eq '(record { a = "A update by principal"; b = "B update by principal" })'

    assert_command dfx canister --network actuallylocal call --query  remote make_struct '("A query by name", "B query by name")'
    assert_eq '(record { a = "A query by name"; b = "B query by name" })'
    assert_command dfx canister --network actuallylocal call          remote make_struct '("A default by name", "B default by name")'
    assert_eq '(record { a = "A default by name"; b = "B default by name" })'
    assert_command dfx canister --network actuallylocal call --update remote make_struct '("A update by name", "B update by name")'
    assert_eq '(record { a = "A update by name"; b = "B update by name" })'

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
    assert_command_fail dfx canister --network actuallylocal call --query "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("call by principal with --query")'
    assert_match 'not a query method'
    assert_command_fail dfx canister --network actuallylocal call --query remote actual_update_mock_query_remote_candid_update '("call by name with --query")'
    assert_match 'not a query method'

    # And the same for dfx canister sign:
    assert_command_fail dfx canister --network actuallylocal sign --query "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("call by principal with --query")'
    assert_match 'not a query method'
    assert_command_fail dfx canister --network actuallylocal sign --query remote actual_update_mock_query_remote_candid_update '("call by name with --query")'
    assert_match 'not a query method'
}

@test "canister create <canister> fails for a remote canister" {
    skip
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json

    assert_command_fail dfx canister --network actuallylocal create remote
    assert_match "remote" canister is remote on network actuallylocal and has canister id: "$REMOTE_CANISTER_ID"
}

@test "canister install <canister> fails for a remote canister" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json

    assert_command_fail dfx canister --network actuallylocal install remote
    assert_match "Canister 'remote' is a remote canister on network 'actuallylocal', and cannot be installed from here."
}

@test "canister create --all and canister install --all skip remote canisters" {
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal
    assert_command dfx --identity alice canister --network actuallylocal call remote write '("this is data in the remote canister")'

    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_eq '("this is data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json

    # Here we want to make sure that create+build+install works with the common flow
    assert_command dfx canister create --all
    assert_command dfx build
    assert_command dfx canister install --all

    assert_command dfx canister call basic read_remote
    assert_eq '("")'
    assert_command dfx canister call remote which_am_i
    assert_eq '("mock")'

    assert_command dfx canister --network actuallylocal create --all
    assert_command dfx build --network actuallylocal
    assert_command dfx canister --network actuallylocal install --all

    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("this is data in the remote canister")'

    # We can change the value by calling the original canister
    assert_command dfx canister --network actuallylocal call "${REMOTE_CANISTER_ID}" write '("altered data (by canister id) in the remote canister")'
    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("altered data (by canister id) in the remote canister")'

    # We can also call through the canister name
    assert_command dfx canister --network actuallylocal call remote write '("altered data (by canister name) in the remote canister")'
    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("altered data (by canister name) in the remote canister")'


    assert_command dfx canister --network actuallylocal call remote which_am_i
    assert_eq '("actual")'

    assert_command jq .basic.actuallylocal canister_ids.json
    assert_eq '"'"$(dfx canister --network actuallylocal id basic)"'"'

    assert_command jq .remote canister_ids.json
    assert_eq "null"
}

@test "for remote build, checks imports against the candid file rather than the mock" {
    # In this test, a canister with a remote ID also has a candid file specified.
    # When building for the remote network, we expect to use this candid file,
    # and for methods calls that don't match the candid file to fail.

    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/extra
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json

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
    setup_actuallylocal_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new --disable-encryption alice

    assert_command dfx --identity alice deploy --network actuallylocal
    assert_command dfx --identity alice canister --network actuallylocal call remote write '("this is data in the remote canister")'

    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_eq '("this is data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/basic
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json

    assert_command dfx deploy
    assert_command dfx canister call basic read_remote
    assert_eq '("")'
    assert_command dfx canister call remote which_am_i
    assert_eq '("mock")'

    assert_command dfx deploy --network actuallylocal
    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("this is data in the remote canister")'

    # We can change the value by calling the original canister
    assert_command dfx canister --network actuallylocal call "${REMOTE_CANISTER_ID}" write '("data altered by canister id in the remote canister")'
    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("data altered by canister id in the remote canister")'

    # We can also call through the canister name
    assert_command dfx canister --network actuallylocal call remote write '("data altered by canister name in the remote canister")'
    assert_command dfx canister --network actuallylocal call basic read_remote
    assert_eq '("data altered by canister name in the remote canister")'


    assert_command dfx canister --network actuallylocal call remote which_am_i
    assert_eq '("actual")'

    assert_command jq .basic.actuallylocal canister_ids.json
    assert_eq '"'"$(dfx canister --network actuallylocal id basic)"'"'

    assert_command jq .remote canister_ids.json
    assert_eq "null"
}
