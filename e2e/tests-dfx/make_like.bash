#!/usr/bin/env bats

load ../utils/_
# load ../utils/cycles-ledger

setup() {
  standard_setup

  install_asset make_like
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "trying to break dependency compiling" {
    dfx_start
    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'
}

# @test "deploy --upgrade-unchanged upgrades even if the .wasm did not change" {
#   dfx_start
#   assert_command dfx deploy

#   assert_command dfx deploy
#   assert_match "Module hash.*is already installed"

#   assert_command dfx deploy --upgrade-unchanged
#   assert_not_match "Module hash.*is already installed"
# }

# @test "deploy without --no-wallet sets wallet and self as the controllers" {
#   dfx_start
#   WALLET=$(dfx identity get-wallet)
#   PRINCIPAL=$(dfx identity get-principal)
#   assert_command dfx deploy hello_backend
#   assert_command dfx canister info hello_backend
#   assert_match "Controllers: ($WALLET $PRINCIPAL|$PRINCIPAL $WALLET)"
# }

# @test "deploy --no-wallet sets only self as the controller" {
#   dfx_start
#   WALLET=$(dfx identity get-wallet)
#   PRINCIPAL=$(dfx identity get-principal)
#   assert_command dfx deploy hello_backend --no-wallet
#   assert_command dfx canister info hello_backend
#   assert_not_match "Controllers: ($WALLET $PRINCIPAL|$PRINCIPAL $WALLET)"
#   assert_match "Controllers: $PRINCIPAL"
# }

# @test "deploy from a subdirectory" {
#   dfx_new hello
#   dfx_start
#   install_asset greet

#   (
#     cd src
#     assert_command dfx deploy
#     assert_match "Installing code for"
#   )

#   assert_command dfx canister call hello_backend greet '("Banzai")'
#   assert_eq '("Hello, Banzai!")'

#   assert_command dfx deploy
#   assert_not_match "Installing code for"
#   assert_match "is already installed"
# }

# @test "deploying multiple canisters with arguments fails" {
#   assert_command_fail dfx deploy --argument hello
#   assert_contains "The init argument can only be set when deploying a single canister."
# }

# @test "deploy one canister with an argument" {
#   dfx_start
#   assert_command dfx deploy hello_backend --argument '()'
# }

# @test "deploy one canister specifying raw argument" {
#   dfx_start
#   assert_command dfx deploy hello_backend --argument '4449444c0000' --argument-type raw
# }

# @test "deploy with an argument in a file" {
#   dfx_start
#   TMPFILE="$(mktemp)"
#   echo '()' >"$TMPFILE"
#   assert_command dfx deploy hello_backend --argument-file "$TMPFILE"
# }

# @test "deploying a dependent doesn't require already-installed dependencies to take args" {
#   install_asset deploy_deps
#   dfx_start
#   assert_command dfx deploy dependency --argument '("dfx")'
#   touch dependency.mo
#   assert_command dfx deploy dependent
#   assert_command dfx canister call dependency greet
#   assert_match "Hello, dfx!"
# }

# @test "deploy succeeds if init_arg is defined in dfx.json" {
#   install_asset deploy_deps
#   dfx_start
#   jq '.canisters.dependency.init_arg="(\"dfx\")"' dfx.json | sponge dfx.json
#   assert_command dfx deploy dependency
#   assert_command dfx canister call dependency greet
#   assert_match "Hello, dfx!"

#   assert_command dfx deploy dependency --mode reinstall --yes --argument '("icp")'
#   assert_contains "Canister 'dependency' has init_arg in dfx.json: (\"dfx\"),"
#   assert_contains "which is different from the one specified in the command line: (\"icp\")."
#   assert_contains "The command line value will be used."
#   assert_command dfx canister call dependency greet
#   assert_match "Hello, icp!"
# }

# @test "reinstalling a single Motoko canister with imported dependency works" {
#   install_asset import_canister
#   dfx_start
#   assert_command dfx deploy
#   assert_command dfx deploy importer --mode reinstall --yes
# }

# @test "deploy succeeds with --specified-id" {
#   dfx_start
#   assert_command dfx deploy hello_backend --specified-id n5n4y-3aaaa-aaaaa-p777q-cai
#   assert_command dfx canister id hello_backend
#   assert_match n5n4y-3aaaa-aaaaa-p777q-cai
# }

# @test "deploy fails if --specified-id without canister_name" {
#   dfx_start
#   assert_command_fail dfx deploy --specified-id n5n4y-3aaaa-aaaaa-p777q-cai
#   assert_match \
# "error: the following required arguments were not provided:
#   <CANISTER_NAME>"
# }

# @test "deploy succeeds when specify canister ID in dfx.json" {
#   dfx_start
#   jq '.canisters.hello_backend.specified_id="n5n4y-3aaaa-aaaaa-p777q-cai"' dfx.json | sponge dfx.json
#   assert_command dfx deploy hello_backend
#   assert_command dfx canister id hello_backend
#   assert_match n5n4y-3aaaa-aaaaa-p777q-cai
# }

# @test "deploy succeeds when specify canister ID both in dfx.json and cli; warning if different; cli value takes effect" {
#   dfx_start
#   jq '.canisters.hello_backend.specified_id="n5n4y-3aaaa-aaaaa-p777q-cai"' dfx.json | sponge dfx.json
#   assert_command dfx deploy hello_backend --specified-id hhn2s-5l777-77777-7777q-cai
#   assert_contains "WARN: Canister 'hello_backend' has a specified ID in dfx.json: n5n4y-3aaaa-aaaaa-p777q-cai,"
#   assert_contains "which is different from the one specified in the command line: hhn2s-5l777-77777-7777q-cai."
#   assert_contains "The command line value will be used."

#   assert_command dfx canister id hello_backend
#   assert_match hhn2s-5l777-77777-7777q-cai
# }

# @test "deploy does not require wallet if all canisters are created" {
#   dfx_start
#   dfx canister create --all --no-wallet
#   assert_command dfx deploy
#   assert_not_contains "Creating a wallet canister"
#   assert_command dfx identity get-wallet
#   assert_contains "Creating a wallet canister"
# }

# @test "can deploy gzip wasm" {
#   jq '.canisters.hello_backend.gzip=true' dfx.json | sponge dfx.json
#   dfx_start
#   assert_command dfx deploy
#   BUILD_HASH="0x$(sha256sum .dfx/local/canisters/hello_backend/hello_backend.wasm.gz | cut -d " " -f 1)"
#   ONCHAIN_HASH="$(dfx canister info hello_backend | tail -n 1 | cut -d " " -f 3)"
#   assert_eq "$BUILD_HASH" "$ONCHAIN_HASH"
# }

# @test "prints the frontend url after deploy" {
#   dfx_new_frontend hello
#   dfx_start
#   assert_command dfx deploy
#   frontend_id=$(dfx canister id hello_frontend)
#   assert_match "http://127.0.0.1.+${frontend_id}"
#   assert_match "${frontend_id}.localhost"
# }

# @test "prints the frontend url if 'frontend' section is not present in dfx.json" {
#   dfx_new_frontend hello
#   jq 'del(.canisters.hello_frontend.frontend)' dfx.json | sponge dfx.json
#   dfx_start
#   assert_command dfx deploy
#   frontend_id=$(dfx canister id hello_frontend)
#   assert_match "http://127.0.0.1.+${frontend_id}"
#   assert_match "${frontend_id}.localhost"
# }

# @test "prints the frontend url if the frontend section has been removed after initial deployment" {
#   dfx_new_frontend hello
#   dfx_start
#   assert_command dfx deploy
#   frontend_id=$(dfx canister id hello_frontend)
#   assert_match "http://127.0.0.1.+${frontend_id}"
#   assert_match "${frontend_id}.localhost"
#   jq 'del(.canisters.hello_frontend.frontend)' dfx.json | sponge dfx.json
#   assert_command dfx deploy
#   assert_match "http://127.0.0.1.+${frontend_id}"
#   assert_match "${frontend_id}.localhost"
# }

# @test "subnet targetting" {
#   # fake cmc setup
#   cd ..
#   dfx_new fake_cmc
#   install_asset fake_cmc
#   install_cycles_ledger_canisters
#   dfx_start
#   assert_command dfx deploy fake-cmc --specified-id "rkp4c-7iaaa-aaaaa-aaaca-cai" # CMC canister id
#   cd ../hello

#   # use --subnet <principal>
#   SUBNET_ID="5kdm2-62fc6-fwnja-hutkz-ycsnm-4z33i-woh43-4cenu-ev7mi-gii6t-4ae" # a random, valid principal
#   assert_command dfx deploy hello_backend --subnet "$SUBNET_ID"
#   cd ../fake_cmc
#   assert_command dfx canister call fake-cmc last_create_canister_args
#   assert_contains "subnet = principal \"$SUBNET_ID\";"
  
#   # use --subnet-type
#   cd ../hello
#   assert_command dfx deploy hello_frontend --subnet-type custom_subnet_type
#   cd ../fake_cmc
#   assert_command dfx canister call fake-cmc last_create_canister_args
#   assert_contains 'subnet_type = opt "custom_subnet_type"'
# }
