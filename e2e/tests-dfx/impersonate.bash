#!/usr/bin/env bats

load ../utils/_
  
setup() {
  standard_setup

  dfx_new hello
}
  
teardown() {
  dfx_stop
  
  standard_teardown
}

@test "impersonate sender" {
    [[ ! "$USE_POCKETIC" ]] && skip "skipped for replica: impersonating sender is only supported for PocketIC"

    dfx_start
    assert_command dfx deploy hello_backend

    # set the management canister as the only controller
    assert_command dfx canister update-settings hello_backend --set-controller aaaaa-aa --yes

    # updating settings now fails because the default identity does not control the canister anymore
    assert_command_fail dfx canister update-settings hello_backend --freezing-threshold 0
    assert_contains "Only controllers of canister $(dfx canister id hello_backend) can call ic00 method update_settings"

    # updating settings succeeds when impersonating the management canister as the sender
    assert_command dfx canister update-settings hello_backend --freezing-threshold 0 --impersonate aaaaa-aa

    # test management canister call failure (setting memory allocation to a low value)
    assert_command_fail dfx canister update-settings hello_backend --memory-allocation 1 --impersonate aaaaa-aa
    assert_contains "Management canister call failed: IC0402: Canister was given 1 B memory allocation but at least"
    
    # canister status fails because the default identity does not control the canister anymore
    assert_command_fail dfx canister status hello_backend
    assert_contains "Only controllers of canister $(dfx canister id hello_backend) can call ic00 method canister_status"
    
    # canister status succeeds when impersonating the management canister as the sender
    assert_command dfx canister status hello_backend --impersonate aaaaa-aa
    assert_contains "Controllers: aaaaa-aa"
    assert_contains "Freezing threshold: 0"

    # freeze the canister
    assert_command dfx canister update-settings hello_backend --freezing-threshold 9223372036854775808 --confirm-very-long-freezing-threshold --impersonate aaaaa-aa

    # test management canister call submission failure
    assert_command_fail dfx canister status hello_backend --impersonate aaaaa-aa
    assert_contains "Failed to submit management canister call: IC0207: Canister $(dfx canister id hello_backend) is out of cycles"

    # test update call submission failure
    assert_command_fail dfx canister call aaaaa-aa canister_status "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --update --impersonate aaaaa-aa
    assert_contains "Failed to submit canister call: IC0207: Canister $(dfx canister id hello_backend) is out of cycles"

    # test async call submission failure
    assert_command_fail dfx canister call aaaaa-aa canister_status "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --async --impersonate aaaaa-aa
    assert_contains "Failed to submit canister call: IC0207: Canister $(dfx canister id hello_backend) is out of cycles"

    # unfreeze the canister
    assert_command dfx canister update-settings hello_backend --freezing-threshold 0 --impersonate aaaaa-aa

    # test update call failure
    assert_command_fail dfx canister call aaaaa-aa delete_canister "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --update --impersonate aaaaa-aa
    assert_contains "Canister call failed: IC0510: Canister $(dfx canister id hello_backend) must be stopped before it is deleted."

    # test update call
    assert_command dfx canister call aaaaa-aa start_canister "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --update --impersonate aaaaa-aa
    assert_contains "()"

    # test async call
    assert_command dfx canister call aaaaa-aa canister_status "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --async --impersonate aaaaa-aa
    assert_contains "Request ID:"

    # test query call failure
    assert_command_fail dfx canister call aaaaa-aa fetch_canister_logs "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --query --impersonate "$(dfx canister id hello_backend)"
    assert_contains "Failed to perform query call: IC0406: Caller $(dfx canister id hello_backend) is not allowed to query ic00 method fetch_canister_logs"

    # test query call
    assert_command dfx canister call aaaaa-aa fetch_canister_logs "(record { canister_id=principal\"$(dfx canister id hello_backend)\" })" --query --impersonate aaaaa-aa
    assert_contains "(record { 1_754_302_831 = vec {} })"
}
