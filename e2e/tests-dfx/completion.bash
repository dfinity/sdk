#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
  use_test_specific_cache_root # because extensions go in the cache
}

teardown() {
  standard_teardown
}

@test "generate bash completion script" {
  assert_command dfx completion --shell bash
  assert_contains "dfx__ledger__transfer"
  assert_contains "dfx__identity__new"
  assert_contains "dfx__identity__help__whoami"
}

@test "generate bash completion script with extensions installed" {
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install sns --version 0.3.1
  assert_command dfx completion --shell bash
  assert_contains "dfx__ledger__transfer"
  assert_contains "dfx__identity__new"
  assert_contains "dfx__identity__help__whoami"
  assert_contains "dfx__nns__install"
  assert_contains "dfx__nns__help__install"
}

@test "generate zsh completion script" {
  assert_command dfx completion --shell zsh
  assert_contains "_dfx__ledger__help__balance_commands"
  assert_contains "_dfx__canister__install_commands"
}

@test "generate zsh completion script with extensions installed" {
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install sns --version 0.3.1
  assert_command dfx completion --shell zsh
  assert_contains "_dfx__ledger__help__balance_commands"
  assert_contains "_dfx__canister__install_commands"
  assert_contains "_dfx__nns__install_commands"
  assert_contains "_dfx__nns__help__install_commands"
  assert_contains "_dfx__sns__propose_commands"
}

@test "generate elvish completion script" {
  assert_command dfx completion --shell elvish
  assert_contains "dfx;help;identity;new"
  assert_contains "dfx;canister;create"
}

@test "generate elvish completion script with extensions installed" {
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install sns --version 0.3.1
  assert_command dfx completion --shell elvish
  assert_contains "dfx;nns;install"
  assert_contains "dfx;help;sns;deploy"
}

@test "generate fish completion script" {
  assert_command dfx completion --shell fish
 assert_contains "Deploys all or a specific canister from the code in your project. By default, all canisters are deployed"
}

@test "generate fish completion script with extensions installed" {
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install sns --version 0.3.1
  assert_command dfx completion --shell fish
  assert_contains "Install an NNS on the local dfx server"
  assert_contains "Subcommand for preparing dapp canister(s) for 1-proposal SNS creation"
}

@test "generate powershell completion script" {
  assert_command dfx completion --shell powershell
  assert_contains "dfx;deploy"
  assert_contains "dfx;canister;create"
}

@test "generate powershell completion script with extensions installed" {
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install sns --version 0.3.1
  assert_command dfx completion --shell powershell
  assert_contains "dfx;ledger;transfer"
  assert_contains "dfx;nns;install"
  assert_contains "dfx;help;sns;deploy"
}
