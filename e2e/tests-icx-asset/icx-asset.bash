#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
    dfx new --no-frontend e2e_project
    cd e2e_project || exit 1
    dfx start --background
    dfx deploy
}

teardown() {
    echo teardown
    dfx stop
}

icx_asset_sync() {
  CANISTER_ID=$(dfx canister id e2e_project_assets)
  assert_command "$ICX_ASSET" --pem "$HOME"/.config/dfx/identity/default/identity.pem sync "$CANISTER_ID" "${@:-src/e2e_project_assets/assets}"
}

icx_asset_list() {
  CANISTER_ID=$(dfx canister id e2e_project_assets)
  assert_command "$ICX_ASSET" --pem "$HOME"/.config/dfx/identity/default/identity.pem ls "$CANISTER_ID"
}

@test "lists assets" {
    for i in $(seq 1 400); do
      echo "some easily duplicate text $i" >>src/e2e_project_assets/assets/notreally.js
    done
    icx_asset_sync

    icx_asset_list

    assert_match "sample-asset.txt.*text/plain.*identity"
    assert_match "notreally.js.*application/javascript.*gzip"
    assert_match "notreally.js.*application/javascript.*identity"
}

@test "creates new files" {
  echo "new file content" >src/e2e_project_assets/assets/new-asset.txt
  icx_asset_sync

  # shellcheck disable=SC2086
  assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/new-asset.txt";accept_encodings=vec{"identity"}})'
}

@test "updates existing files" {
    echo -n "an asset that will change" >src/e2e_project_assets/assets/asset-to-change.txt
    assert_command dfx deploy

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/asset-to-change.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2154
    assert_match '"an asset that will change"' "$stdout"

    echo -n "an asset that has been changed" >src/e2e_project_assets/assets/asset-to-change.txt

    icx_asset_sync

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/asset-to-change.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2154
    assert_match '"an asset that has been changed"' "$stdout"
  echo pass
}

@test "deletes removed files" {
    touch src/e2e_project_assets/assets/will-delete-this.txt
    dfx deploy

    assert_command dfx canister call --query e2e_project_assets get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_assets list  '(record{})'
    assert_match '"/will-delete-this.txt"'

    rm src/e2e_project_assets/assets/will-delete-this.txt

    icx_asset_sync

    assert_command_fail dfx canister call --query e2e_project_assets get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_assets list  '(record{})'
    assert_not_match '"/will-delete-this.txt"'
}

@test "unsets asset encodings that are removed from project" {

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --update e2e_project_assets store '(record{key="/sample-asset.txt"; content_type="text/plain"; content_encoding="arbitrary"; content=blob "content encoded in another way!"})'

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'

    icx_asset_sync

    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    # shellcheck disable=SC2086
    assert_command_fail dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'
}

@test "synchronizes multiple directories" {
    mkdir -p multiple/a
    mkdir -p multiple/b
    echo "x_contents" >multiple/a/x
    echo "y_contents" >multiple/b/y

    icx_asset_sync multiple/a multiple/b
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/x";accept_encodings=vec{"identity"}})'
    assert_match "x_contents"
    # shellcheck disable=SC2086
    assert_command dfx canister ${DFX_NO_WALLET:-} call --query e2e_project_assets get '(record{key="/y";accept_encodings=vec{"identity"}})'
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
    touch src/e2e_project_assets/assets/.not-seen
    touch src/e2e_project_assets/assets/is-seen

    mkdir -p src/e2e_project_assets/assets/.dir-skipped
    touch src/e2e_project_assets/assets/.dir-skipped/also-ignored

    mkdir -p src/e2e_project_assets/assets/dir-not-skipped
    touch src/e2e_project_assets/assets/dir-not-skipped/not-ignored

    icx_asset_sync

    assert_command dfx canister call --query e2e_project_assets get '(record{key="/is-seen";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/dir-not-skipped/not-ignored";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister call --query e2e_project_assets get '(record{key="/.not-seen";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister call --query e2e_project_assets get '(record{key="/.dir-skipped/also-ignored";accept_encodings=vec{"identity"}})'

    assert_command dfx canister call --query e2e_project_assets list  '(record{})'

    assert_match 'is-seen'
    assert_match 'not-ignored'

    assert_not_match 'not-seen'
    assert_not_match 'also-ignored'
}
