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

@test "leaves in place files that were already installed" {
    install_asset assetscanister
    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset1.bin bs=400000 count=1
    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset2.bin bs=400000 count=1

    dfx_start
    assert_command dfx deploy

    assert_match '/asset1.bin 1/1'
    assert_match '/asset2.bin 1/1'

    dd if=/dev/urandom of=src/e2e_project_assets/assets/asset2.bin bs=400000 count=1

    CANISTER_ID=$(dfx canister id e2e_project_assets)
    REPLICA_PORT=$(cat .dfx/webserver-port)
    assert_command "$ICX_ASSET" --replica http://localhost:"$REPLICA_PORT" --pem "$HOME/.config/dfx/identity/default/identity.pem" --fetch-root-key sync "$CANISTER_ID" src/e2e_project_assets/assets/

    assert_match '/asset1.bin.*is already installed'
    assert_match '/asset2.bin 1/1'
}

@test "unsets asset encodings that are removed from project" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister --no-wallet call --update e2e_project_assets store '(record{key="/sample-asset.txt"; content_type="text/plain"; content_encoding="arbitrary"; content=blob "content encoded in another way!"})'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'

    CANISTER_ID=$(dfx canister id e2e_project_assets)
    REPLICA_PORT=$(cat .dfx/webserver-port)
    assert_command "$ICX_ASSET" --replica http://localhost:"$REPLICA_PORT" --pem "$HOME/.config/dfx/identity/default/identity.pem" --fetch-root-key sync "$CANISTER_ID" src/e2e_project_assets/assets/

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

    CANISTER_ID=$(dfx canister id e2e_project_assets)
    REPLICA_PORT=$(cat .dfx/webserver-port)
    assert_command "$ICX_ASSET" --replica http://localhost:"$REPLICA_PORT" --pem "$HOME/.config/dfx/identity/default/identity.pem" --fetch-root-key sync "$CANISTER_ID" src/e2e_project_assets/assets/

    assert_command_fail dfx canister call --query e2e_project_assets get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_assets list  '(record{})'
    assert_not_match '"/will-delete-this.txt"'
}
