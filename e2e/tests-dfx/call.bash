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

@test "call --output json" {
  install_asset method_signatures

  dfx_start
  dfx deploy

  assert_command dfx canister call hello_backend returns_string '("you")' --output json
  assert_eq '"Hello, you!"'

  assert_command dfx canister call hello_backend returns_opt_string '(null)' --output json
  assert_eq '[]'
  assert_command dfx canister call hello_backend returns_opt_string '(opt "world")' --output json
  assert_eq '[
  "Hello, world!"
]'


  # int is unbounded, so formatted as a string
  assert_command dfx canister call hello_backend returns_int '(67)' --output json
  assert_eq '"67"'
  assert_command dfx canister call hello_backend returns_int '(111222333444555666777888999 : int)' --output json
  assert_eq '"111_222_333_444_555_666_777_888_999"'

  assert_command dfx canister call hello_backend returns_int32 '(67)' --output json
  assert_eq '67'

  assert_command dfx canister call hello_backend returns_principal '(principal "fg7gi-vyaaa-aaaal-qadca-cai")' --output json
  assert_eq '"fg7gi-vyaaa-aaaal-qadca-cai"'

  # variant
  assert_command dfx canister call hello_backend returns_variant '(0)' --output json
  assert_eq '{
  "foo": null
}'
  assert_command dfx canister call hello_backend returns_variant '(1)' --output json
  assert_eq '{
  "bar": "a bar"
}'
  assert_command dfx canister call hello_backend returns_variant '(2)' --output json
  assert_eq '{
  "baz": {
    "a": 51
  }
}'

  assert_command dfx canister call hello_backend returns_strings '()' --output json
  assert_eq '[
  "Hello, world!",
  "Hello, Mars!"
]'

  assert_command dfx canister call hello_backend returns_object '()' --output json
  assert_eq '{
  "bar": "42",
  "foo": "baz"
}'

  assert_command dfx canister call hello_backend returns_blob '("abd")' --output json
  assert_eq '[
  97,
  98,
  100
]'

  assert_command dfx canister call hello_backend returns_tuple '()' --output json
  assert_eq '"the first element"
42
"the third element"'


  assert_command dfx canister call hello_backend returns_single_elem_tuple '()' --output json
  assert_eq '"the only element"'
}

@test "call --candid <path to candid file>" {
  install_asset call

  dfx_start
  dfx deploy
  assert_command dfx canister call hello_backend make_struct '("A", "B")'
  assert_eq '(record { c = "A"; d = "B" })'

  CANISTER_ID=$(dfx canister id hello_backend)
  rm .dfx/local/canister_ids.json

  # if no candid method known, then no field names
  assert_command dfx canister call "$CANISTER_ID" make_struct2 '("A", "B")'
  # shellcheck disable=SC2154
  assert_eq '(record { 99 = "A"; 100 = "B" })' "$stdout"

  # if passing the candid file, field names available
  assert_command dfx canister call --candid full.did "$CANISTER_ID" make_struct2 '("A", "B")'
  assert_eq '(record { c = "A"; d = "B" })'

  # given a canister id, fetch the did file from metadata
  assert_command dfx canister call "$CANISTER_ID" make_struct '("A", "B")'
  assert_eq '(record { c = "A"; d = "B" })'
}

@test "call without argument, using candid assistant" {
  install_asset echo
  dfx_start
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/candid_assist.exp"
}

@test "call subcommand accepts canister identifier as canister name" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  assert_command dfx canister call "$(dfx canister id hello_backend)" greet '("Names are difficult")'
  assert_match '("Hello, Names are difficult!")'
}

@test "call subcommand accepts raw argument" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  # The encoded raw argument was generated with `didc encode '("raw")'`
  assert_command dfx canister call hello_backend greet '4449444c00017103726177' --type raw
  assert_match '("Hello, raw!")'
}

@test "call subcommand accepts argument from a file" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  TMP_NAME_FILE="$(mktemp)"
  printf '("Names can be very long")' > "$TMP_NAME_FILE"
  assert_command dfx canister call --argument-file "$TMP_NAME_FILE" hello_backend greet
  assert_match '("Hello, Names can be very long!")'
  rm "$TMP_NAME_FILE"
}

@test "call subcommand accepts argument from stdin" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  TMP_NAME_FILE="$(mktemp)"
  printf '("stdin")' > "$TMP_NAME_FILE"
  assert_command dfx canister call --argument-file - hello_backend greet < "$TMP_NAME_FILE"
  assert_match '("Hello, stdin!")'
  rm "$TMP_NAME_FILE"
}

@test "call random value (pattern)" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  assert_command dfx canister call hello_backend greet --random '{ value = Some ["\"DFINITY\""] }'
  assert_match '("Hello, DFINITY!")'
}

@test "error on empty arguments when the method requires some" {
  install_asset greet
  dfx_start
  dfx deploy
  assert_command_fail dfx canister call hello_backend greet
}

@test "call random value (empty)" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  assert_command dfx canister call hello_backend greet --random ''
  assert_match '("Hello, .*!")'
}

@test "long call" {
  install_asset recurse
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  [[ "$USE_POCKETIC" ]] && dfx ledger fabricate-cycles --t 9999999 --canister hello_backend
  assert_command dfx canister call hello_backend recurse 100
}

@test "call with cycles" {
  dfx_start
  dfx deploy
  assert_command_fail dfx canister call hello_backend greet '' --with-cycles 100
  assert_command dfx canister call hello_backend greet '' --with-cycles 100 --wallet "$(dfx identity get-wallet)"
  dfx identity whoami
}

@test "call with cycles with wallet by name or by principal" {
  dfx_start
  dfx deploy
  assert_command_fail dfx canister call hello_backend greet '' --with-cycles 100

  assert_command dfx canister call hello_backend greet '' --with-cycles 100 --wallet "$(dfx identity get-wallet)"
  assert_command dfx canister call hello_backend greet '' --with-cycles 100 --wallet default
}


@test "call by canister id outside of a project" {
  install_asset greet
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install hello_backend
  ID="$(dfx canister id hello_backend)"
  NETWORK="http://localhost:$(get_webserver_port)"
  (
    cd "$E2E_TEMP_DIR"
    mkdir "not-a-project-dir"
    cd "not-a-project-dir"
    assert_command dfx canister call "$ID" greet '("you")' --network "$NETWORK"
    assert_match '("Hello, you!")'
  )
}

@test "call a canister which is deployed then removed from dfx.json" {
  dfx_start
  dfx deploy
  CANISTER_ID=$(dfx canister id hello_backend)
  jq 'del(.canisters.hello_backend)' dfx.json | sponge dfx.json
  assert_command dfx canister call hello_backend greet '("you")'
  assert_match '("Hello, you!")'
  assert_command dfx canister call "$CANISTER_ID" greet '("you")'
  assert_match '("Hello, you!")'
}

@test "inter-canister calls" {
  dfx_new_rust inter
  install_asset inter
  dfx_start
  dfx deploy

  # calling motoko canister from rust canister
  assert_command dfx canister call inter_rs read
  assert_match '(0 : nat)'
  assert_command dfx canister call inter_rs inc
  assert_command dfx canister call inter_rs read
  assert_match '(1 : nat)'
  assert_command dfx canister call inter_rs write '(5)'
  assert_command dfx canister call inter_rs read
  assert_match '(5 : nat)'

  # calling rust canister from motoko canister
  assert_command dfx canister call inter_mo write '(0)'
  assert_command dfx canister call inter_mo read
  assert_match '(0 : nat)'
  assert_command dfx canister call inter_mo inc
  assert_command dfx canister call inter_mo read
  assert_match '(1 : nat)'
  assert_command dfx canister call inter_mo write '(6)'
  assert_command dfx canister call inter_mo read
  assert_match '(6 : nat)'

  # calling rust canister from rust canister, trough motoko canisters
  assert_command dfx canister call inter2_rs write '(0)'
  assert_command dfx canister call inter2_rs read
  assert_match '(0 : nat)'
  assert_command dfx canister call inter2_rs inc
  assert_command dfx canister call inter2_rs read
  assert_match '(1 : nat)'
  assert_command dfx canister call inter2_rs write '(7)'
  assert_command dfx canister call inter2_rs read
  assert_match '(7 : nat)'

  # calling motoko canister from motoko canister, trough rust canisters
  assert_command dfx canister call inter2_mo write '(0)'
  assert_command dfx canister call inter2_mo read
  assert_match '(0 : nat)'
  assert_command dfx canister call inter2_mo inc
  assert_command dfx canister call inter2_mo read
  assert_match '(1 : nat)'
  assert_command dfx canister call inter2_mo write '(8)'
  assert_command dfx canister call inter2_mo read
  assert_match '(8 : nat)'
}
