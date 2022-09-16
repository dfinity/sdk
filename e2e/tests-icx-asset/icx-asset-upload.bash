#!/usr/bin/env bats

load ../utils/_

setup() {
      # We want to work from a different temporary directory for every test.
    x=$(mktemp -d -t icx-asset-e2e-XXXXXXXX)
    export DFX_CONFIG_ROOT="$x"
    cd "$DFX_CONFIG_ROOT" || exit

    dfx new --no-frontend e2e_project
    cd e2e_project || exit 1
    dfx start --background
    dfx deploy
}

teardown() {
    dfx stop
    rm -rf "$DFX_CONFIG_ROOT"
}

icx_asset_sync() {
  CANISTER_ID=$(dfx canister id e2e_project_assets)
  assert_command "$ICX_ASSET" --pem "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem sync "$CANISTER_ID" src/e2e_project_assets/assets
}

icx_asset_list() {
  CANISTER_ID=$(dfx canister id e2e_project_assets)
  assert_command "$ICX_ASSET" --pem "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem ls "$CANISTER_ID"
}

icx_asset_upload() {
  # for some reason, if you pass more than 1 parameter, and replace "$1" with "$@",
  # this function doesn't call icx-asset at all.
  CANISTER_ID=$(dfx canister id e2e_project_assets)
  assert_command "$ICX_ASSET" --pem "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem upload "$CANISTER_ID" "$1"
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
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_assets store '(record{key="/sample-asset.txt"; content_type="application/pdf"; content_encoding="identity"; content=blob "whatever contents!"})'
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_assets store '(record{key="/sample-asset.txt"; content_type="application/pdf"; content_encoding="arbitrary"; content=blob "other contents"})'

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

    CANISTER_ID=$(dfx canister id e2e_project_assets)
    assert_command "$ICX_ASSET" --pem "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem upload "$CANISTER_ID" some_dir/*.txt

    icx_asset_list

    # expect: (is this surprising?)
    #   /a.txt
    #   /b.txt

    assert_match " /a.txt.*text/plain.*identity"
    assert_match " /b.txt.*text/plain.*identity"
}


@test "uploads multiple files from absolute path" {
    mkdir some_dir
    echo "some stuff" >some_dir/a.txt
    echo "more things" >some_dir/b.txt

    CANISTER_ID=$(dfx canister id e2e_project_assets)
    assert_command "$ICX_ASSET" --pem "$DFX_CONFIG_ROOT"/.config/dfx/identity/default/identity.pem upload \
       "$CANISTER_ID" \
       "$(realpath some_dir/a.txt)" "$(realpath some_dir/b.txt)"

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

    # expect:
    #   /some_dir/a.txt
    #   /some_dir/b.txt

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
