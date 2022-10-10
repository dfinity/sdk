#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    stop_webserver
    dfx_stop

    standard_teardown
}

@test "can build a custom canister with wasm and/or candid from a url" {
    install_asset wasm/identity
    mkdir -p www/wasm
    mv main.wasm www/wasm/
    mv main.did www/wasm
    start_webserver --directory www
    dfx_start

    dfx_new

    jq '.canisters={}' dfx.json | sponge dfx.json

    jq '.canisters.e2e_project.candid="http://localhost:'"$E2E_WEB_SERVER_PORT"'/wasm/main.did"' dfx.json | sponge dfx.json
    jq '.canisters.e2e_project.wasm="http://localhost:'"$E2E_WEB_SERVER_PORT"'/wasm/main.wasm"' dfx.json | sponge dfx.json
    jq '.canisters.e2e_project.type="custom"' dfx.json | sponge dfx.json

    dfx deploy

    ID=$(dfx canister id e2e_project)
    assert_command dfx canister call e2e_project getCanisterId
    assert_match "$ID"
}

@test "build uses default build args" {
    install_asset default_args
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build --check
    assert_match "unknown option"
    assert_match "compacting-gcX"
}

@test "build uses canister build args" {
    install_asset canister_args
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build --check
    assert_match "unknown option"
    assert_match "compacting-gcY"
    assert_not_match "compacting-gcX"
}

@test "empty canister build args don't shadow default" {
    install_asset empty_canister_args
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build --check
    assert_match '"--error-detail" "5"'
    assert_match "unknown option"
    assert_match "compacting-gcX"
}

@test "build fails on invalid motoko" {
    install_asset invalid
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build supports relative imports" {
    install_asset import
    dfx_start
    dfx canister create --all
    assert_command dfx build
    dfx canister install --all
    assert_command dfx canister call e2e_project_backend greet World
    assert_match "10World"
}

@test "build succeeds on default project" {
    dfx_start
    dfx canister create --all
    assert_command dfx build
}

@test "build succeeds if disable shrink" {
    jq '.canisters.e2e_project_backend.shrink=false' dfx.json | sponge dfx.json
    dfx_start
    dfx canister create --all
    assert_command dfx build
}

# TODO: Before Tungsten, we need to update this test for code with inter-canister calls.
# Currently due to new canister ids, the wasm binary will be different for inter-canister calls.
@test "build twice produces the same wasm binary" {
  dfx_start
  dfx canister create --all
  assert_command dfx build
  cp .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm ./old.wasm
  assert_command dfx build
  assert_command diff .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm ./old.wasm
}

@test "build outputs warning" {
    install_asset warning
    dfx_start
    dfx canister create --all
    assert_command dfx build
    assert_match "warning \[M0145\], this pattern of type"
}

@test "build fails on unknown imports" {
    install_asset import_error
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'import error \[M0011\], canister alias "random" not defined'
}

@test "build fails if canister type is not supported" {
  dfx_start
  dfx canister create --all
  jq '.canisters.e2e_project_backend.type="unknown_canister_type"' dfx.json | sponge dfx.json
  assert_command_fail dfx build
  # shellcheck disable=SC2016
  assert_match 'unknown variant `unknown_canister_type`'

  # If canister type is invalid, `dfx stop` fails
  jq '.canisters.e2e_project_backend.type="motoko"' dfx.json | sponge dfx.json
}

@test "can build a custom canister type" {
  install_asset custom_canister
  install_asset wasm/identity
  dfx_start
  dfx canister create --all
  assert_command dfx build custom
  assert_match "CUSTOM_CANISTER_BUILD_DONE"
  assert_command dfx build custom2
  assert_match "CUSTOM_CANISTER2_BUILD_DONE"
  assert_command dfx build custom3

  dfx canister install --all
  assert_command dfx canister call custom fromQuery
  assert_command dfx canister call custom2 fromQuery

  # dfx sets the candid:service metadata
  dfx canister metadata custom candid:service >installed.did
  assert_command diff main.did installed.did
}

@test "upgrade check writes .old.did under .dfx" {
  install_asset custom_canister
  install_asset wasm/identity

  dfx_start
  dfx deploy

  echo yes | dfx deploy --mode=reinstall custom

  # dfx intentionally leaves this file after creating it for comparison,
  # so that the developer can look at the differences too.
  # This test makes sure that the file is created under the .dfx/ directory,
  # which is where other temporary / build artifacts go.
  assert_file_not_exists ./main.old.did
  assert_file_exists .dfx/local/canisters/custom/custom.old.did
}

@test "custom canister build script picks local executable first" {
  install_asset custom_canister
  install_asset wasm/identity

  dfx_start
  dfx canister create custom2
  jq '.canisters.custom2.build="ln"' dfx.json | sponge dfx.json
  mv ./build.sh ./ln

  assert_command dfx build custom2
  assert_match CUSTOM_CANISTER2_BUILD_DONE
}

@test "build succeeds with network parameter" {
  dfx_start
  dfx canister create --all --network local
  assert_command dfx build --network local
}

@test "build succeeds with URL as network parameter" {
    dfx_start
    webserver_port=$(get_webserver_port)
    dfx canister create --all --network "http://127.0.0.1:$webserver_port"
    assert_command dfx build --network "http://127.0.0.1:$webserver_port"
}

@test "build succeeds when requested network is configured" {
  dfx_start

  setup_actuallylocal_shared_network

  assert_command dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
}

@test "build with wallet succeeds when requested network is configured" {
  dfx_start
  setup_actuallylocal_shared_network
  assert_command dfx_set_wallet

  assert_command dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
}

@test "build output for local network is in expected directory" {
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_command ls .dfx/local/canisters/e2e_project_backend/
  assert_command ls .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm
}

@test "build with wallet output for non-local network is in expected directory" {
  dfx_start
  setup_actuallylocal_shared_network
  assert_command dfx_set_wallet

  dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
  assert_command ls .dfx/actuallylocal/canisters/e2e_project_backend/
  assert_command ls .dfx/actuallylocal/canisters/e2e_project_backend/e2e_project_backend.wasm
}

@test "does not add candid:service metadata for a custom canister if there are no build steps" {
  install_asset prebuilt_custom_canister
  install_asset wasm/identity

  dfx_start
  dfx deploy

  # this canister has a build step, so dfx sets the candid metadata
  dfx canister metadata custom_with_build_step candid:service >from_canister.txt
  diff custom_with_build_step.did from_canister.txt

  # this canister doesn't have a build step, so dfx leaves the candid metadata as-is
  dfx canister metadata prebuilt_custom_no_build candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister has a build step, but it is an empty string, so dfx leaves the candid:service metadata as-is
  dfx canister metadata prebuilt_custom_blank_build candid:service >from_canister.txt
  diff main.did from_canister.txt

  # this canister has a build step, but it is an empty array, so dfx leaves the candid:service metadata as-is
  dfx canister metadata prebuilt_custom_empty_build candid:service >from_canister.txt
  diff main.did from_canister.txt
}
