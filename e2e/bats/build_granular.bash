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

    assert_command dfx canister call --query e2e_project_assets retrieve '("binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01\20\80\0a\77\31\32\20\00\78\79\0a\4b\4c\0b\0a\6a\6b")'

    assert_command dfx canister call --query e2e_project_assets retrieve '("text-with-newlines.txt")' --output idl
    assert_eq '(blob "\63\68\65\72\72\69\65\73\0a\69\74\27\73\20\63\68\65\72\72\79\20\73\65\61\73\6f\6e\0a\43\48\45\52\52\49\45\53")'

}

@test "cyclic dependencies are detected" {
    install_asset transitive_deps_canisters
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build canister_e
    assert_match " There is a dependency cycle between canisters found at canister canister_e -> canister_d -> canister_e"
}

@test "multiple non-cyclic dependency paths to the same canister are ok" {
    install_asset transitive_deps_canisters
    dfx_start
    dfx canister create --all
    assert_command dfx build canister_f
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
