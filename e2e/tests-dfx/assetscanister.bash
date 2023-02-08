#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

create_batch() {
    reg="batch_id = ([0-9]*) : nat"
    assert_command      dfx canister call e2e_project_frontend create_batch '(record { })'
    # shellcheck disable=SC2154
    [[ "$stdout" =~ $reg ]]
    BATCH_ID="${BASH_REMATCH[1]}"
    echo "$BATCH_ID"
}

check_permission_failure() {
    expected="$1"
    # Why are these different? https://dfinity.atlassian.net/browse/SDK-955 will find out.
    if [ "$USE_IC_REF" ]
    then
        assert_contains "canister did not respond"
    else
        assert_contains "$expected"
    fi
}

@test "validation methods" {
  assert_command dfx identity new controller --storage-mode plaintext
  assert_command dfx identity use controller
  CONTROLLER_PRINCIPAL=$(dfx identity get-principal)

  install_asset assetscanister
  dfx_start
  assert_command dfx deploy

  assert_command dfx identity new prepare --storage-mode plaintext
  PREPARE_PRINCIPAL=$(dfx identity get-principal --identity prepare)

  assert_command dfx canister call e2e_project_frontend validate_grant_permission "(record { to_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { Prepare }; })"
  assert_contains 'Ok = "grant Prepare permission to principal '"$PREPARE_PRINCIPAL"'"'

  assert_command dfx canister call e2e_project_frontend validate_grant_permission "(record { to_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { Prepare }; })" --identity prepare
  assert_contains 'Ok = "grant Prepare permission to principal '"$PREPARE_PRINCIPAL"'"'

  assert_command dfx canister call e2e_project_frontend validate_revoke_permission "(record { of_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { Commit }; })"
  assert_contains 'Ok = "revoke Commit permission from principal '"$PREPARE_PRINCIPAL"'"'

  FE_CANISTER_ID="$(dfx canister id e2e_project_frontend)"
  rm .dfx/local/canister_ids.json
  assert_command_fail dfx canister call "$FE_CANISTER_ID" validate_revoke_permission "(record { of_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { FlyBeFree }; })"
  assert_contains "trapped"
}

@test "access control - fine-grained" {
  assert_command dfx identity new controller --storage-mode plaintext
  assert_command dfx identity use controller
  CONTROLLER_PRINCIPAL=$(dfx identity get-principal)

  assert_command dfx identity new non-permissioned-controller --storage-mode plaintext

  assert_command dfx identity new commit --storage-mode plaintext
  assert_command dfx identity new manage-permissions --storage-mode plaintext
  assert_command dfx identity new no-permissions --storage-mode plaintext
  assert_command dfx identity new prepare --storage-mode plaintext

  PREPARE_PRINCIPAL=$(dfx identity get-principal --identity prepare)
  COMMIT_PRINCIPAL=$(dfx identity get-principal --identity commit)
  MANAGE_PERMISSIONS_PRINCIPAL=$(dfx identity get-principal --identity manage-permissions)

  install_asset assetscanister
  dfx_start
  assert_command dfx deploy

  assert_command dfx canister update-settings e2e_project_frontend --add-controller non-permissioned-controller

  # initialization: the deploying controller has Commit permissions, no one else has permissions
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { ManagePermissions }; })'
  assert_eq "(vec {})"
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Commit }; })'
  assert_eq "(
  vec {
    principal \"$CONTROLLER_PRINCIPAL\";
  },
)"
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Prepare }; })'
  assert_eq "(vec {})"

  # granting permissions

  # users without any permissions cannot grant
  assert_command_fail dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$MANAGE_PERMISSIONS_PRINCIPAL\"; permission = variant { ManagePermissions }; })" --identity no-permissions
  check_permission_failure "Caller does not have ManagePermissions permission and is not a controller"
  # anonymous cannot grant
  assert_command_fail dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$MANAGE_PERMISSIONS_PRINCIPAL\"; permission = variant { ManagePermissions }; })" --identity anonymous
  check_permission_failure "Caller does not have ManagePermissions permission and is not a controller"

  # controllers: can grant and revoke permissions
  # first, with the controller that created the canister
  assert_command dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$MANAGE_PERMISSIONS_PRINCIPAL\"; permission = variant { ManagePermissions }; })"
  # controller without any perms can too
  assert_command dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { Prepare }; })" --identity non-permissioned-controller
  # principal with only ManagePermissions can
  assert_command dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$COMMIT_PRINCIPAL\"; permission = variant { Commit }; })" --identity manage-permissions

  # now make sure access came out as expected
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { ManagePermissions }; })'
  assert_eq "(
  vec {
    principal \"$MANAGE_PERMISSIONS_PRINCIPAL\";
  },
)"
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Commit }; })'
  assert_contains "$CONTROLLER_PRINCIPAL"
  assert_contains "$COMMIT_PRINCIPAL"

  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Prepare }; })'
  assert_eq "(
  vec {
    principal \"$PREPARE_PRINCIPAL\";
  },
)"

  # create batch
  assert_command      dfx canister call e2e_project_frontend create_batch '(record { })'
  assert_command      dfx canister call e2e_project_frontend create_batch '(record { })' --identity commit
  assert_command      dfx canister call e2e_project_frontend create_batch '(record { })' --identity prepare
  assert_command_fail dfx canister call e2e_project_frontend create_batch '(record { })' --identity manage-permissions
  assert_contains "Caller does not have Prepare permission"
  assert_command_fail dfx canister call e2e_project_frontend create_batch '(record { })' --identity no-permissions
  assert_contains "Caller does not have Prepare permission"
  assert_command_fail dfx canister call e2e_project_frontend create_batch '(record { })' --identity anonymous
  assert_contains "Caller does not have Prepare permission"

  # create chunk
  BATCH_ID="$(create_batch)"

  echo "batch id is $BATCH_ID"
  args="(record { batch_id=$BATCH_ID; content=blob \"the content\"})"
  assert_command      dfx canister call e2e_project_frontend create_chunk "$args"
  assert_command      dfx canister call e2e_project_frontend create_chunk "$args" --identity commit
  assert_command      dfx canister call e2e_project_frontend create_chunk "$args" --identity prepare
  assert_command_fail dfx canister call e2e_project_frontend create_chunk "$args" --identity manage-permissions
  assert_contains "Caller does not have Prepare permission"
  assert_command_fail dfx canister call e2e_project_frontend create_chunk "$args" --identity no-permissions
  assert_contains "Caller does not have Prepare permission"
  assert_command_fail dfx canister call e2e_project_frontend create_chunk "$args" --identity anonymous
  assert_contains "Caller does not have Prepare permission"

  # create_asset
  args='(record { key="/a.txt"; content_type="text/plain" })'
  assert_command      dfx canister call e2e_project_frontend create_asset "$args"
  assert_command      dfx canister call e2e_project_frontend create_asset "$args" --identity commit
  assert_command_fail dfx canister call e2e_project_frontend create_asset "$args" --identity prepare
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend create_asset "$args" --identity manage-permissions
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend create_asset "$args" --identity no-permissions
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend create_asset "$args" --identity anonymous
  assert_contains "Caller does not have Commit permission"

  # commit_batch
  BATCH_ID="$(create_batch)"
  args="(record { batch_id=$BATCH_ID; operations=vec{} })"
  assert_command      dfx canister call e2e_project_frontend commit_batch "$args"
  BATCH_ID="$(create_batch)"
  args="(record { batch_id=$BATCH_ID; operations=vec{} })"
  assert_command      dfx canister call e2e_project_frontend commit_batch "$args" --identity commit
  assert_command_fail dfx canister call e2e_project_frontend commit_batch "$args" --identity prepare
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend commit_batch "$args" --identity manage-permissions
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend commit_batch "$args" --identity no-permissions
  assert_contains "Caller does not have Commit permission"
  assert_command_fail dfx canister call e2e_project_frontend commit_batch "$args" --identity anonymous
  assert_contains "Caller does not have Commit permission"


  # revoking permissions

  assert_command      dfx canister call e2e_project_frontend create_batch '(record { })' --identity commit
  # controller w/o permissions can revoke
  assert_command      dfx canister call e2e_project_frontend revoke_permission  "(record { of_principal=principal \"$COMMIT_PRINCIPAL\"; permission = variant { Commit }; })" --identity non-permissioned-controller
  assert_command_fail dfx canister call e2e_project_frontend create_batch '(record { })' --identity commit
  assert_contains "Caller does not have Prepare permission"
  assert_command_fail dfx canister call e2e_project_frontend commit_batch "$args" --identity commit
  assert_contains "Caller does not have Commit permission"

  assert_command      dfx canister call e2e_project_frontend create_batch '(record { })' --identity prepare
  # principal with only ManagePermissions can revokje
  assert_command      dfx canister call e2e_project_frontend revoke_permission  "(record { of_principal=principal \"$PREPARE_PRINCIPAL\"; permission = variant { Prepare }; })" --identity manage-permissions
  assert_command_fail dfx canister call e2e_project_frontend create_batch '(record { })' --identity prepare
  assert_contains "Caller does not have Prepare permission"

  # can revoke your own permissions even without ManagePermissions
  assert_command      dfx canister call e2e_project_frontend grant_permission "(record { to_principal=principal \"$COMMIT_PRINCIPAL\"; permission = variant { Commit }; })" --identity manage-permissions
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Commit }; })'
  assert_contains "$COMMIT_PRINCIPAL"
  assert_command      dfx canister call e2e_project_frontend revoke_permission  "(record { of_principal=principal \"$COMMIT_PRINCIPAL\"; permission = variant { Commit }; })" --identity commit
  assert_command dfx canister call e2e_project_frontend list_permitted '(record { permission = variant { Commit }; })'
  assert_not_contains "$COMMIT_PRINCIPAL"


}

@test "take ownership" {
  assert_command dfx identity new controller --storage-mode plaintext
  assert_command dfx identity use controller
  CONTROLLER_PRINCIPAL=$(dfx identity get-principal)

  assert_command dfx identity new authorized1 --storage-mode plaintext
  AUTHORIZED1_PRINCIPAL=$(dfx identity get-principal --identity authorized1)

  assert_command dfx identity new authorized2 --storage-mode plaintext
  AUTHORIZED2_PRINCIPAL=$(dfx identity get-principal --identity authorized2)

  install_asset assetscanister
  dfx_start
  assert_command dfx deploy

  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$AUTHORIZED1_PRINCIPAL\")"
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$AUTHORIZED2_PRINCIPAL\")"

  assert_command dfx canister call e2e_project_frontend deauthorize "(principal \"$CONTROLLER_PRINCIPAL\")"
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$CONTROLLER_PRINCIPAL"
  assert_contains "$AUTHORIZED1_PRINCIPAL"
  assert_contains "$AUTHORIZED2_PRINCIPAL"

  # authorized cannot take ownership
  assert_command_fail dfx canister call e2e_project_frontend take_ownership "()" --identity authorized1
  assert_command_fail dfx canister call e2e_project_frontend take_ownership "()" --identity authorized2
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$CONTROLLER_PRINCIPAL"
  assert_contains "$AUTHORIZED1_PRINCIPAL"
  assert_contains "$AUTHORIZED2_PRINCIPAL"

  # controller can take ownership
  assert_command dfx canister call e2e_project_frontend take_ownership "()"

  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$CONTROLLER_PRINCIPAL"
  assert_not_contains "$AUTHORIZED1_PRINCIPAL"
  assert_not_contains "$AUTHORIZED2_PRINCIPAL"
}

@test "authorize and deauthorize work as expected" {
  assert_command dfx identity new controller --storage-mode plaintext
  assert_command dfx identity use controller
  CONTROLLER_PRINCIPAL=$(dfx identity get-principal)
  assert_command dfx identity new authorized --storage-mode plaintext
  AUTHORIZED_PRINCIPAL=$(dfx identity get-principal --identity authorized)
  assert_command dfx identity new backdoor --storage-mode plaintext
  BACKDOOR_PRINCIPAL=$(dfx identity get-principal --identity backdoor)
  assert_command dfx identity new stranger --storage-mode plaintext
  assert_command dfx identity use stranger
  STRANGER_PRINCIPAL=$(dfx identity get-principal)
  assert_command dfx identity use controller

  install_asset assetscanister
  dfx_start
  assert_command dfx deploy

  # deployer is automatically authorized
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$CONTROLLER_PRINCIPAL"

  # non-controller is not allowed to deauthorize principals
  assert_command dfx identity use stranger
  assert_command_fail dfx canister call e2e_project_frontend deauthorize "(principal \"$CONTROLLER_PRINCIPAL\")"

  # authorized user can deauthorize
  assert_command dfx identity use controller
  assert_command dfx canister call e2e_project_frontend deauthorize "(principal \"$CONTROLLER_PRINCIPAL\")"
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$CONTROLLER_PRINCIPAL"

  # while not authorized, dfx deploy fails, even as controller
  echo "new file content" > 'src/e2e_project_frontend/assets/new_file.txt'
  assert_command_fail dfx deploy

  # canister controllers may always authorize principals, even if they're not authorized themselves
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$STRANGER_PRINCIPAL\")"
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$STRANGER_PRINCIPAL"

  # authorized principals, that are not controllers, cannot authorize other principals
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$AUTHORIZED_PRINCIPAL\")" --identity controller
  assert_command_fail dfx canister call e2e_project_frontend authorize "(principal \"$BACKDOOR_PRINCIPAL\")" --identity authorized

  check_permission_failure "Caller does not have ManagePermissions permission and is not a controller."

  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$BACKDOOR_PRINCIPAL"

  # authorized principals, that are not controllers, cannot deauthorize others
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$BACKDOOR_PRINCIPAL\")" --identity controller
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$BACKDOOR_PRINCIPAL"
  assert_command_fail dfx canister call e2e_project_frontend deauthorize "(principal \"$BACKDOOR_PRINCIPAL\")" --identity authorized

  check_permission_failure "Caller is not a controller"

  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$BACKDOOR_PRINCIPAL"

  # authorized principals, that are not controllers, can deauthorize themselves
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_contains "$AUTHORIZED_PRINCIPAL"
  assert_command dfx canister call e2e_project_frontend deauthorize "(principal \"$AUTHORIZED_PRINCIPAL\")" --identity authorized
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$AUTHORIZED_PRINCIPAL"

  # canister controller may always deauthorize, even if they're not authorized themselves
  assert_command dfx canister call e2e_project_frontend deauthorize "(principal \"$STRANGER_PRINCIPAL\")"
  assert_command dfx canister call e2e_project_frontend list_authorized '()'
  assert_not_contains "$STRANGER_PRINCIPAL"

  # after authorizing, deploy works again, even for non-controller
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$STRANGER_PRINCIPAL\")"
  assert_command dfx identity use stranger
  assert_command dfx deploy
}

@test "http_request percent-decodes urls" {
    install_asset assetscanister

    dfx_start

    echo "contents of file with space in filename" >'src/e2e_project_frontend/assets/filename with space.txt'
    echo "contents of file with plus in filename" >'src/e2e_project_frontend/assets/has+plus.txt'
    echo "contents of file with percent in filename" >'src/e2e_project_frontend/assets/has%percent.txt'
    echo "filename is an ae symbol" >'src/e2e_project_frontend/assets/æ'
    echo "filename is percent symbol" >'src/e2e_project_frontend/assets/%'
    echo "filename contains question mark" >'src/e2e_project_frontend/assets/filename?withqmark.txt'
    dd if=/dev/urandom of='src/e2e_project_frontend/assets/large with spaces.bin' bs=2500000 count=1


    dfx deploy

    # decode as expected
    assert_command dfx canister  call --query e2e_project_frontend http_request '(record{url="/filename%20with%20space.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with space in filename"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/has%2bplus.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with plus in filename"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/has%2Bplus.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with plus in filename"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/has%%percent.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with percent in filename"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/%e6";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename is an ae symbol" # candid looks like blob "filename is \c3\a6\0a"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/%%";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename is percent"
     # this test ensures url decoding happens after removing the query string
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/filename%3fwithqmark.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename contains question mark"

    # these error conditions can't be tested with curl, because something responds first with Bad Request.
    # THESE TESTS WERE REMOVED BECAUSE THE RUST CANISTER DOES NOT SUPPORT REJECTING MESSAGES
    # TODO: Reenable those tests.
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: % must be followed by '%' or two hex digits"
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%z";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: % must be followed by two hex digits, but only one was found"
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%zz";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: neither character after % is a hex digit"
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%e";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: % must be followed by two hex digits, but only one was found"
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%g6";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: first character after % is not a hex digit"
    #    assert_command_fail dfx canister call --query e2e_project_frontend http_request '(record{url="/%ch";headers=vec{};method="GET";body=vec{}})'
    #    assert_match "error decoding url: second character after % is not a hex digit"

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl --fail -vv http://localhost:"$PORT"/filename%20with%20space.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "200 OK" "$stderr"
    assert_match "contents of file with space in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/has%2bplus.txt?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "contents of file with plus in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/has%%percent.txt?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "contents of file with percent in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/%e6?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "filename is an ae symbol"

    assert_command curl --fail -vv http://localhost:"$PORT"/%%?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "filename is percent symbol"

    assert_command curl --fail -vv http://localhost:"$PORT"/filename%3fwithqmark.txt?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "filename contains question mark"

    assert_command curl --fail -vv --output lws-curl-output.bin "http://localhost:$PORT/large%20with%20spaces.bin?canisterId=$ID"
    diff 'src/e2e_project_frontend/assets/large with spaces.bin' lws-curl-output.bin

    # curl now reports "curl: (3) URL using bad/illegal format or missing URL" so we cannot verify behavior
    # assert_command_fail curl --fail -vv --path-as-is http://localhost:"$PORT"/'filename with space'.txt?canisterId="$ID"
    # assert_match "400 Bad Request" "$stderr"
}

@test "generates gzipped content encoding for .js files" {
    install_asset assetscanister
    for i in $(seq 1 400); do
      echo "some easily duplicate text $i" >>src/e2e_project_frontend/assets/notreally.js
    done

    dfx_start
    assert_command dfx deploy
    dfx canister call --query e2e_project_frontend list '(record{})'

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl -v --output not-compressed http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_not_match "content-encoding:"
    diff not-compressed src/e2e_project_frontend/assets/notreally.js

    assert_command curl -v --output encoded-compressed-1.gz -H "Accept-Encoding: gzip" http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_match "content-encoding: gzip"
    gunzip encoded-compressed-1.gz
    diff encoded-compressed-1 src/e2e_project_frontend/assets/notreally.js

    # should split up accept-encoding lines with more than one encoding
    assert_command curl -v --output encoded-compressed-2.gz -H "Accept-Encoding: gzip, deflate, br" http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_match "content-encoding: gzip"
    gunzip encoded-compressed-2.gz
    diff encoded-compressed-2 src/e2e_project_frontend/assets/notreally.js
}

@test "leaves in place files that were already installed" {
    install_asset assetscanister
    dd if=/dev/urandom of=src/e2e_project_frontend/assets/asset1.bin bs=400000 count=1
    dd if=/dev/urandom of=src/e2e_project_frontend/assets/asset2.bin bs=400000 count=1

    dfx_start
    assert_command dfx deploy

    assert_match '/asset1.bin 1/1'
    assert_match '/asset2.bin 1/1'

    dd if=/dev/urandom of=src/e2e_project_frontend/assets/asset2.bin bs=400000 count=1

    assert_command dfx deploy
    assert_match '/asset1.bin.*is already installed'
    assert_match '/asset2.bin 1/1'
}

@test "unsets asset encodings that are removed from project" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister call --update e2e_project_frontend store '(record{key="/sample-asset.txt"; content_type="text/plain"; content_encoding="arbitrary"; content=blob "content encoded in another way!"})'

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'

    dfx deploy

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'
}

@test "verifies sha256, if specified" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/text-with-newlines.txt";accept_encodings=vec{"identity"}})'

    assert_command dfx canister call --query e2e_project_frontend get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=opt vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister call --query e2e_project_frontend get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister call --query e2e_project_frontend get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=opt vec { 88; 87; 86; }})'
    assert_match 'sha256 mismatch'

    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=opt vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=opt vec { 88; 87; 86; }})'
    assert_match 'sha256 mismatch'

}

@test "can store and retrieve assets by key" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_frontend

    assert_command dfx canister call --query e2e_project_frontend retrieve '("/binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01 \80\0aw12 \00xy\0aKL\0b\0ajk")'

    assert_command dfx canister call --query e2e_project_frontend retrieve '("/text-with-newlines.txt")' --output idl
    assert_eq '(blob "cherries\0ait\27s cherry season\0aCHERRIES")'

    assert_command dfx canister call --update e2e_project_frontend store '(record{key="AA"; content_type="text/plain"; content_encoding="identity"; content=blob "hello, world!"})'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_frontend retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command dfx canister call --query e2e_project_frontend retrieve '("AA")' --output idl
    assert_eq '(blob "hello, world!")'

    assert_command dfx canister call --query e2e_project_frontend retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command_fail dfx canister call --query e2e_project_frontend retrieve '("C")'
}

@test "asset canister supports http requests" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_frontend

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)
    assert_command curl http://localhost:"$PORT"/text-with-newlines.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_eq "cherries
it's cherry season
CHERRIES" "$stdout"
}

@test 'can store arbitrarily large files' {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref" # this takes too long for ic-ref's wasm interpreter

    install_asset assetscanister
    dd if=/dev/urandom of=src/e2e_project_frontend/assets/large-asset.bin bs=1000000 count=6

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_frontend

    # retrieve() refuses to serve just part of an asset
    assert_command_fail dfx canister call --query e2e_project_frontend retrieve '("/large-asset.bin")'
    assert_match 'Asset too large.'

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/large-asset.bin";accept_encodings=vec{"identity"}})'
    assert_match 'total_length = 6_000_000'
    assert_match 'content_type = "application/octet-stream"'
    assert_match 'content_encoding = "identity"'

    assert_command dfx canister call --query e2e_project_frontend get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=2})'

    assert_command dfx canister call --query e2e_project_frontend get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=3})'
    assert_command_fail dfx canister call --query e2e_project_frontend get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=4})'

    PORT=$(get_webserver_port)
    CANISTER_ID=$(dfx canister id e2e_project_frontend)
    curl -v --output curl-output.bin "http://localhost:$PORT/large-asset.bin?canisterId=$CANISTER_ID"
    diff src/e2e_project_frontend/assets/large-asset.bin curl-output.bin
}

@test "list() return assets" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_frontend

    assert_command dfx canister call --query e2e_project_frontend list '(record{})'
    assert_match '"/binary/noise.txt"'
    assert_match 'length = 19'
    assert_match '"/text-with-newlines.txt"'
    assert_match 'length = 36'
    assert_match '"/sample-asset.txt"'
    assert_match 'length = 24'
}

@test "identifies content type" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all

    touch src/e2e_project_frontend/assets/index.html
    touch src/e2e_project_frontend/assets/logo.png
    touch src/e2e_project_frontend/assets/index.js
    touch src/e2e_project_frontend/assets/main.css
    touch src/e2e_project_frontend/assets/index.js.map
    touch src/e2e_project_frontend/assets/index.js.LICENSE.txt
    touch src/e2e_project_frontend/assets/index.js.LICENSE

    dfx build
    dfx canister install e2e_project_frontend

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/index.html";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/html"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/logo.png";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "image/png"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/index.js";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "application/javascript"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/main.css";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/css"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/index.js.map";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/index.js.LICENSE.txt";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/index.js.LICENSE";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "application/octet-stream"'
}

@test "deletes assets that are removed from project" {
    install_asset assetscanister

    dfx_start

    touch src/e2e_project_frontend/assets/will-delete-this.txt
    dfx deploy

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend list  '(record{})'
    assert_match '"/will-delete-this.txt"'

    rm src/e2e_project_frontend/assets/will-delete-this.txt
    dfx deploy

    assert_command_fail dfx canister call --query e2e_project_frontend get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend list  '(record{})'
    assert_not_match '"/will-delete-this.txt"'
}

@test "asset configuration via .ic-assets.json5" {
    install_asset assetscanister

    dfx_start

    touch src/e2e_project_frontend/assets/ignored.txt
    touch src/e2e_project_frontend/assets/index.html
    touch src/e2e_project_frontend/assets/.hidden.txt

    mkdir src/e2e_project_frontend/assets/.well-known
    touch src/e2e_project_frontend/assets/.well-known/thing.json
    touch src/e2e_project_frontend/assets/.well-known/file.txt

    echo '[
      {
        "match": "ignored.txt",
        "ignore": true
      },
      {
        "match": "*",
        "cache": {
          "max_age": 500
        },
        "headers": {
          "x-header": "x-value"
        }
      },
      {
        "match": ".*",
        "ignore": false,
        "cache": {
          "max_age": 888
        },
        "headers": {
          "x-extra-header": "x-extra-value"
        }
      },
      {
        "match": "ignored.txt",
        "ignore": true
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5
    echo '[
      {
        "match": "*",
        "headers": {
          "x-well-known-header": "x-well-known-value"
        }
      },
      {
        "match": "*.json",
        "cache": {
          "max_age": 1000
        }
      },
      {
        "match": "file.txt",
        "headers": null
      }
    ]' > src/e2e_project_frontend/assets/.well-known/.ic-assets.json5

    dfx deploy

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl --head "http://localhost:$PORT/.well-known/thing.json?canisterId=$ID"
    assert_match "x-extra-header: x-extra-value"
    assert_match "x-header: x-value"
    assert_match "x-well-known-header: x-well-known-value"
    assert_match "cache-control: max-age=1000"

    assert_command curl --head "http://localhost:$PORT/.well-known/file.txt?canisterId=$ID"
    assert_match "cache-control: max-age=888"
    assert_not_match "x-well-known-header: x-well-known-value"
    assert_not_match "x-header: x-value"
    assert_not_match "x-extra-header: x-extra-value"

    assert_command curl --head "http://localhost:$PORT/index.html?canisterId=$ID"
    assert_match "cache-control: max-age=500"
    assert_match "x-header: x-value"
    assert_not_match "x-extra-header: x-extra-value"

    assert_command curl --head "http://localhost:$PORT/.hidden.txt?canisterId=$ID"
    assert_match "cache-control: max-age=888"
    assert_match "x-header: x-value"
    assert_match "x-extra-header: x-extra-value"

    # assert_command curl -vv "http://localhost:$PORT/ignored.txt?canisterId=$ID"
    # assert_match "404 Not Found"
    # from logs:
    # Staging contents of new and changed assets:
    #   /sample-asset.txt 1/1 (24 bytes)
    #   /text-with-newlines.txt 1/1 (36 bytes)
    #   /.well-known/file.txt 1/1 (0 bytes)
    #   /index.html 1/1 (0 bytes)
    #   /.hidden.txt 1/1 (0 bytes)
    #   /binary/noise.txt 1/1 (19 bytes)
    #   /.well-known/thing.json 1/1 (0 bytes)
}

@test "asset configuration via .ic-assets.json5 - nested dot directories" {
    install_asset assetscanister

    dfx_start

    touch src/e2e_project_frontend/assets/thing.json
    touch src/e2e_project_frontend/assets/.ignored-by-defualt.txt

    mkdir src/e2e_project_frontend/assets/.well-known
    touch src/e2e_project_frontend/assets/.well-known/thing.json

    mkdir src/e2e_project_frontend/assets/.well-known/.hidden
    touch src/e2e_project_frontend/assets/.well-known/.hidden/ignored.txt

    mkdir src/e2e_project_frontend/assets/.well-known/.another-hidden
    touch src/e2e_project_frontend/assets/.well-known/.another-hidden/ignored.txt

    echo '[
      {
        "match": ".well-known",
        "ignore": false
      },
      {
        "match": "**/*",
        "cache": { "max_age": 2000 }
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5
    echo '[
      {
        "match": "*",
        "headers": {
          "x-header": "x-value"
        }
      },
      {
        "match": ".hidden",
        "ignore": true
      }
    ]' > src/e2e_project_frontend/assets/.well-known/.ic-assets.json5
    echo '[
      {
        "match": "*",
        "ignore": false
      }
    ]' > src/e2e_project_frontend/assets/.well-known/.hidden/.ic-assets.json5
    echo '[
      {
        "match": "*",
        "ignore": false
      }
    ]' > src/e2e_project_frontend/assets/.well-known/.another-hidden/.ic-assets.json5

    dfx deploy

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl --head "http://localhost:$PORT/thing.json?canisterId=$ID"
    assert_match "cache-control: max-age=2000"
    assert_command curl --head "http://localhost:$PORT/.well-known/thing.json?canisterId=$ID"
    assert_match "cache-control: max-age=2000"
    assert_match "x-header: x-value"

    assert_command curl -vv "http://localhost:$PORT/.ignored-by-defualt.txt?canisterId=$ID"
    assert_match "404 Not Found"
    assert_command curl -vv "http://localhost:$PORT/.well-known/.hidden/ignored.txt?canisterId=$ID"
    assert_match "404 Not Found"
    assert_command curl -vv "http://localhost:$PORT/.well-known/.another-hidden/ignored.txt?canisterId=$ID"
    assert_match "404 Not Found"

}
@test "asset configuration via .ic-assets.json5 - overwriting default headers" {
    install_asset assetscanister

    dfx_start

    touch src/e2e_project_frontend/assets/thing.json

    echo '[
      {
        "match": "thing.json",
        "cache": { "max_age": 2000 },
        "headers": {
          "Content-Encoding": "my-encoding",
          "Content-Type": "x-type",
          "Cache-Control": "custom",
          "etag": "my-etag"
        }
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5

    dfx deploy

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl --head "http://localhost:$PORT/thing.json?canisterId=$ID"
    assert_match "cache-control: custom"
    assert_match "content-encoding: my-encoding"
    assert_match "content-type: x-type"
    assert_not_match "etag: my-etag"
    assert_match "etag: \"[a-z0-9]{64}\""
}

@test "aliasing rules: <filename> to <filename>.html or <filename>/index.html" {
    echo "test alias file" >'src/e2e_project_frontend/assets/test_alias_file.html'
    mkdir 'src/e2e_project_frontend/assets/index_test'
    echo "test index file" >'src/e2e_project_frontend/assets/index_test/index.html'

    dfx_start
    dfx deploy

    # decode as expected
    assert_command dfx canister  call --query e2e_project_frontend http_request '(record{url="/test_alias_file.html";headers=vec{};method="GET";body=vec{}})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/test_alias_file";headers=vec{};method="GET";body=vec{}})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request '(record{url="/index_test";headers=vec{};method="GET";body=vec{}})'
    assert_match "test index file"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file.html";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/index_test";content_encoding="identity";index=0})'
    assert_match "test index file"

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)

    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file.html?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command curl --fail -vv http://localhost:"$PORT"/index_test?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test index file"
    assert_command curl --fail -vv http://localhost:"$PORT"/index_test/index?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test index file"

    # redirect survives upgrade
    assert_command dfx deploy --upgrade-unchanged
    assert_match "is already installed"

    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file.html?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command curl --fail -vv http://localhost:"$PORT"/index_test?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test index file"

    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file.html";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/index_test";content_encoding="identity";index=0})'
    assert_match "test index file"

    # disabling redirect works
    echo "DISABLING NOW"
    echo '[
      {
        "match": "test_alias_file.html",
        "enable_aliasing": false
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5
    # '--mode reinstall --yes' can be removed once SDK-817 is implemented
    dfx deploy e2e_project_frontend --mode reinstall --yes
    
    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file.html?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command_fail curl --fail -vv http://localhost:"$PORT"/test_alias_file?canisterId="$ID"
    assert_match "404 Not Found" "$stderr"
    assert_command curl --fail -vv http://localhost:"$PORT"/index_test?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test index file"

    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file.html";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command_fail dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file";content_encoding="identity";index=0})'
    assert_match "key not found"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/index_test";content_encoding="identity";index=0})'
    assert_match "test index file"

    # disabled redirect survives canister upgrade
    echo "UPGRADE"
    assert_command dfx deploy --upgrade-unchanged
    
    assert_command curl --fail -vv http://localhost:"$PORT"/test_alias_file.html?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "200 OK" "$stderr"
    assert_match "test alias file"
    assert_command_fail curl --fail -vv http://localhost:"$PORT"/test_alias_file?canisterId="$ID"
    assert_match "404 Not Found" "$stderr"
    assert_command curl --fail -vv http://localhost:"$PORT"/index_test?canisterId="$ID"
    assert_match "200 OK" "$stderr"
    assert_match "test index file"

    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file.html";content_encoding="identity";index=0})'
    assert_match "test alias file"
    assert_command_fail dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/test_alias_file";content_encoding="identity";index=0})'
    assert_match "key not found"
    assert_command dfx canister call --query e2e_project_frontend http_request_streaming_callback '(record{key="/index_test";content_encoding="identity";index=0})'
    assert_match "test index file"

}

@test "asset configuration via .ic-assets.json5 - detect unused config" {
    install_asset assetscanister

    dfx_start

    mkdir src/e2e_project_frontend/assets/somedir
    touch src/e2e_project_frontend/assets/somedir/upload-me.txt
    echo '[
      {
        "match": "nevermatchme",
        "cache": { "max_age": 2000 }
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5
    echo '[
      {
        "match": "upload-me.txt",
        "headers": { "key": "value" }
      },
      {
        "match": "nevermatchme",
        "headers": {},
        "ignore": false
      },
      {
        "match": "nevermatchmetoo",
        "headers": null,
        "ignore": false
      },
      {
        "match": "non-matcher",
        "headers": {"x-header": "x-value"},
        "ignore": false
      },
      {
        "match": "/thanks-for-not-stripping-forward-slash",
        "headers": {"x-header": "x-value"},
        "ignore": false
      }
    ]' > src/e2e_project_frontend/assets/somedir/.ic-assets.json5

    assert_command dfx deploy
    assert_match 'WARN: 1 unmatched configuration in .*/src/e2e_project_frontend/assets/.ic-assets.json config file:'
    assert_contains 'WARN: {
  "match": "nevermatchme",
  "cache": {
    "max_age": 2000
  },
  "allow_raw_access": false
}'
    assert_match 'WARN: 4 unmatched configurations in .*/src/e2e_project_frontend/assets/somedir/.ic-assets.json config file:'
    assert_contains 'WARN: {
  "match": "nevermatchme",
  "headers": {},
  "ignore": false,
  "allow_raw_access": false
}
WARN: {
  "match": "nevermatchmetoo",
  "headers": {},
  "ignore": false,
  "allow_raw_access": false
}
WARN: {
  "match": "non-matcher",
  "headers": {
    "x-header": "x-value"
  },
  "ignore": false,
  "allow_raw_access": false
}'
    # splitting this up into two checks, because the order is different on macos vs ubuntu
    assert_contains 'WARN: {
  "match": "/thanks-for-not-stripping-forward-slash",
  "headers": {
    "x-header": "x-value"
  },
  "ignore": false,
  "allow_raw_access": false
}'
}

@test "asset configuration via .ic-assets.json5 - get and set asset properties" {
    install_asset assetscanister

    dfx_start

    mkdir src/e2e_project_frontend/assets/somedir
    touch src/e2e_project_frontend/assets/somedir/upload-me.txt
    echo '[
      {
        "match": "**/*",
        "cache": { "max_age": 2000 },
        "headers": { "x-key": "x-value" }
      }
    ]' > src/e2e_project_frontend/assets/.ic-assets.json5

    dfx deploy

    # read properties
    assert_command dfx canister call e2e_project_frontend get_asset_properties '("/somedir/upload-me.txt")'
    assert_contains '(
  record {
    headers = opt vec { record { "x-key"; "x-value" } };
    allow_raw_access = opt false;
    max_age = opt (2_000 : nat64);
  },
)'

    # access required to update
    assert_command_fail dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; max_age=opt(opt(5:nat64))  })' --identity anonymous
    assert_match "Caller does not have Commit permission"
    dfx identity new other --storage-mode plaintext
    assert_command_fail dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; max_age=opt(opt(5:nat64))  })' --identity other
    assert_match "Caller does not have Commit permission"

    # set max_age property and read it back
    assert_command dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; max_age=opt(opt(5:nat64))  })'
    assert_contains '()'
    assert_command dfx canister call e2e_project_frontend get_asset_properties '("/somedir/upload-me.txt")'
    assert_contains '(
  record {
    headers = opt vec { record { "x-key"; "x-value" } };
    allow_raw_access = opt false;
    max_age = opt (5 : nat64);
  },
)'

    # set headers property and read it back
    assert_command dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; headers=opt(opt(vec{record {"new-key"; "new-value"}}))})'
    assert_contains '()'
    assert_command dfx canister call e2e_project_frontend get_asset_properties '("/somedir/upload-me.txt")'
    assert_contains '(
  record {
    headers = opt vec { record { "new-key"; "new-value" } };
    allow_raw_access = opt false;
    max_age = opt (5 : nat64);
  },
)'

    # set allow_raw_access property and read it back
    assert_command dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; allow_raw_access=opt(opt(true))})'
    assert_contains '()'
    assert_command dfx canister call e2e_project_frontend get_asset_properties '("/somedir/upload-me.txt")'
    assert_contains '(
  record {
    headers = opt vec { record { "new-key"; "new-value" } };
    allow_raw_access = opt true;
    max_age = opt (5 : nat64);
  },
)'

    # set headers and max_age property to None and read it back
    assert_command dfx canister call e2e_project_frontend set_asset_properties '( record { key="/somedir/upload-me.txt"; headers=opt(null); max_age=opt(null); allow_raw_access=opt(null)})'
    assert_contains '()'
    assert_command dfx canister call e2e_project_frontend get_asset_properties '("/somedir/upload-me.txt")'
    assert_contains '(record { headers = null; allow_raw_access = null; max_age = null })'
}

@test "asset configuration via .ic-assets.json5 - pretty printing when deploying" {
    install_asset assetscanister

    dfx_start

    mkdir src/e2e_project_frontend/assets/somedir
    echo "content" > src/e2e_project_frontend/assets/somedir/upload-me.txt
    echo '[
      {
        "match": "**/*",
        "cache": { "max_age": 2000 },
        "headers": {
          "x-header": "x-value"
        },
        "enable_aliasing": true
      },
    ]' > src/e2e_project_frontend/assets/somedir/.ic-assets.json5

    assert_command dfx deploy
    assert_match '/somedir/upload-me.txt 1/1 \(8 bytes\) sha [0-9a-z]* \(with cache and 1 header\)'
}
