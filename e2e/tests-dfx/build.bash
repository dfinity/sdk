#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "build without cargo-audit installed cannot check for vulnerabilities" {
  assert_command rustup default stable
  assert_command rustup target add wasm32-unknown-unknown
  install_asset vulnerable_rust_deps
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_match "Cannot check for vulnerabilities in rust canisters because cargo-audit is not installed."
}

@test "build with vulnerabilities in rust dependencies emits a warning" {
  assert_command rustup default stable
  assert_command rustup target add wasm32-unknown-unknown
  assert_command cargo install cargo-audit
  assert_command cargo audit --version
  install_asset vulnerable_rust_deps
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_match "Audit found vulnerabilities"
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
  # shellcheck disable=SC2094
  cat <<<"$(jq '.canisters.e2e_project_backend.type="unknown_canister_type"' dfx.json)" >dfx.json
  assert_command_fail dfx build
  # shellcheck disable=SC2016
  assert_match 'unknown variant `unknown_canister_type`'

  # If canister type is invalid, `dfx stop` fails
  # shellcheck disable=SC2094
  cat <<<"$(jq '.canisters.e2e_project_backend.type="motoko"' dfx.json)" >dfx.json
}

@test "can build a custom canister type" {
  install_asset custom_canister
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

@test "custom canister build script picks local executable first" {
  install_asset custom_canister
  dfx_start
  dfx canister create custom2
  #shellcheck disable=SC2094
  cat <<<"$(jq '.canisters.custom2.build="ln"' dfx.json)" >dfx.json
  mv ./build.sh ./ln

  assert_command dfx build custom2
  assert_match CUSTOM_CANISTER2_BUILD_DONE
}

@test "build succeeds with network parameter" {
  dfx_start
  dfx canister --network local create --all
  assert_command dfx build --network local
}

@test "build succeeds with URL as network parameter" {
    dfx_start
    webserver_port=$(get_webserver_port)
    dfx canister --network "http://127.0.0.1:$webserver_port" create --all
    assert_command dfx build --network "http://127.0.0.1:$webserver_port"
}

@test "build succeeds when requested network is configured" {
  dfx_start

  setup_actuallylocal_network

  assert_command dfx canister --network actuallylocal create --all
  assert_command dfx build --network actuallylocal
}

@test "build with wallet succeeds when requested network is configured" {
  dfx_start
  setup_actuallylocal_network
  assert_command dfx_set_wallet

  assert_command dfx canister --network actuallylocal create --all
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
  setup_actuallylocal_network
  assert_command dfx_set_wallet

  dfx canister --network actuallylocal create --all
  assert_command dfx build --network actuallylocal
  assert_command ls .dfx/actuallylocal/canisters/e2e_project_backend/
  assert_command ls .dfx/actuallylocal/canisters/e2e_project_backend/e2e_project_backend.wasm
}

@test "does not add candid:service metadata for a custom canister if there are no build steps" {
  install_asset prebuilt_custom_canister

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
