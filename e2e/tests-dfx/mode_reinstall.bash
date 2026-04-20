#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new_assets hello
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "install --mode=reinstall --all fails" {
  dfx_start
  assert_command_fail dfx canister install --mode=reinstall --all

  assert_match "The --mode=reinstall is only valid when specifying a single canister, because reinstallation destroys all data in the canister." "$output"
}

@test "install --mode=reinstall fails if no canister is provided" {
  # This fails because clap protects against it.

  dfx_start
  assert_command_fail dfx canister install --mode=reinstall
  assert_match \
"error: the following required arguments were not provided:
  --all" "$output"
}

@test "reinstall succeeds when a canister name is provided" {
  dfx_start
  dfx deploy

  # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
  # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
  echo yes | (
    assert_command dfx canister install --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
}

@test "install --mode=reinstall refused if not approved" {
  dfx_start
  dfx deploy

  echo no | (
    assert_command_fail dfx canister install --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"

    assert_not_match "Installed code for canister" "$output"
    assert_contains "Refusing to install canister without approval" "$output"
    assert_contains "User declined consent" "$output"
  )
}

@test "deploy --mode=reinstall fails if no canister name specified" {
  dfx_start
  assert_command_fail dfx deploy --mode=reinstall

  assert_match "The --mode=reinstall is only valid when deploying a single canister, because reinstallation destroys all data in the canister." "$output"
}

@test "deploy --mode=reinstall succeeds when a canister name is provided" {
  dfx_start
  dfx deploy

  # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
  # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
  echo yes | (
    assert_command dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
}

@test "deploy --mode=reinstall refused if not approved" {
  dfx_start
  dfx deploy

  echo no | (
    assert_command_fail dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"

    assert_not_match "Installed code for canister" "$output"
    assert_contains "Refusing to install canister without approval" "$output"
    assert_contains "User declined consent" "$output"
  )
}

@test "deploy --mode=reinstall does not reinstall dependencies" {
  dfx_start
  install_asset counter
  dfx deploy

  assert_command dfx canister call hello_backend read
  assert_eq "(0 : nat)" "$output"

  assert_command dfx canister call hello_backend inc
  assert_eq "()" "$output"

  assert_command dfx canister call hello_backend read
  assert_eq "(1 : nat)" "$output"

  dfx canister call hello_backend inc
  assert_command dfx canister call hello_backend read
  assert_eq "(2 : nat)" "$output"


  # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
  # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
  echo "yes" | (
    assert_command dfx deploy --mode=reinstall hello_frontend

    assert_match "You are about to reinstall the hello_frontend canister." "$output"
    assert_not_match "You are about to reinstall the hello_backend canister." "$output"
    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_frontend," "$output"
  )

  # the hello_backend canister should not have been upgraded (which would reset the non-stable var)
  assert_command dfx canister call hello_backend read
  assert_eq "(2 : nat)" "$output"
}

@test "confirmation dialogue accepts multiple forms of 'yes'" {
  dfx_start
  dfx deploy

  # if the pipe is alone with assert_command, $stdout, $stderr etc will not be available,
  # so all the assert_match calls will fail.  http://mywiki.wooledge.org/BashFAQ/024
  echo yes | (
    assert_command dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
  echo y | (
    assert_command dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
  echo YES | (
    assert_command dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
  echo YeS | (
    assert_command dfx deploy --mode=reinstall hello_backend

    assert_match "YOU WILL LOSE ALL DATA IN THE CANISTER" "$output"
    assert_match "Reinstalled code for canister hello_backend" "$output"
  )
}
