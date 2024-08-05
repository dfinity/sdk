#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new
  install_asset error_context
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "problems reading wallets.json" {
  dfx_start

  assert_command dfx identity get-wallet

  echo "invalid json" >"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"

  assert_command_fail dfx identity get-wallet
  assert_match "failed to parse contents of .*/network/local/wallets.json as json"
  assert_match "expected value at line 1 column 1"

  assert_command_fail dfx wallet upgrade
  assert_match "failed to parse contents of .*/network/local/wallets.json as json"
  assert_match "expected value at line 1 column 1"

  echo '{ "identities": {} }' >"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"

  # maybe you were sudo when you made it
  chmod u=w,go= "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"
  assert_command_fail dfx identity get-wallet
  assert_match "failed to read .*/network/local/wallets.json"
  assert_match "Permission denied"

  assert_command_fail dfx wallet upgrade
  assert_match "failed to read .*/network/local/wallets.json"
  assert_match "Permission denied"

  # can't write it?
  chmod u=r,go= "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"
  assert_command dfx identity new --storage-mode plaintext alice
  assert_command_fail dfx identity get-wallet --identity alice
  assert_match "failed to write to .*/local/wallets.json"
  assert_match "Permission denied"
}

@test "address already in use" {
  dfx_start

  port=$(get_webserver_port)
  address="127.0.0.1:$port"

  # fool dfx start into thinking dfx isn't running
  mv "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/pid" "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/hidden_pid"

  assert_command_fail dfx start  --host "$address"

  # What was the purpose of the address
  assert_match "frontend address"

  # What was the address we were looking for
  assert_match "$address"

  # The underlying cause
  assert_match "Address already in use"

  # Allow dfx stop to stop dfx in teardown.  Otherwise, bats will never exit
  mv "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/hidden_pid" "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/pid"
}

@test "corrupt dfx.json" {
  echo "corrupt" >dfx.json
  assert_command_fail dfx deploy

  # The bare minimum is to mention the file
  assert_match "dfx.json"

  # It's nice to mention the full path to the file
  assert_match "$(pwd)/dfx.json"

  # The underlying cause
  assert_match "expected value at line 1 column 1"
}

@test "packtool missing" {
  dfx_start

  assert_command dfx canister create packtool_missing

  jq '.defaults.build.packtool="not-a-valid-packtool and some parameters"' dfx.json | sponge dfx.json


  assert_command_fail dfx build packtool_missing

  # expect to see the name of the packtool and the parameters
  assert_match '"not-a-valid-packtool" "and" "some" "parameters"'
  # expect to see the underlying cause
  assert_match "No such file or directory"
}

@test "moc missing" {
  use_test_specific_cache_root   # Because this test modifies a file in the cache

  dfx_start

  assert_command dfx canister create m_o_c_missing

  rm -f "$(dfx cache show)/moc"
  assert_command_fail dfx build m_o_c_missing

  # expect to see the name of the binary
  assert_match "moc"

  # expect to see the full path of the binary
  assert_contains "$(dfx cache show)/moc"

  # expect to see the underlying cause
  assert_match "No such file or directory"
}

@test "npm is not installed" {
  dfx_start

  assert_command dfx canister create npm_missing

  # this is how dfx decides to run `npm run build'
  touch package.json

  dfx_path="$(which dfx)"
  # commands needed by assert_command_fail:
  helpers_path="$(which mktemp rm echo | xargs -n 1 dirname | sort | uniq | tr '\n' ':')"
  PATH="$helpers_path" assert_command_fail "$dfx_path" deploy npm_missing

  # expect to see the npm command line
  assert_contains 'program: "npm"'
  assert_match 'args: \[.*"npm".*"run".*"build".*\]'
  # expect to see the name of the canister
  assert_match "npm_missing"
  # expect to see the underlying cause
  assert_match "No such file or directory"
}

@test "missing asset source directory" {
  dfx_start

  assert_command dfx canister create asset_bad_source_path

  assert_command_fail dfx deploy asset_bad_source_path

  # expect to see the bad path
  assert_match "src/does/not/exist"
  # expect to see the name of the canister
  assert_match "asset_bad_source_path"
  # expect to see the underlying cause
  assert_match "No such file or directory"
}

@test "custom bad build step" {
  dfx_start

  assert_command dfx canister create custom_bad_build_step

  assert_command_fail dfx build custom_bad_build_step

  # expect to see what it tried to call
  assert_match "not-the-name-of-an-executable-that-exists"
  # expect to see the name of the canister
  assert_match "custom_bad_build_step"
  # expect to see the underlying cause
  assert_match "The custom tool failed"
}

@test "invalid optimization level" {
  jq '.canisters.bad_optimization_level.optimize="bad_level"' dfx.json | sponge dfx.json
  assert_command_fail dfx_start
  assert_match "expected one of "
}

@test "HTTP 403 has a full diagnosis" {
  dfx_new hello
  install_asset greet
  dfx_start
  assert_command dfx deploy

  # make sure normal status command works
  assert_command dfx canister status hello_backend

  # create a non-controller ID
  assert_command dfx identity new alice --storage-mode plaintext
  assert_command dfx identity use alice

  # calling canister status with different identity provokes HTTP 403
  assert_command_fail dfx canister status hello_backend
  assert_match "not part of the controllers" # this is part of the error explanation
  assert_match "'dfx canister update-settings --add-controller <controller principal to add> <canister id/name or --all> \(--network ic\)'" # this is part of the solution
}

@test "bad wallet canisters get diagnosed" {
  dfx_new hello
  dfx_start
  dfx deploy hello_backend --no-wallet
  id=$(dfx canister id hello_backend)
  dfx identity set-wallet "$id" --force
  assert_command_fail dfx wallet balance
  assert_contains "it did not contain a function that dfx was looking for"
  assert_contains "dfx identity set-wallet <PRINCIPAL> --identity <IDENTITY>"
}
