#!/usr/bin/env bats

load ../utils/_

setup() {
    # when running e2e tests not in GitHub CI (so e.g. locally), build icx-proxy and set environment variable
    if [ -z "$ICX_ASSET" ]; then
        cargo build -p icx-asset
        ICX_ASSET="$(pwd)/target/debug/icx-asset"
    fi

    standard_setup

    dfx_new_frontend
    dfx_start

    assert_command dfx deploy
}

teardown() {
    dfx_stop

    standard_teardown
}

icx_asset_sync() {
  IDENTITY="$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem
  REPLICA_ADDRESS="http://localhost:$(get_webserver_port)"
  CANISTER_ID=$(dfx canister id e2e_project_frontend)
  if [ -z "$1" ]; then
      assert_command "$ICX_ASSET" --pem "$IDENTITY" --replica "$REPLICA_ADDRESS" sync "$CANISTER_ID" src/e2e_project_frontend/assets
  else
      # shellcheck disable=SC2086
      assert_command "$ICX_ASSET" --pem "$IDENTITY" --replica "$REPLICA_ADDRESS" sync "$CANISTER_ID" $1 $2
  fi
}

icx_asset_list() {
  IDENTITY="$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem
  REPLICA_ADDRESS="http://localhost:$(get_webserver_port)"
  CANISTER_ID=$(dfx canister id e2e_project_frontend)
  assert_command "$ICX_ASSET" --pem "$IDENTITY" --replica "$REPLICA_ADDRESS" ls "$CANISTER_ID"
}

icx_asset_upload() {
  IDENTITY="$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem
  REPLICA_ADDRESS="http://localhost:$(get_webserver_port)"
  CANISTER_ID=$(dfx canister id e2e_project_frontend)
  # shellcheck disable=SC2086
  assert_command "$ICX_ASSET" --pem "$IDENTITY" --replica "$REPLICA_ADDRESS" upload "$CANISTER_ID" $1 $2
}

@test "lists assets" {
    for i in $(seq 1 400); do
      echo "some easily duplicate text $i" >>src/e2e_project_frontend/assets/notreally.js
    done
    icx_asset_sync

    icx_asset_list

    assert_match "sample-asset.txt.*text/plain.*identity"
    assert_match "notreally.js.*application/javascript.*gzip"
    assert_match "notreally.js.*application/javascript.*identity"
}

@test "creates new files" {
  echo "new file content" >src/e2e_project_frontend/assets/new-asset.txt
  icx_asset_sync

  # shellcheck disable=SC2086
  assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/new-asset.txt";accept_encodings=vec{"identity"}})'
}

@test "updates existing files" {
    echo -n "an asset that will change" >src/e2e_project_frontend/assets/asset-to-change.txt
    assert_command dfx deploy

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/asset-to-change.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2154
    assert_match '"an asset that will change"' "$stdout"

    echo -n "an asset that has been changed" >src/e2e_project_frontend/assets/asset-to-change.txt

    icx_asset_sync

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/asset-to-change.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2154
    assert_match '"an asset that has been changed"' "$stdout"
  echo pass
}

@test "deletes removed files" {
    touch src/e2e_project_frontend/assets/will-delete-this.txt
    dfx deploy

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend list  '(record{})'
    assert_match '"/will-delete-this.txt"'

    rm src/e2e_project_frontend/assets/will-delete-this.txt

    icx_asset_sync

    assert_command_fail dfx canister call --query e2e_project_frontend get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend list  '(record{})'
    assert_not_match '"/will-delete-this.txt"'
}

@test "unsets asset encodings that are removed from project" {

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_frontend store '(record{key="/sample-asset.txt"; content_type="text/plain"; content_encoding="arbitrary"; content=blob "content encoded in another way!"})'

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'

    icx_asset_sync

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2086
    assert_command_fail dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'
}

@test "synchronizes multiple directories" {
    mkdir -p multiple/a
    mkdir -p multiple/b
    echo "x_contents" >multiple/a/x
    echo "y_contents" >multiple/b/y

    icx_asset_sync multiple/a multiple/b
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/x";accept_encodings=vec{"identity"}})'
    assert_match "x_contents"
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_frontend get '(record{key="/y";accept_encodings=vec{"identity"}})'
    assert_match "y_contents"
}

@test "reports errors about assets with the same key from multiple sources" {
    mkdir -p multiple/a
    mkdir -p multiple/b
    echo "a_duplicate_contents" >multiple/a/duplicate
    echo "b_duplicate_contents" >multiple/b/duplicate

    assert_command_fail icx_asset_sync multiple/a multiple/b
    assert_match "Asset with key '/duplicate' defined at .*/e2e_project/multiple/b/duplicate and .*/e2e_project/multiple/a/duplicate"
}

@test "ignores filenames and directories starting with a dot" {
    touch src/e2e_project_frontend/assets/.not-seen
    touch src/e2e_project_frontend/assets/is-seen

    mkdir -p src/e2e_project_frontend/assets/.dir-skipped
    touch src/e2e_project_frontend/assets/.dir-skipped/also-ignored

    mkdir -p src/e2e_project_frontend/assets/dir-not-skipped
    touch src/e2e_project_frontend/assets/dir-not-skipped/not-ignored

    icx_asset_sync

    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/is-seen";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_frontend get '(record{key="/dir-not-skipped/not-ignored";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister call --query e2e_project_frontend get '(record{key="/.not-seen";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister call --query e2e_project_frontend get '(record{key="/.dir-skipped/also-ignored";accept_encodings=vec{"identity"}})'

    assert_command dfx canister call --query e2e_project_frontend list  '(record{})'

    assert_match 'is-seen'
    assert_match 'not-ignored'

    assert_not_match 'not-seen'
    assert_not_match 'also-ignored'
}

@test "does not delete files that are not being uploaded" {
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    icx_asset_upload /=some_dir

    icx_asset_list

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"

    echo "ccc" >c.txt
    icx_asset_upload c.txt

    icx_asset_list

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"
    assert_match " /c.txt.*text/plain.*identity"
}

@test "deletes asset if necessary in order to change content type" {
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_frontend store '(record{key="/sample-asset.txt"; content_type="application/pdf"; content_encoding="identity"; content=blob "whatever contents!"})'
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_frontend store '(record{key="/sample-asset.txt"; content_type="application/pdf"; content_encoding="arbitrary"; content=blob "other contents"})'

    icx_asset_list

    assert_match " /sample-asset.txt.*application/pdf.*identity"
    assert_match " /sample-asset.txt.*application/pdf.*arbitrary"

    echo "just some text" >sample-asset.txt

    # icx-asset upload should delete the asset (and upload its replacement) since the content type is different.
    icx_asset_upload sample-asset.txt

    icx_asset_list

    assert_match " /sample-asset.txt.*text/plain.*identity"
    assert_not_match " /sample-asset.txt.*application/pdf.*arbitrary"
}

@test "uploads multiple files" {
    echo "this is the file content" >uploaded.txt
    echo "this is the file content ttt" >xyz.txt
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    icx_asset_upload some_dir/*.txt
    icx_asset_list

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"
}


@test "uploads multiple files from absolute path" {
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    icx_asset_upload "$(realpath some_dir/a.txt)" "$(realpath some_dir/b.txt)"
    icx_asset_list

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"
}

@test "uploads a file by name" {
    echo "this is the file content" >uploaded.txt

    icx_asset_upload uploaded.txt

    icx_asset_list

    assert_match " /uploaded.txt.*text/plain.*identity"
}

@test "can override asset name" {
    echo "this is the file content" >uploaded.txt

    icx_asset_upload /abcd.txt=uploaded.txt

    icx_asset_list

    assert_match " /abcd.txt.*text/plain.*identity"
}

@test "uploads a directory by name" {
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    icx_asset_upload some_dir

    icx_asset_list

    assert_match " /some_dir/a.txt.*text/plain.*identity"
    assert_match " /some_dir/b.txt.*text/plain.*identity"
}

@test "uploads a directory by name as root" {
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    icx_asset_upload /=some_dir

    icx_asset_list

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"
}
