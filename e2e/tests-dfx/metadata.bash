#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "custom canister metadata rules" {
  install_asset metadata/custom
  install_asset wasm/identity

  dfx_start
  dfx deploy

  echo "leaves existing metadata in a custom canister with no metadata settings"
  dfx canister metadata --identity anonymous custom_with_default_metadata candid:service >metadata.txt
  diff main.did metadata.txt

  echo "adds candid:service public metadata from candid field if a metadata entry exists"
  dfx canister metadata --identity anonymous custom_with_standard_candid_service_metadata candid:service >metadata.txt
  diff custom_with_standard_candid_service_metadata.did metadata.txt

  echo "adds candid:service metadata from candid field with private visibility per metadata entry"
  assert_command_fail dfx canister metadata --identity anonymous custom_with_private_candid_service_metadata candid:service >metadata.txt
  dfx canister metadata custom_with_private_candid_service_metadata candid:service >metadata.txt
  diff custom_with_private_candid_service_metadata.did metadata.txt
}

@test "rust canister metadata rules" {
  rustup default stable
  rustup target add wasm32-unknown-unknown

  dfx_new_rust

  dfx_start
  dfx deploy

  echo "adds public candid:service metadata to a default rust canister"
  dfx canister metadata --identity anonymous e2e_project_backend candid:service >metadata.txt
  diff src/e2e_project_backend/e2e_project_backend.did metadata.txt

  echo "adds private candid:service metadata if so configured"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="candid:service"|.canisters.e2e_project_backend.metadata[0].visibility="private"' dfx.json | sponge dfx.json
  dfx deploy
  assert_command_fail dfx canister metadata --identity anonymous e2e_project_backend candid:service
  dfx canister metadata e2e_project_backend candid:service >metadata.txt
  diff src/e2e_project_backend/e2e_project_backend.did metadata.txt
}

@test "motoko canister metadata rules" {
  dfx_new
  dfx_start
  install_asset metadata/motoko
  dfx canister create --all

  echo "permits specification of a replacement candid definition, if it is a valid subtype"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  assert_command dfx build
  find . -name '*.did'
  jq '.canisters.e2e_project_backend.metadata[0].name="candid:service"|.canisters.e2e_project_backend.metadata[0].path="valid_subtype.did"' dfx.json | sponge dfx.json
  dfx build

  echo "reports an error if a specified candid:service metadata is not a valid subtype for the canister"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="candid:service"|.canisters.e2e_project_backend.metadata[0].path="not_subtype_rename.did"' dfx.json | sponge dfx.json
  assert_command_fail dfx build
  assert_match "Method new_method is only in the expected type"

  echo "reports an error if a specified candid:service metadata is not a valid subtype for the canister"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="candid:service"|.canisters.e2e_project_backend.metadata[0].path="not_subtype_numbertype.did"' dfx.json | sponge dfx.json
  assert_command_fail dfx build
  assert_match "int is not a subtype of nat"


  echo "adds private candid:service metadata if so configured"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="candid:service"|.canisters.e2e_project_backend.metadata[0].visibility="private"' dfx.json | sponge dfx.json
  dfx deploy
  assert_command_fail dfx canister metadata --identity anonymous e2e_project_backend candid:service
  dfx canister metadata e2e_project_backend candid:service >metadata.txt
  diff .dfx/local/canisters/e2e_project_backend/e2e_project_backend.did metadata.txt


  echo "adds public candid:service metadata to a default motoko canister"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  dfx deploy
  dfx canister metadata --identity anonymous e2e_project_backend candid:service >metadata.txt
  diff .dfx/local/canisters/e2e_project_backend/e2e_project_backend.did metadata.txt
}

@test "adds arbitrary metadata to a motoko canister" {
  dfx_new
  dfx_start
  install_asset metadata/motoko
  dfx canister create --all

  echo "adds public arbitrary metadata to a default motoko canister"
  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="arbitrary"|.canisters.e2e_project_backend.metadata[0].path="arbitrary-metadata.txt"' dfx.json | sponge dfx.json
  echo "can be anything" >arbitrary-metadata.txt
  dfx deploy
  dfx canister metadata --identity anonymous e2e_project_backend arbitrary >from-canister.txt
  diff arbitrary-metadata.txt from-canister.txt

  # with private visibility
  jq '.canisters.e2e_project_backend.metadata[0].visibility="private"' dfx.json | sponge dfx.json
  dfx deploy
  assert_command_fail dfx canister metadata --identity anonymous e2e_project_backend arbitrary
  dfx canister metadata e2e_project_backend arbitrary >from-canister.txt
  diff arbitrary-metadata.txt from-canister.txt
}

@test "uses the first metadata definition for name and network" {
  dfx_new
  dfx_start
  install_asset metadata/motoko
  dfx canister create --all

  jq 'del(.canisters.e2e_project_backend.metadata)' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[0].name="multiple"|.canisters.e2e_project_backend.metadata[0].path="empty-networks-matches-nothing.txt"|.canisters.e2e_project_backend.metadata[0].networks=[]' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[1].name="multiple"|.canisters.e2e_project_backend.metadata[1].path="different-network-no-match.txt"|.canisters.e2e_project_backend.metadata[1].networks=["ic"]' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[2].name="multiple"|.canisters.e2e_project_backend.metadata[2].path="first-match-chosen.txt"' dfx.json | sponge dfx.json
  jq '.canisters.e2e_project_backend.metadata[3].name="multiple"|.canisters.e2e_project_backend.metadata[3].path="earlier-match-ignored.txt"' dfx.json | sponge dfx.json
  echo "dfx will install this file" >first-match-chosen.txt
  dfx deploy
  dfx canister metadata --identity anonymous e2e_project_backend multiple >from-canister.txt
  diff first-match-chosen.txt from-canister.txt
}

@test "can add metadata to a compressed canister" {
  dfx_start
  install_asset gzip
  install_asset wasm/identity
  jq '.canisters.gzipped.metadata[0].name="arbitrary"|.canisters.gzipped.metadata[0].content="arbitrary content"' dfx.json | sponge dfx.json

  assert_command dfx deploy
  assert_command dfx canister metadata gzipped arbitrary
  assert_eq "$output" "arbitrary content"
}

@test "existence of build steps do not control custom canister metadata" {
  install_asset prebuilt_custom_canister
  install_asset wasm/identity

  dfx_start
  dfx deploy

  # this canister has a build step, which doesn't matter: dfx leaves the candid metadata
  dfx canister metadata custom_with_build_step candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister doesn't have a build step, so dfx leaves the candid metadata as-is
  dfx canister metadata prebuilt_custom_no_build candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister has a build step, but it is an empty string, so dfx leaves the candid:service metadata as-is
  dfx canister metadata prebuilt_custom_blank_build candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister has a build step, but it is an empty array, so dfx leaves the candid:service metadata as-is
  dfx canister metadata prebuilt_custom_empty_build candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister has a local import in did file, the metadata should flatten the definitions
  assert_command dfx canister metadata prebuilt_local_import candid:service
  assert_eq "service : { getCanisterId : () -> (principal) query }"
}

@test "can read canister metadata from replica" {
  dfx_new hello
  dfx_start

  assert_command dfx deploy

  dfx canister metadata hello_backend candid:service >metadata.txt
  assert_command diff .dfx/local/canisters/hello_backend/hello_backend.did ./metadata.txt
}

@test "asset canister provides candid:service metadata" {
  dfx_new_assets hello
  dfx_start

  assert_command dfx deploy
  REPO_ROOT=${BATS_TEST_DIRNAME}/../../

  dfx canister metadata hello_frontend candid:service >candid_service_metadata.txt
  assert_command diff "$REPO_ROOT/src/distributed/assetstorage.did" ./candid_service_metadata.txt
}

# shellcheck disable=SC2154
@test "can generate tech_stack field of the standardized dfx metadata" {
  dfx_new
  install_asset metadata/tech_stack

  dfx_start

  # a doesn't define the tech_stack object, the dfx metadata is not added
  echo -ne '\0asm\x01\0\0\0' > a.wasm
  assert_command dfx deploy a
  assert_command_fail dfx canister metadata a dfx

  # b defines one cdk item
  assert_command dfx deploy b
  assert_command dfx canister metadata b dfx
  echo "$stdout" > b.json
  assert_command jq -r '.tech_stack.cdk | keys[]' b.json
  assert_eq "ic-cdk"

  # c defines language->rust version
  assert_command dfx deploy c
  assert_command dfx canister metadata c dfx
  echo "$stdout" > c.json
  assert_command jq -r '.tech_stack.language.rust.version' c.json
  assert_eq "1.75.0"

  # d defines language->rust version with value_command
  assert_command dfx deploy d
  assert_command dfx canister metadata d dfx
  echo "$stdout" > d.json
  assert_command jq -r '.tech_stack.language.rust.version' d.json
  assert_eq "1.75.0"

  # e defines multiple lib items
  assert_command dfx deploy e
  assert_command dfx canister metadata e dfx
  echo "$stdout" > e.json
  assert_command jq -r '.tech_stack.lib | keys[]' e.json
  assert_eq "ic-cdk-timers
ic-stable-structures"

  # f defines all 5 categories
  assert_command dfx deploy f
  assert_command dfx canister metadata f dfx
  echo "$stdout" > f.json
  assert_command jq -r '.tech_stack.cdk | keys[]' f.json
  assert_eq "ic-cdk"
  assert_command jq -r '.tech_stack.language | keys[]' f.json
  assert_eq "rust"
  assert_command jq -r '.tech_stack.lib | keys[]' f.json
  assert_eq "ic-cdk-timers"
  assert_command jq -r '.tech_stack.tool | keys[]' f.json
  assert_eq "dfx"
  assert_command jq -r '.tech_stack.other | keys[]' f.json
  assert_eq "bitcoin"

  # g defines a value_command that is a local file without "./" prefix and the file name contains whitespace
  assert_command dfx deploy g
  assert_command dfx canister metadata g dfx
  echo "$stdout" > g.json
  assert_command jq -r '.tech_stack.language.rust.version' g.json
  assert_eq "1.75.0"

  # h defines a value_command that is a local command(prefix "./") contains whitespace
  assert_command dfx deploy h
  assert_command dfx canister metadata h dfx
  echo "$stdout" > h.json
  assert_command jq -r '.tech_stack.language.rust.version' h.json
  assert_eq "1.75.0"

  # i defines a value_command that fails
  assert_command_fail dfx deploy i
  assert_contains "Failed to run the value_command: language->rust->version."

  # j defines a value_command that returns a non-valid string
  echo -e "\xc3\x28" > invalid_utf8.txt
  assert_command_fail dfx deploy j
  assert_contains "The value_command didn't return a valid UTF-8 string: language->rust->version."

  # TODO: remove this when we have motoko extension
  # k is a motoko canister which doesn't define tech_stack
  # tech_stack->language->motoko is added by default
  assert_command dfx deploy k
  assert_command dfx canister metadata k dfx
  echo "$stdout" > k.json
  assert_command jq -r '.tech_stack.language | keys[]' k.json
  assert_eq "motoko"
}

# TODO: remove this when we have rust extension
# shellcheck disable=SC2154
@test "rust canister set default tech_stack" {
  dfx_new_rust
  dfx_start
  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister metadata e2e_project_backend dfx
  echo "$stdout" > e2e_project_backend.json
  assert_command jq -r '.tech_stack.language.rust | keys[]' e2e_project_backend.json
  assert_eq "version"
  assert_command jq -r '.tech_stack.cdk."ic-cdk" | keys[]' e2e_project_backend.json
  assert_eq "version"
}
