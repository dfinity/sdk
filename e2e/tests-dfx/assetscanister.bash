#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a different temporary directory for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export TEMPORARY_HOME="$x"
    export HOME="$TEMPORARY_HOME"
    cd "$TEMPORARY_HOME" || exit

    dfx_new
}

teardown() {
    dfx_stop
    rm -rf "$TEMPORARY_HOME"
}

@test "http_request percent-decodes urls" {
    install_asset assetscanister

    dfx_start

    echo "contents of file with space in filename" >'src/e2e_project_assets/assets/filename with space.txt'
    echo "contents of file with plus in filename" >'src/e2e_project_assets/assets/has+plus.txt'
    echo "contents of file with percent in filename" >'src/e2e_project_assets/assets/has%percent.txt'
    echo "filename is an ae symbol" >'src/e2e_project_assets/assets/æ'
    echo "filename is percent symbol" >'src/e2e_project_assets/assets/%'
    echo "filename contains question mark" >'src/e2e_project_assets/assets/filename?withqmark.txt'
    dd if=/dev/urandom of='src/e2e_project_assets/assets/large with spaces.bin' bs=2500000 count=1


    dfx deploy

    # decode as expected
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/filename%20with%20space.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with space in filename"
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/has%2bplus.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with plus in filename"
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/has%2Bplus.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with plus in filename"
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/has%%percent.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "contents of file with percent in filename"
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%e6";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename is an ae symbol" # candid looks like blob "filename is \c3\a6\0a"
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%%";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename is percent"
     # this test ensures url decoding happens after removing the query string
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/filename%3fwithqmark.txt";headers=vec{};method="GET";body=vec{}})'
    assert_match "filename contains question mark"

    # these error conditions can't be tested with curl, because something responds first with Bad Request.
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: % must be followed by '%' or two hex digits"
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%z";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: % must be followed by two hex digits, but only one was found"
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%zz";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: neither character after % is a hex digit"
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%e";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: % must be followed by two hex digits, but only one was found"
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%g6";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: first character after % is not a hex digit"
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request '(record{url="/%ch";headers=vec{};method="GET";body=vec{}})'
    assert_match "error decoding url: second character after % is not a hex digit"

    ID=$(dfx canister id e2e_project_assets)
    PORT=$(cat .dfx/webserver-port)

    assert_command curl --fail -vv http://localhost:"$PORT"/filename%20with%20space.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "contents of file with space in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/has%2bplus.txt?canisterId="$ID"
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "contents of file with plus in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/has%%percent.txt?canisterId="$ID"
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "contents of file with percent in filename"

    assert_command curl --fail -vv http://localhost:"$PORT"/%e6?canisterId="$ID"
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "filename is an ae symbol"

    assert_command curl --fail -vv http://localhost:"$PORT"/%%?canisterId="$ID"
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "filename is percent symbol"

    assert_command curl --fail -vv http://localhost:"$PORT"/filename%3fwithqmark.txt?canisterId="$ID"
    assert_match "HTTP/1.1 200 OK" "$stderr"
    assert_match "filename contains question mark"

    assert_command curl --fail -vv --output lws-curl-output.bin "http://localhost:$PORT/large%20with%20spaces.bin?canisterId=$ID"
    diff 'src/e2e_project_assets/assets/large with spaces.bin' lws-curl-output.bin

    assert_command_fail curl --fail -vv http://localhost:"$PORT"/'filename with space'.txt?canisterId="$ID"
    assert_match "400 Bad Request" "$stderr"
}

@test "generates gzipped content encoding for .js files" {
    install_asset assetscanister
    for i in $(seq 1 400); do
      echo "some easily duplicate text $i" >>src/e2e_project_assets/assets/notreally.js
    done

    dfx_start
    assert_command dfx deploy
    dfx canister call --query e2e_project_assets list '(record{})'

    ID=$(dfx canister id e2e_project_assets)
    PORT=$(cat .dfx/webserver-port)

    assert_command curl -v --output not-compressed http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_not_match "content-encoding:"
    diff not-compressed src/e2e_project_assets/assets/notreally.js

    assert_command curl -v --output encoded-compressed-1.gz -H "Accept-Encoding: gzip" http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_match "content-encoding: gzip"
    gunzip encoded-compressed-1.gz
    diff encoded-compressed-1 src/e2e_project_assets/assets/notreally.js

    # should split up accept-encoding lines with more than one encoding
    assert_command curl -v --output encoded-compressed-2.gz -H "Accept-Encoding: gzip, deflate, br" http://localhost:"$PORT"/notreally.js?canisterId="$ID"
    assert_match "content-encoding: gzip"
    gunzip encoded-compressed-2.gz
    diff encoded-compressed-2 src/e2e_project_assets/assets/notreally.js
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

    assert_command dfx deploy
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

    dfx deploy

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"identity"}})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/sample-asset.txt";accept_encodings=vec{"arbitrary"}})'
}

@test "verifies sha256, if specified" {
    install_asset assetscanister

    dfx_start
    dfx deploy

    assert_command dfx canister --no-wallet call --query e2e_project_assets get '(record{key="/text-with-newlines.txt";accept_encodings=vec{"identity"}})'

    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets get_chunk '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 88; 87; 86; }})'
    assert_match 'sha256 mismatch'

    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 243; 191; 114; 177; 83; 18; 144; 121; 131; 38; 109; 183; 89; 244; 120; 136; 53; 187; 14; 74; 8; 112; 86; 100; 115; 8; 179; 155; 69; 78; 95; 160; }})'
    assert_command dfx canister --no-wallet call --query e2e_project_assets http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0})'
    assert_command_fail dfx canister --no-wallet call --query e2e_project_assets http_request_streaming_callback '(record{key="/text-with-newlines.txt";content_encoding="identity";index=0;sha256=vec { 88; 87; 86; }})'
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
    dfx canister --no-wallet install e2e_project_assets

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
    touch src/e2e_project_assets/assets/index.js.LICENSE

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
    assert_command dfx canister call --query e2e_project_assets get '(record{key="/index.js.LICENSE";accept_encodings=vec{"identity"}})'
    assert_match 'content_type = "application/octet-stream"'
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
