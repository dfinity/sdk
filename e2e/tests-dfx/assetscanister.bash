#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    dfx_new
}

teardown() {
    dfx_stop
}

@test "verifies sha256, if specified" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/text-with-newlines.txt";accept_encodings=vec{"identity"}})'
    #assert_eq "aaaa"

    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 88; 87; 86; }})'
    assert_match 'sha256 mismatch'

    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request_next '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request_next '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request_next '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 88; 87; 86; }})'
    assert_match 'sha256 mismatch'

}

@test "can store and retrieve assets by key" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets retrieve '("/binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01 \80\0aw12 \00xy\0aKL\0b\0ajk")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("/text-with-newlines.txt")' --output idl
    assert_eq '(blob "cherries\0ait'\''s cherry season\0aCHERRIES")'

    assert_command dfx canister call --update e2e_project_assets store '(record{key="AA"; content_type="text/plain"; content_encoding="identity"; content=blob "hello, world!"})'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("AA")' --output idl
    assert_eq '(blob "hello, world!")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("B")' --output idl
    assert_eq '(blob "XWV")'

    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("C")'

    HOME=. assert_command_fail dfx canister call --update e2e_project_assets store '(record{key="index.js"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 1; 2; 3; }})'
    assert_match "Only a custodian can call this method."
}

@test "asset canister supports http requests" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    ID=$(dfx canister id e2e_project_assets)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/text-with-newlines.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_eq "cherries
it's cherry season
CHERRIES" "$stdout"
}

@test 'can store arbitrarily large files' {
    [ "$USE_IC_REF" ] && skip "skip for ic-ref" # this takes too long for ic-ref's wasm interpreter

    install_asset assetscanister
    dd if=/dev/urandom of=src/e2e_project_assets/assets/large-asset.bin bs=1000000 count=6

    dfx_start
    dfx canister --no-wallet create --all
    dfx build
    dfx canister --no-wallet install --memory-allocation 15mb e2e_project_assets

    # retrieve() refuses to serve just part of an asset
    assert_command_fail dfx canister call --query e2e_project_assets retrieve '("/large-asset.bin")'
    assert_match 'Asset too large.'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/large-asset.bin";accept_encodings=vec{"identity"}})'
    assert_match 'total_length = 6_000_000'
    assert_match 'content_type = "application/octet-stream"'
    assert_match 'content_encoding = "identity"'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=2})'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=3})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/large-asset.bin";content_encoding="identity";index=4})'

    PORT=$(cat .dfx/webserver-port)
    CANISTER_ID=$(dfx canister id e2e_project_assets)
    curl -v --output curl-output.bin "http://localhost:$PORT/large-asset.bin?canisterId=$CANISTER_ID"
    diff src/e2e_project_assets/assets/large-asset.bin curl-output.bin
}

@test "list() return assets" {
    install_asset assetscanister

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets list '(record{})'
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

    touch src/e2e_project_assets/assets/index.html
    touch src/e2e_project_assets/assets/logo.png
    touch src/e2e_project_assets/assets/index.js
    touch src/e2e_project_assets/assets/main.css
    touch src/e2e_project_assets/assets/index.js.map
    touch src/e2e_project_assets/assets/index.js.LICENSE.txt

    dfx build
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets get '(record{key="/index.html";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/html"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/logo.png";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "image/png"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/index.js";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "application/javascript"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/main.css";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/css"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/index.js.map";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/index.js.LICENSE.txt";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "text/plain"'
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
    dfx deploy

    assert_command_fail dfx canister call --query e2e_project_assets get '(record{key="/will-delete-this.txt";accept_encodings=vec{"identity"}})'
    assert_command dfx canister call --query e2e_project_assets list  '(record{})'
    assert_not_match '"/will-delete-this.txt"'
}
