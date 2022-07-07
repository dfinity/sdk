#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "direct dependencies are built" {
    dfx_start
    dfx canister create --all
    #specify build for only assets_canister
    dfx build e2e_project_frontend

    #validate direct dependency built and is callable
    assert_command dfx canister install e2e_project_backend
    assert_command dfx canister call e2e_project_backend greet World
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
    dfx build e2e_project_backend
    # validate assets canister wasn't built and can't be installed
    assert_command_fail dfx canister install e2e_project_frontend
    assert_match "No such file or directory"
}


@test "manual build of specified canisters succeeds" {
    install_asset assetscanister

    dfx_start
    dfx canister create e2e_project_backend
    dfx build e2e_project_backend
    assert_command dfx canister install e2e_project_backend
    assert_command dfx canister call e2e_project_backend greet World

    assert_command_fail dfx canister install e2e_project_frontend
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_frontend'."
    dfx canister create e2e_project_frontend
    dfx build e2e_project_frontend
    dfx canister install e2e_project_frontend

    assert_command dfx canister call --query e2e_project_frontend retrieve '("/binary/noise.txt")' --output idl
    assert_eq '(blob "\b8\01 \80\0aw12 \00xy\0aKL\0b\0ajk")'

    assert_command dfx canister call --query e2e_project_frontend retrieve '("/text-with-newlines.txt")' --output idl
    assert_eq '(blob "cherries\0ait\27s cherry season\0aCHERRIES")'
}

@test "cyclic dependencies are detected" {
    install_asset transitive_deps_canisters
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build canister_e
    assert_match "The dependency analyzer failed: Found circular dependency: canister_e -> canister_d -> canister_e"
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
