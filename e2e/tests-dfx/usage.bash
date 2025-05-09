load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "dfx help succeeds" {
  dfx --help
}

@test "dfx help contains new command" {
  dfx --help | grep new
}

@test "using an invalid command fails" {
  run dfx blurp
  if [[ $status -eq 0 ]]; then
    echo "$@" >&2
    exit 1
  fi
}

@test "returns the right error if not in a project" {

  assert_command_fail dfx build
  assert_match "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"

  dfx new t --no-frontend
  cd t
  dfx_start
  dfx canister create --all
  assert_command dfx build
}

@test "does not create .dfx just by running dfx even if in a project" {
  echo "{}" >dfx.json
  dfx identity get-principal
  assert_directory_not_exists .dfx
}

@test "does not unconditionally read dfx.json" {
  echo "garbage" >dfx.json
  assert_command dfx identity get-principal
}

@test "--identity and --network are stil accepted as prefix" {
  dfx_new
  install_asset whoami
  dfx_start
  dfx deploy
  dfx identity new alice --storage-mode plaintext
  assert_command dfx --identity alice canister --network local call whoami whoami
  assert_match "$(dfx --identity alice identity get-principal)"
  assert_match "$(dfx identity get-principal --identity alice)"
}

@test "print_mo" {
  dfx_new
  install_asset print
  dfx_start 2>stderr.txt
  dfx canister create --all
  dfx build
  dfx canister install e2e_project
  dfx canister call e2e_project hello
  sleep 2
  run tail -2 stderr.txt
  assert_match "Hello, World! from DFINITY"
}
