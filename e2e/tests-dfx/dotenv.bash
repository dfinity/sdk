#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new_assets
}

teardown() {
  dfx_stop

  standard_teardown
}


@test "puts .env in project root" {
  dfx_start
  jq '.canisters["e2e_project_backend"].post_install="echo post install backend"' dfx.json | sponge dfx.json
  jq '.canisters["e2e_project_frontend"].post_install="echo post install frontend"' dfx.json | sponge dfx.json

  mkdir subdir
  mkdir subdir/canister-install-all subdir/canister-install-single
  mkdir subdir/build-all subdir/build-single
  mkdir subdir/deploy-single subdir/deploy-all
  dfx canister create --all
  ( cd subdir/build-single && dfx build e2e_project_frontend )
  ( cd subdir/build-all && dfx build --all )
  ( cd subdir/canister-install-single && dfx canister install e2e_project_backend )
  dfx canister uninstall-code e2e_project_backend
  ( cd subdir/canister-install-all && dfx canister install --all )
  rm -rf .dfx
  ( cd subdir/deploy-single && dfx deploy e2e_project_backend)
  ( cd subdir/deploy-all && dfx deploy )

  assert_command find . -name .env
  assert_eq "./.env"
}

@test "the output_env_file must be contained within project" {
  dfx_start
  mkdir ../outside

  assert_command_fail dfx deploy --output-env-file nonexistent/.env
  assert_contains "failed to canonicalize output_env_file"
  assert_contains "working-dir/e2e_project/nonexistent: No such file or directory"
  assert_command_fail dfx deploy --output-env-file /etc/passwd
  assert_contains "The output_env_file must be a relative path, but is /etc/passwd"
  assert_command_fail dfx deploy --output-env-file ../outside/.env
  assert_match "The output_env_file must be within the project root, but is .*/working-dir/e2e_project/../outside/.env"
}

@test "writes environment variables to .env" {
  dfx_start
  dfx canister create --all
  # .env should also include canisters that are not explicit dependencies
  jq 'del(.canisters.e2e_project_frontend.dependencies)' dfx.json  | sponge dfx.json
  backend_canister=$(dfx canister id e2e_project_backend)
  frontend_canister=$(dfx canister id e2e_project_frontend)

  assert_command dfx build e2e_project_frontend

  assert_file_exists .env
  env=$(< .env)
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_BACKEND='$backend_canister'" "$env"
  assert_contains "E2E_PROJECT_BACKEND_CANISTER_ID='$backend_canister'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_FRONTEND='$frontend_canister'" "$env"
  assert_contains "E2E_PROJECT_FRONTEND_CANISTER_ID='$frontend_canister'" "$env"

  setup_actuallylocal_project_network
  dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
  assert_contains "DFX_NETWORK='actuallylocal'" "$(< .env)"
}

@test "deploy writes all environment variables to .env" {
  dfx_start
  dfx canister create --all
  # .env should also include canisters that are not explicit dependencies
  jq 'del(.canisters.e2e_project_frontend.dependencies)' dfx.json  | sponge dfx.json
  backend_canister=$(dfx canister id e2e_project_backend)
  frontend_canister=$(dfx canister id e2e_project_frontend)

  assert_command dfx deploy e2e_project_frontend

  assert_file_exists .env
  env=$(< .env)
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_BACKEND='$backend_canister'" "$env"
  assert_contains "E2E_PROJECT_BACKEND_CANISTER_ID='$backend_canister'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_FRONTEND='$frontend_canister'" "$env"
  assert_contains "E2E_PROJECT_FRONTEND_CANISTER_ID='$frontend_canister'" "$env"

  setup_actuallylocal_project_network
  dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
  assert_contains "DFX_NETWORK='actuallylocal'" "$(< .env)"
}

@test "writes all environment variables to .env" {
  dfx_start
  cat dfx.json
  jq '.canisters.second_backend.type="motoko"' dfx.json | sponge dfx.json
  jq '.canisters.second_backend.main="src/e2e_project_backend/main.mo"' dfx.json | sponge dfx.json
  cat dfx.json
  # .env should also include canisters that are not explicit dependencies
  jq 'del(.canisters.e2e_project_frontend.dependencies)' dfx.json  | sponge dfx.json
  cat dfx.json
  dfx canister create --all
  backend_canister=$(dfx canister id e2e_project_backend)
  frontend_canister=$(dfx canister id e2e_project_frontend)
  second_backend_canister=$(dfx canister id second_backend)

  assert_command dfx deploy e2e_project_frontend

  assert_file_exists .env
  env=$(< .env)
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_BACKEND='$backend_canister'" "$env"
  assert_contains "E2E_PROJECT_BACKEND_CANISTER_ID='$backend_canister'" "$env"
  assert_contains "CANISTER_ID_E2E_PROJECT_FRONTEND='$frontend_canister'" "$env"
  assert_contains "E2E_PROJECT_FRONTEND_CANISTER_ID='$frontend_canister'" "$env"
  assert_contains "CANISTER_ID_SECOND_BACKEND='$second_backend_canister'" "$env"
  assert_contains "SECOND_BACKEND_CANISTER_ID='$second_backend_canister'" "$env"

  setup_actuallylocal_project_network
  dfx canister create --all --network actuallylocal
  assert_command dfx build --network actuallylocal
  assert_contains "DFX_NETWORK='actuallylocal'" "$(< .env)"
}

@test "writes environment variables to selected file" {
  dfx_start
  dfx canister create --all

  assert_command dfx build --output-env-file flag.env
  assert_file_exists flag.env
  assert_contains "DFX_NETWORK='local'" "$(< flag.env)"

  jq '.output_env_file="json.env"' dfx.json | sponge dfx.json
  assert_command dfx build
  assert_file_exists json.env
  assert_contains "DFX_NETWORK='local'" "$(< json.env)"

  jq 'del(.output_env_file)' dfx.json | sponge dfx.json
  assert_command dfx build
  assert_file_not_exists .env
}

@test "does not clobber existing .env content" {
  dfx_start
  dfx canister create --all
  echo 'foo=bar' > .env

  assert_command dfx build
  assert_file_exists .env
  env=$(< .env)
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "foo=bar" "$env"

  echo 'baz=quux' >> .env
  assert_command dfx build
  env=$(< .env)
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "foo=bar" "$env"
  assert_contains "baz=quux" "$env"

  # deliberately corrupt the file
  head -n 3 .env | sponge .env
  echo 'baz=quux' >> .env
  assert_command dfx build
  env=$(< .env)
  assert_contains "# END DFX CANISTER ENVIRONMENT VARIABLES" "$env"
  assert_contains "DFX_NETWORK='local'" "$env"
  assert_contains "foo=bar" "$env"
  assert_contains "baz=quux" "$env"
}
