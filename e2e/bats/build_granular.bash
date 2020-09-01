#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

teardown() {
    dfx_stop
}

@test "direct dependencies are built" {
    dfx_start
    dfx canister create --all
    #specify build for only assets_canister
    dfx build e2e_project_assets

    #validate direct dependency built and is callable
    assert_command dfx canister install e2e_project
    assert_command dfx canister call e2e_project greet World
}

@test "transitive dependencies are built" {
    install_asset transitive_deps_canisters
    dfx_start
    dfx canister create --all
    #install of tertiary dependency canister will fail since its not built
    assert_command_fail dfx canister install canister_a
    #specify build for primary canister 
    dfx build canister_c

    #validate tertiary transitive dependency is built and callable
    assert_command dfx canister install canister_a
    assert_command dfx canister call canister_a greet World
    assert_match '("Namaste, World!")'
}

@test "unspecified dependencies are not built" {
    dfx_start
    dfx canister create --all
    # only build motoko canister
    dfx build e2e_project
    # validate assets canister wasn't built and can't be installed
    assert_command_fail dfx canister install e2e_project_assets
    assert_match "No such file or directory"
}


@test "manual build of specified canisters succeeds" {
    install_asset assetscanister

    dfx_start
    dfx canister create e2e_project
    dfx build e2e_project
    assert_command dfx canister install e2e_project
    assert_command dfx canister call e2e_project greet World

    assert_command_fail dfx canister install e2e_project_assets
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_assets'."
    dfx canister create e2e_project_assets
    dfx build e2e_project_assets
    dfx canister install e2e_project_assets

    assert_command dfx canister call --query e2e_project_assets retrieve '("binary/noise.txt")'
    assert_eq '(vec { 184; 1; 32; 128; 10; 119; 49; 50; 32; 0; 120; 121; 10; 75; 76; 11; 10; 106; 107; })'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")'
    assert_eq '(vec { 99; 104; 101; 114; 114; 105; 101; 115; 10; 105; 116; 39; 115; 32; 99; 104; 101; 114; 114; 121; 32; 115; 101; 97; 115; 111; 110; 10; 67; 72; 69; 82; 82; 73; 69; 83; })'

}

@test "cyclic dependencies are detected" {
    install_asset transitive_deps_canisters
    dfx_start
    dfx canister create --all
    assert_command dfx build canister_e
    assert_match "Possible circular dependency detected during evaluation of canister_d's dependency on canister_e."
}

@test "the all flag builds everything" {
    dfx_start
    dfx canister create --all
    assert_command dfx build --all
    assert_command dfx canister install --all
}


@test "the all flags conflicts with canister name" {
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build e2e_project --all
}
