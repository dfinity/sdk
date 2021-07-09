#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a different temporary directory for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export TEMPORARY_HOME="$x"
    export HOME="$TEMPORARY_HOME"
    cd "$TEMPORARY_HOME" || exit

    x=$(dfx cache show)
    export ICX_ASSET="$x/icx-asset"

    dfx_new
}

teardown() {
    dfx_stop
    rm -rf "$TEMPORARY_HOME"
}

icx_asset_sync() {
    CANISTER_ID=$(dfx canister id e2e_project_assets)
    REPLICA_PORT=$(cat .dfx/webserver-port)
    assert_command "$ICX_ASSET" --replica http://localhost:"$REPLICA_PORT" --pem "$HOME/.config/dfx/identity/default/identity.pem" sync "$CANISTER_ID" src/e2e_project_assets/assets/
}

@test "leaves in place files that were already installed" {
    install_asset assetscanister
    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset1.bin bs=400000 count=1
    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset2.bin bs=400000 count=1

    dfx_start
    assert_command dfx deploy

    assert_match '/asset1.bin 1/1'
    assert_match '/asset2.bin 1/1'

    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset2.bin bs=400000 count=1

    icx_asset_sync

    assert_match '/asset1.bin.*is already installed'
    assert_match '/asset2.bin 1/1'
}

@test "creates new assets" {
    install_asset assetscanister

    dfx_start
    assert_command dfx deploy

    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/new_asset.txt";accept_encodings=vec{"identity"}})'

    echo -n "this is a new asset" >src/e2e_project_assets/assets/new_asset.txt

    icx_asset_sync

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/new_asset.txt";accept_encodings=vec{"identity"}})'

    assert_eq "('this is a new asset')"
}

@test "modifies changed assets" {
    install_asset assetscanister

    dfx_start
    assert_command dfx deploy

    echo -n "a changed asset" >src/e2e_project_assets/assets/text-with-newlines.txt

    icx_asset_sync

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/text-with-newlines.txt";accept_encodings=vec{"identity"}})'

    # shellcheck disable=SC2154
    assert_eq "a changed asset" "$stdout"
}

@test "unsets asset encodings that are removed from project" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister --no-wallet call --update e2e_project_assets store '(record{key="/sample-asset.txt"; content_type="text/plain"; content_encoding="arbitrary"; content=blob "content encoded in another way!"})'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'

    icx_asset_sync

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'
}

@test "deletes assets that are removed from project" {
    install_asset assetscanister

    dfx_start

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
