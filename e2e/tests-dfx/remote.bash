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

@test "canister call" {
  # exercise this code path:
  #    dfx canister call --query, by canister ID (of the remote canister), to an update call
  #    this means the correct query/update value will have to come from the candid file
  #    the call, as a query, would fail (because it's an update method)

    install_asset remote/call/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new alice

    assert_command dfx --identity alice deploy --network actuallylocal
    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
    assert_command dfx --identity alice canister --network actuallylocal call remote read
    #assert_eq '("initial data in the remote canister")'

    REMOTE_CANISTER_ID=$(jq -r .remote.actuallylocal canister_ids.json)
    echo "Remote canister id: $REMOTE_CANISTER_ID"
    rm canister_ids.json

    install_asset remote/call/mock
    setup_actuallylocal_network
    setup_local_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .canisters.remote.remote.id.actuallylocal=\""$REMOTE_CANISTER_ID"\" dfx.json)" >dfx.json
    cat <<<"$(jq '.canisters.remote.remote.candid="remote.did"' dfx.json)" >dfx.json

    # set up: remote method is update, local is query
    # call remote method as update to make a change
    assert_command dfx deploy --network actuallylocal
    assert_command dfx canister --network actuallylocal call remote which_am_i
    #assert_eq '("actual")'

    cat dfx.json
    cat canister_ids.json

    assert_command dfx canister --network actuallylocal call --query "$REMOTE_CANISTER_ID" make_struct '("A", "B")'
    # if looking up a remote canister by principal id did not work, the candid decoder would not know the field names.
    assert_eq '(record { a = "A"; b = "B" })' "$stdout"

    # if the remote.candid field is not used here, the call would fail, because the called method is actually an query
    # the actual method: an query
    # the local mock: a update
    # the remote/candid: a query
    # if remote.candid is not used: call fails (attempt to call query as an update)
    # if remote.candid is used: call succeeds (call query as query, per remote/candid)

    # If the lookup by principal id failed here, the method would be assumed to be an update
    # since the remote candid indicates the method is a query, it is instead called as a query; no change
    # this verifies that a remote canister can be looked up by canister id to find the method type.
    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
    assert_command dfx canister --network actuallylocal call --query  "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_query '("A0 call by principal with --query")'
    assert_match 'A0 call by principal with --query actual actual_query_mock_query_remote_candid_query'
    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_match 'initial data in the remote canister'

    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
    assert_command dfx canister --network actuallylocal call          "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_query '("A0 call by principal with default")'
    assert_match 'A0 call by principal with default actual actual_query_mock_query_remote_candid_query'
    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_match 'initial data in the remote canister'

    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
    assert_command dfx canister --network actuallylocal call --update "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_query '("A0 call by principal with --update")'
    assert_match 'A0 call by principal with --update actual actual_query_mock_query_remote_candid_query'
    assert_command dfx --identity alice canister --network actuallylocal call remote read
    assert_match 'initial data in the remote canister'

#    return 0
#    ###
#    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
#    assert_command dfx canister --network actuallylocal call --query  "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_update '("A1 call by principal with --query")'
#    assert_match 'A1 call by principal with --query actual actual_query_mock_query_remote_candid_update'
#    assert_command dfx --identity alice canister --network actuallylocal call remote read
#    assert_match '("initial data in the remote canister")'
#
#    # should pick update from the remote candid, call the query function as an update, actually make a change
#    assert_command dfx --identity alice canister --network actuallylocal call remote write '("initial data in the remote canister")'
#    assert_command dfx canister --network actuallylocal call          "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_update '("A2 call by principal with --query")'
#    assert_command dfx --identity alice canister --network actuallylocal call remote read
#    assert_match '("initial data in the remote canister")'
#
#    return 0
#
#    assert_command_fail dfx canister --network actuallylocal call --update "$REMOTE_CANISTER_ID" actual_query_mock_query_remote_candid_update '("A3 call by principal with --query")'
#
#    assert_command_fail dfx canister --network actuallylocal call --query  remote_mo actual_query_mock_query_remote_candid_update '("A1 call by name with --query")'
#
#    assert_command_fail dfx canister --network actuallylocal call          remote_mo actual_query_mock_query_remote_candid_update '("A2 call by name with --query")'
#
#    assert_command_fail dfx canister --network actuallylocal call --update remote_mo actual_query_mock_query_remote_candid_update '("A3 call by name with --query")'
#
#
#
#    # can't call an update method with --query
#    assert_command_fail dfx canister --network actuallylocal call --query "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("A call by principal with --query")'
#    assert_eq 'Error: Invalid method call: actual_update_mock_query_remote_candid_update is not a query method.'
#
#    assert_command dfx canister --network actuallylocal call "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("B call by principal with defaul)")'
#    assert_eq "call by principal (default) actual actual_update_mock_query_remote_candid_update"
#    assert_command dfx canister --network actuallylocal call --update "$REMOTE_CANISTER_ID" actual_update_mock_query_remote_candid_update '("C call by principal with --update")'
#    assert_eq "Y"
#
#    # can't call an update method with --query
#    assert_command_fail dfx canister --network actuallylocal call --query remote actual_update_mock_query_remote_candid_update '("D call by name with --query")'
#    assert_eq 'Error: Invalid method call: actual_update_mock_query_remote_candid_update is not a query method.'
#
#    assert_command dfx canister --network actuallylocal call remote actual_update_mock_query_remote_candid_update '("E call by name with default")'
#    assert_eq "Z"
#    assert_command dfx canister --network actuallylocal call --update remote actual_update_mock_query_remote_candid_update '("F call by name with --update")'
#    assert_eq "W"
#
}

@test "canister sign" {
  skip
  echo
}

@test "canister create <canister> fails for a remote canister" {
  skip
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new alice

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
  skip
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    dfx identity new alice

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
    assert_match "Error: Canister 'remote' is a remote canister on network 'actuallylocal', and cannot be installed from here."
}

@test "canister create --all and canister install --all skip remote canisters" {
  skip
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new alice

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
  skip
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
    dfx identity new alice

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
  skip
    install_asset remote/actual
    dfx_start
    setup_actuallylocal_network

    #
    # Set up the "remote" canister, with a different controller in order to
    # demonstrate that we don't try to install/upgrade it as a remote canister.
    #
    dfx identity new alice

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
