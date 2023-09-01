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

@test "canister url performs as expected on local deploy" {
    dfx_new_frontend hello
    dfx_start
    dfx deploy
    assert_command dfx canister url hello_backend
    assert_eq "http://127.0.0.1:4943/?canisterId=be2us-64aaa-aaaaa-qaabq-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai"
    assert_command dfx canister url hello_frontend
    assert_eq "http://127.0.0.1:4943/?canisterId=bd3sg-teaaa-aaaaa-qaaba-cai"
}

@test "canister url performs as expected on remote canisters" {
    # set dfx.json to string
    echo '{"canisters": {"whoami": {"type": "pull", "id": "ivcos-eqaaa-aaaab-qablq-cai"}}}' > dfx.json
    assert_command dfx canister url whoami --network ic
    assert_eq "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ivcos-eqaaa-aaaab-qablq-cai"
}
