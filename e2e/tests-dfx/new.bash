#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop
  standard_teardown
}

@test "dfx new - good names" {
  dfx new --no-frontend a_good_name_
  dfx new --no-frontend A
  dfx new --no-frontend b
  dfx new --no-frontend a_
  dfx new --no-frontend a_1
  dfx new --no-frontend a1
  dfx new --no-frontend a1a
}

@test "dfx new - bad names" {
  assert_command_fail dfx new _a_good_name_
  assert_command_fail dfx new __also_good
  assert_command_fail dfx new _1
  assert_command_fail dfx new _a
  assert_command_fail dfx new 1
  assert_command_fail dfx new 1_
  assert_command_fail dfx new -
  assert_command_fail dfx new _
  assert_command_fail dfx new '🕹'
  assert_command_fail dfx new '不好'
  assert_command_fail dfx new 'a:b'
}

@test "dfx new --help shows possible backend template names" {
  assert_command dfx new --help
  assert_match "\[possible values.*motoko.*\]"
  assert_match "\[possible values.*rust.*\]"
  assert_match "\[possible values.*kybra.*\]"
  assert_match "\[possible values.*azle.*\]"
}

@test "dfx new --type <bad type> shows possible values" {
  assert_command_fail dfx new --type bad_type
  assert_match "\[possible values.*motoko.*\]"
  assert_match "\[possible values.*rust.*\]"
  assert_match "\[possible values.*kybra.*\]"
  assert_match "\[possible values.*azle.*\]"
}

@test "dfx new readmes contain appropriate links" {
  assert_command dfx new --type rust e2e_rust --no-frontend
  assert_command grep "https://docs.rs/ic-cdk" e2e_rust/README.md
  assert_command dfx new --type motoko e2e_motoko --no-frontend
  assert_command grep "https://internetcomputer.org/docs/current/motoko/main/language-manual" e2e_motoko/README.md
}

@test "dfx new emits projects of the correct type" {
  assert_command dfx new --type rust e2e_rust --no-frontend
  assert_command jq -r '.canisters.e2e_rust_backend.type' e2e_rust/dfx.json
  assert_eq "rust"
  assert_command dfx new --type motoko e2e_motoko --no-frontend
  assert_command jq -r '.canisters.e2e_motoko_backend.type' e2e_motoko/dfx.json
  assert_eq "motoko"
}

@test "frontend templates apply successfully" {
  for frontend in sveltekit vue react vanilla simple-assets none; do
    assert_command dfx new e2e_${frontend/-/_} --frontend $frontend
  done
  assert_file_not_exists e2e_none/src/e2e_none_frontend
}

@test "frontend templates pass the frontend tests" {
  dfx_start
  for frontend in sveltekit vue react vanilla; do
    assert_command dfx new e2e_$frontend --frontend $frontend --extras frontend-tests
    pushd e2e_$frontend
    assert_command dfx deploy
    assert_command npm test --workspaces
    popd
  done
}

@test "backend templates" {
  for backend in motoko rust kybra azle; do
    assert_command dfx new e2e_$backend --type $backend --no-frontend
  done
}

@test "interactive template selection" {
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/rust_svelte_with_tests_and_ii.exp"
  assert_file_exists e2e_project/Cargo.toml
  assert_file_exists e2e_project/src/e2e_project_frontend/src/routes/+page.svelte
  assert_file_exists e2e_project/src/e2e_project_frontend/src/setupTests.js
  assert_command jq .canisters.internet_identity e2e_project/dfx.json
}

@test "hyphenated names" {
  dfx_start
  assert_command dfx new e2e-project --type motoko --frontend vanilla --extras frontend-tests
  cd e2e-project
  assert_command jq '.canisters["e2e-project-backend","e2e-project-frontend"]' dfx.json
  assert_command dfx deploy
  assert_command npm test --workspaces
}

@test "variants of node and npm installed or not" {
  which node || skip "node not installed"
  which npm || skip "npm not installed"

  mkdir node-only
  cp "$(which node)" node-only/node

  mkdir dfx-only
  cp "$(which dfx)" dfx-only/dfx

  mkdir node-installable
  cat > node-installable/node <<EOF
#!/bin/bash
echo "Command 'node' not found, but can be installed with:" >&2
echo "apt install npm" >&2
echo "Please ask your administrator."
exit 127
EOF
  chmod +x node-installable/node

  mkdir npm-installable
  cat > npm-installable/npm <<EOF
#!/bin/bash
echo "Command 'npm' not found, but can be installed with:" >&2
echo "apt install npm" >&2
echo "Please ask your administrator."
exit 127
EOF
  chmod +x npm-installable/npm

  DFX_ONLY_DIR=$PWD/dfx-only
  NODE_ONLY_DIR=$PWD/node-only
  NODE_INSTALLABLE_DIR=$PWD/node-installable
  NPM_INSTALLABLE_DIR=$PWD/npm-installable

  # neither node nor npm are installed (no binaries)
  PATH="/usr/bin:/bin:$DFX_ONLY_DIR" \
    assert_command dfx new e2e_project1 --type motoko --frontend sveltekit
  assert_contains "Node could not be found. Skipping installing the frontend example code."
  assert_contains "npm could not be found. Skipping installing the frontend example code."
  assert_contains "You can bypass this check by using the --frontend flag."

  # node is installable, but not installed
  PATH="/usr/bin:/bin:$DFX_ONLY_DIR:$NODE_INSTALLABLE_DIR" \
    assert_command dfx new e2e_project2 --type motoko --frontend sveltekit
  assert_contains "Node could not be found. Skipping installing the frontend example code."
  assert_contains "npm could not be found. Skipping installing the frontend example code."
  assert_contains "You can bypass this check by using the --frontend flag."

  # node is installed, but there is no npm binary
  PATH="/usr/bin:/bin:$DFX_ONLY_DIR:$NODE_ONLY_DIR" \
    assert_command dfx new e2e_project3 --type motoko --frontend sveltekit
  assert_not_contains "Node could not be found"
  assert_contains "npm could not be found. Skipping installing the frontend example code."
  assert_contains "You can bypass this check by using the --frontend flag."

  # node is installed; npm is not, but a stub reports that it is installable
  PATH="/usr/bin:/bin:$DFX_ONLY_DIR:$NODE_ONLY_DIR:$NPM_INSTALLABLE_DIR" \
    assert_command dfx new e2e_project4 --type motoko --frontend sveltekit
  assert_not_contains "Node could not be found"
  assert_contains "npm could not be found. Skipping installing the frontend example code."
  assert_contains "You can bypass this check by using the --frontend flag."
}
